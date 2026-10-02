use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};

use crate::models::credit_cards::{
    self, CreateCreditCardRequest, CreditCardSummary, UpdateCreditCardRequest,
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/credit-cards")
        .add("/summary", get(summary))
        .add("/", get(list).post(create))
        .add("/{id}", get(show).put(update).delete(remove))
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
