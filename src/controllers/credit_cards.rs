use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    Set, TransactionSession, TransactionTrait,
};

use crate::ledger;
use crate::models::credit_cards::{
    self, CreateCreditCardRequest, CreditCardResponse, CreditCardSummary, UpdateCreditCardRequest,
};
use crate::models::{banks, card_repayment};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/credit-cards")
        .add("/summary", get(summary))
        .add("/{id}/statement", get(statement))
        .add("/{id}/repay", post(repay))
        .add("/", get(list).post(create))
        .add("/{id}", get(show).put(update).delete(remove))
}

/// Money is `f64` in the schema, so round at every boundary.
fn round2(v: f64) -> f64 {
    let r = (v * 100.0).round() / 100.0;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RepayCardRequest {
    /// Amount to pay towards the card. Clamped to the amount owed.
    pub amount: f64,
    /// Bank the repayment is drawn from. Absent or blank means a cash repayment,
    /// which leaves every bank balance untouched.
    pub bank_id: Option<String>,
}

/// Pay down a card's outstanding balance.
///
/// The two movements the money actually makes — out of the bank (when one is
/// named) and off the card — are applied exactly once here. The `expenses` row
/// this writes is deliberately *unlinked*: it records the repayment in the
/// activity trail for the user to see, but carries no `bank_id` or
/// `credit_card_id`, so the expense ledger hooks leave it alone and the money is
/// not moved a second time.
#[debug_handler]
async fn repay(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<RepayCardRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let uid = super::uid(&ctx, &auth).await?;

    if !params.amount.is_finite() || params.amount <= 0.0 {
        return Err(Error::BadRequest(
            "amount must be greater than zero".to_string(),
        ));
    }

    let Some(card) = credit_cards::Entity::find_by_id(&id)
        .filter(credit_cards::Column::UserId.eq(uid))
        .one(&*db)
        .await?
    else {
        return not_found();
    };

    // Blank and "0" ids mean "not linked", the same rule the ledger applies.
    let bank_id = params
        .bank_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "0")
        .map(str::to_string);

    if let Some(bank_id) = bank_id.as_deref() {
        let owned = banks::Entity::find_by_id(bank_id)
            .filter(banks::Column::UserId.eq(uid))
            .one(&*db)
            .await?;
        if owned.is_none() {
            return not_found();
        }
    }

    // Never repay more than is owed, and never drive the balance below zero.
    let applied = round2(params.amount.min(round2(card.current_balance)).max(0.0));
    if applied <= 0.0 {
        return Err(Error::BadRequest(
            "card has no outstanding balance to repay".to_string(),
        ));
    }

    let txn = db
        .begin()
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    let result = apply_repayment(&txn, uid, &card, applied, bank_id.as_ref()).await;
    let (card_out, repayment) = match result {
        Ok(v) => {
            if let Err(e) = txn.commit().await {
                return Err(Error::string(&e.to_string()));
            }
            v
        }
        Err(e) => {
            let _ = txn.rollback().await;
            return Err(e);
        }
    };

    let bank = match bank_id.as_deref() {
        Some(bid) => {
            banks::Entity::find_by_id(bid)
                .filter(banks::Column::UserId.eq(uid))
                .one(&*db)
                .await?
        }
        None => None,
    };

    format::json(serde_json::json!({
        "card": CreditCardResponse::from(card_out),
        "bank": bank,
        "repayment": repayment,
        "amount_applied": applied,
        "bank_id": bank_id,
    }))
}

/// The write half of a repayment, run inside the caller's transaction so a
/// failure leaves nothing half-applied.
async fn apply_repayment(
    db: &impl ConnectionTrait,
    uid: i64,
    card: &credit_cards::Model,
    applied: f64,
    bank_id: Option<&String>,
) -> Result<(credit_cards::Model, card_repayment::Model)> {
    let now = Utc::now();
    let stamp = now.to_rfc3339();
    let now = Utc::now();
    let stamp = now.to_rfc3339();
    // A repayment is a transfer (bank -> card), not an expense: the purchase it
    // settles was already recorded when the card was charged. Writing it as an
    // expense would double-count spending in lists, summaries and analytics, so
    // it lives in card_repayments and never touches the expenses ledger.
    let repayment = card_repayment::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(uid),
        card_id: Set(card.id.clone()),
        bank_id: Set(bank_id.map(|b| b.to_string())),
        amount: Set(applied),
        paid_on: Set(now.format("%Y-%m-%d").to_string()),
        created_at: Set(stamp),
    }
    .insert(db)
    .await
    .map_err(|e| Error::string(&e.to_string()))?;

    ledger::adjust_bank(db, uid, bank_id, -applied)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    ledger::adjust_card(db, uid, Some(&card.id), -applied)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    let card_out = credit_cards::Entity::find_by_id(&card.id)
        .filter(credit_cards::Column::UserId.eq(uid))
        .one(db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?
        .ok_or_else(|| Error::BadRequest("card no longer exists".to_string()))?;

    Ok((card_out, repayment))
}

#[debug_handler]
async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<serde_json::Value>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let search = params.get("search").and_then(|v| v.as_str()).unwrap_or("");
    let page = params.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
    let per_page = params.get("perPage").and_then(|v| v.as_u64()).unwrap_or(50);

    let mut query = credit_cards::Entity::find()
        .filter(credit_cards::Column::UserId.eq(super::uid(&ctx, &auth).await?));
    if !search.is_empty() {
        query = query.filter(
            Condition::any()
                .add(credit_cards::Column::Name.contains(search))
                .add(credit_cards::Column::BankName.contains(search)),
        );
    }
    query = query.order_by_desc(credit_cards::Column::UpdatedAt);

    let total = query.clone().count(&*db).await?;
    let items = query
        .paginate(&*db, per_page)
        .fetch_page(page.saturating_sub(1))
        .await?;

    format::json(serde_json::json!({
        "data": items,
        "total": total,
        "page": page,
        "perPage": per_page
    }))
}

#[debug_handler]
async fn show(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let item = credit_cards::Entity::find_by_id(&id)
        .filter(credit_cards::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    match item {
        Some(item) => format::json(item),
        None => not_found(),
    }
}

/// Every charge that moved this card, newest first, with the running amount owed walked
/// backwards from the stored `current_balance`.
#[debug_handler]
async fn statement(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let uid = super::uid(&ctx, &auth).await?;
    let Some(item) = credit_cards::Entity::find_by_id(id)
        .filter(credit_cards::Column::UserId.eq(uid))
        .one(&*db)
        .await?
    else {
        return not_found();
    };
    let entries = crate::statement::for_card(&*db, uid, &item.id, item.current_balance)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(entries)
}

#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateCreditCardRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let now = Utc::now().to_rfc3339();
    let credit_limit = params.credit_limit.unwrap_or(0.0);
    let current_balance = params.current_balance.unwrap_or(0.0);
    let active = credit_cards::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(super::uid(&ctx, &auth).await?),
        name: Set(params.name),
        bank_name: Set(params.bank_name),
        card_type: Set(params.card_type),
        last_four_digits: Set(params.last_four_digits),
        credit_limit: Set(credit_limit),
        current_balance: Set(current_balance),
        available_credit: Set((credit_limit - current_balance).max(0.0)),
        interest_rate: Set(params.interest_rate.unwrap_or(0.0)),
        billing_cycle_day: Set(params.billing_cycle_day.unwrap_or(1)),
        due_date_day: Set(params.due_date_day.unwrap_or(5)),
        annual_fee: Set(params.annual_fee.unwrap_or(0.0)),
        reward_program: Set(params.reward_program),
        reward_points: Set(params.reward_points.unwrap_or(0)),
        is_active: Set(params.is_active.unwrap_or(true)),
        sync_status: Set("pending".to_string()),
        last_synced_at: Set(None),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now)),
    };
    let item = active
        .insert(&*db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(item)
}

#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<UpdateCreditCardRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = credit_cards::Entity::find_by_id(&id)
        .filter(credit_cards::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };
    let mut active = existing.into_active_model();
    if let Some(v) = params.name {
        active.name = Set(v);
    }
    if let Some(v) = params.bank_name {
        active.bank_name = Set(v);
    }
    if let Some(v) = params.card_type {
        active.card_type = Set(v);
    }
    if let Some(v) = params.last_four_digits {
        active.last_four_digits = Set(v);
    }
    if let Some(v) = params.credit_limit {
        active.credit_limit = Set(v);
    }
    if let Some(v) = params.current_balance {
        active.current_balance = Set(v);
    }
    let cl = match active.credit_limit.clone() {
        sea_orm::ActiveValue::Unchanged(v) | sea_orm::ActiveValue::Set(v) => v,
        sea_orm::ActiveValue::NotSet => 0.0,
    };
    let cb = match active.current_balance.clone() {
        sea_orm::ActiveValue::Unchanged(v) | sea_orm::ActiveValue::Set(v) => v,
        sea_orm::ActiveValue::NotSet => 0.0,
    };
    active.available_credit = Set((cl - cb).max(0.0));
    if let Some(v) = params.interest_rate {
        active.interest_rate = Set(v);
    }
    if let Some(v) = params.billing_cycle_day {
        active.billing_cycle_day = Set(v);
    }
    if let Some(v) = params.due_date_day {
        active.due_date_day = Set(v);
    }
    if let Some(v) = params.annual_fee {
        active.annual_fee = Set(v);
    }
    if let Some(v) = params.reward_program {
        active.reward_program = Set(Some(v));
    }
    if let Some(v) = params.reward_points {
        active.reward_points = Set(v);
    }
    if let Some(v) = params.is_active {
        active.is_active = Set(v);
    }
    active.sync_status = Set("pending".to_string());
    active.updated_at = Set(Some(Utc::now().to_rfc3339()));
    let item = active
        .update(&*db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(item)
}

#[debug_handler]
async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = credit_cards::Entity::find_by_id(&id)
        .filter(credit_cards::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    if existing.is_none() {
        return not_found();
    }
    credit_cards::Entity::delete_by_id(&id)
        .exec(&*db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(serde_json::json!({ "ok": true }))
}

#[debug_handler]
async fn summary(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let rows = credit_cards::Entity::find()
        .filter(credit_cards::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .all(&*db)
        .await?;
    let s = CreditCardSummary {
        total_limit: rows.iter().map(|r| r.credit_limit).sum(),
        total_balance: rows.iter().map(|r| r.current_balance).sum(),
        total_available: rows.iter().map(|r| r.available_credit).sum(),
        count: rows.len() as i64,
    };
    format::json(s)
}
