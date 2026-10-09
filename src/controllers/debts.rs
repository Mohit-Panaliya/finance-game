use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};

use crate::models::debts::{self, CreateDebtRequest, DebtResponse, DebtSummary, UpdateDebtRequest};

/// Money is `f64` in the schema, so round at every boundary.
fn round2(v: f64) -> f64 {
    let r = (v * 100.0).round() / 100.0;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

/// Loan figures are enforced here rather than with CHECK constraints (the Turso
/// engine refuses to open a database that contains them). Negative amounts or
/// tenures are a client error, not a validation failure, so they surface as 400.
fn check_loan_fields(interest: Option<f64>, emi: Option<f64>, tenure: Option<i64>) -> Result<()> {
    let bad_money = |v: Option<f64>| v.map(|x| !x.is_finite() || x < 0.0).unwrap_or(false);
    if bad_money(interest) || bad_money(emi) || tenure.map(|t| t < 0).unwrap_or(false) {
        return Err(Error::BadRequest(
            "loan interest, EMI and tenure cannot be negative".to_string(),
        ));
    }
    Ok(())
}

/// Reads a boolean query flag. Query strings arrive as JSON strings, so `as_bool()`
/// would always miss and the flag would silently keep its default.
fn query_flag(value: Option<&serde_json::Value>, default: bool) -> bool {
    match value {
        Some(serde_json::Value::String(s)) => {
            matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes")
        }
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(1) != 0,
        _ => default,
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/debts")
        .add("/summary", get(summary))
        .add("/", get(list).post(create))
        .add("/{id}/settle", post(settle))
        .add("/{id}/top-up", post(top_up))
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
    let search = params
        .get("search")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let direction = params
        .get("direction")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let include_settled = query_flag(params.get("includeSettled"), true);
    let page = params.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
    let per_page = params.get("perPage").and_then(|v| v.as_u64()).unwrap_or(50);

    let mut query =
        debts::Entity::find().filter(debts::Column::UserId.eq(super::uid(&ctx, &auth).await?));
    if !search.is_empty() {
        query = query.filter(
            Condition::any()
                .add(debts::Column::Counterparty.contains(&search))
                .add(debts::Column::Note.contains(&search))
                .add(debts::Column::Kind.contains(&search)),
        );
    }
    if direction == "lent" || direction == "borrowed" {
        query = query.filter(debts::Column::Direction.eq(direction.clone()));
    }

    if !include_settled {
        query = query.filter(debts::Column::SettledAmount.eq(0.0));
    }

    let total = query.clone().count(&*db).await?;
    let rows = query
        .order_by_desc(debts::Column::OccurredDate)
        .paginate(&*db, per_page)
        .fetch_page(page.saturating_sub(1))
        .await?;

    let items: Vec<DebtResponse> = rows.iter().cloned().map(DebtResponse::from).collect();

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
    let item = debts::Entity::find_by_id(&id)
        .filter(debts::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    match item {
        Some(item) => format::json(DebtResponse::from(item)),
        None => not_found(),
    }
}

#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateDebtRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    check_loan_fields(
        params.interest_rate,
        params.emi_amount,
        params.tenure_months,
    )?;
    let now = Utc::now().to_rfc3339();
    let active = debts::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(super::uid(&ctx, &auth).await?),
        direction: Set(params.direction.unwrap_or_else(|| "lent".to_string())),
        counterparty: Set(params.counterparty),
        amount: Set(round2(params.amount.unwrap_or(0.0))),
        settled_amount: Set(round2(params.settled_amount.unwrap_or(0.0))),
        currency: Set(params.currency.unwrap_or_else(|| "INR".to_string())),
        kind: Set(params.kind.unwrap_or_else(|| "loan".to_string())),
        interest_rate: Set(params.interest_rate.unwrap_or(0.0)),
        emi_amount: Set(params.emi_amount.unwrap_or(0.0)),
        tenure_months: Set(params.tenure_months.unwrap_or(0)),
        lender: Set(params.lender),
        account_id: Set(params.account_id),
        occurred_date: Set(params
            .occurred_date
            .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string())),
        due_date: Set(params.due_date),
        note: Set(params.note),
        settled_date: Set(params.settled_date),
        sync_status: Set("pending".to_string()),
        last_synced_at: Set(None),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now)),
    };
    let item = active
        .insert(&*db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(DebtResponse::from(item))
}

#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<UpdateDebtRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = debts::Entity::find_by_id(&id)
        .filter(debts::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };
    check_loan_fields(
        params.interest_rate,
        params.emi_amount,
        params.tenure_months,
    )?;
    let mut active = existing.into_active_model();
    if let Some(v) = params.direction {
        active.direction = Set(v);
    }
    if let Some(v) = params.counterparty {
        active.counterparty = Set(v);
    }
    if let Some(v) = params.amount {
        active.amount = Set(round2(v));
    }
    if let Some(v) = params.settled_amount {
        active.settled_amount = Set(round2(v));
    }
    if let Some(v) = params.currency {
        active.currency = Set(v);
    }
    if let Some(v) = params.kind {
        active.kind = Set(v);
    }
    if let Some(v) = params.interest_rate {
        active.interest_rate = Set(v);
    }
    if let Some(v) = params.emi_amount {
        active.emi_amount = Set(v);
    }
    if let Some(v) = params.tenure_months {
        active.tenure_months = Set(v);
    }
    if let Some(v) = params.lender {
        active.lender = Set(Some(v));
    }
    if let Some(v) = params.account_id {
        active.account_id = Set(Some(v));
    }
    if let Some(v) = params.occurred_date {
        active.occurred_date = Set(v);
    }
    if let Some(v) = params.due_date {
        active.due_date = Set(Some(v));
    }
    if let Some(v) = params.note {
        active.note = Set(Some(v));
    }
    if let Some(v) = params.settled_date {
        active.settled_date = Set(Some(v));
    }
    active.sync_status = Set("pending".to_string());
    active.updated_at = Set(Some(Utc::now().to_rfc3339()));
    let item = active
        .update(&*db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(DebtResponse::from(item))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SettleDebtRequest {
    /// Amount to mark as repaid. Clamped to what is still outstanding so a
    /// double-tap can never drive `settled_amount` past `amount`.
    pub amount: Option<f64>,
    pub settled_date: Option<String>,
}

#[debug_handler]
async fn settle(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<SettleDebtRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = debts::Entity::find_by_id(&id)
        .filter(debts::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };

    let open = existing.outstanding();
    let already_settled = existing.settled_amount;
    let amount = match params.amount {
        Some(v) if v.is_finite() && v > 0.0 => v.min(open),
        _ => open,
    };

    let mut active = existing.into_active_model();
    active.settled_amount = Set(already_settled + amount);
    active.settled_date =
        Set(Some(params.settled_date.unwrap_or_else(|| {
            Utc::now().format("%Y-%m-%d").to_string()
        })));
    active.sync_status = Set("pending".to_string());
    active.updated_at = Set(Some(Utc::now().to_rfc3339()));

    let item = active
        .update(&*db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(DebtResponse::from(item))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TopUpDebtRequest {
    /// Extra principal to add to the debt. Must be greater than zero.
    pub amount: Option<f64>,
}

/// Add more money to an existing debt ("I borrowed another 20k on top").
///
/// The extra principal is added to `amount`, so the outstanding balance grows by
/// the same figure. A previously settled debt re-opens and drops its settlement
/// date, because there is now money owed again.
#[debug_handler]
async fn top_up(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<TopUpDebtRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let Some(existing) = debts::Entity::find_by_id(&id)
        .filter(debts::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?
    else {
        return not_found();
    };

    let add = match params.amount {
        Some(v) if v.is_finite() && v > 0.0 => round2(v),
        _ => {
            return Err(Error::BadRequest(
                "amount must be greater than zero".to_string(),
            ))
        }
    };

    let new_amount = round2(existing.amount + add);
    let mut active = existing.clone().into_active_model();
    active.amount = Set(new_amount);
    // Re-opening a settled debt clears the date the books were closed.
    if new_amount - existing.settled_amount > f64::EPSILON {
        active.settled_date = Set(None);
    }
    active.sync_status = Set("pending".to_string());
    active.updated_at = Set(Some(Utc::now().to_rfc3339()));

    let item = active
        .update(&*db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(DebtResponse::from(item))
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
    let existing = debts::Entity::find_by_id(&id)
        .filter(debts::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    if existing.is_none() {
        return not_found();
    }
    debts::Entity::delete_by_id(&id)
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
    let rows = debts::Entity::find()
        .filter(debts::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .all(&*db)
        .await?;
    format::json(DebtSummary::from_rows(&rows, Utc::now().date_naive()))
}
