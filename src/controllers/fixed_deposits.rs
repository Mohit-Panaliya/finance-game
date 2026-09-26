use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

use crate::models::fixed_deposits::{
    self, CreateFixedDepositRequest, FixedDepositSummary, UpdateFixedDepositRequest,
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/fixed-deposits")
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
    let per_page = params
        .get("perPage")
        .and_then(|v| v.as_u64())
        .unwrap_or(50);

    let mut query =
        fixed_deposits::Entity::find().filter(fixed_deposits::Column::UserId.eq(super::uid(&ctx, &auth).await?));
    if !search.is_empty() {
        query = query.filter(
            Condition::any()
                .add(fixed_deposits::Column::Name.contains(search))
                .add(fixed_deposits::Column::Status.contains(search)),
        );
    }
    query = query.order_by_desc(fixed_deposits::Column::UpdatedAt);

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
    let item = fixed_deposits::Entity::find_by_id(&id)
        .filter(fixed_deposits::Column::UserId.eq(super::uid(&ctx, &auth).await?))
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
    Json(params): Json<CreateFixedDepositRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let now = Utc::now().to_rfc3339();
    let principal = params.principal_amount;
    let rate = params.interest_rate;
    let tenure = params.tenure_months;
    let current_value = principal * (1.0 + rate / 100.0 * (tenure as f64 / 12.0));
    let active = fixed_deposits::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(super::uid(&ctx, &auth).await?),
        bank_id: Set(params.bank_id),
        name: Set(params.name),
        fd_type: Set(params.fd_type.unwrap_or_else(|| "regular".to_string())),
        principal_amount: Set(principal),
        interest_rate: Set(rate),
        tenure_months: Set(tenure),
        start_date: Set(params.start_date),
        maturity_date: Set(params.maturity_date),
        compounding_frequency: Set(
            params
                .compounding_frequency
                .unwrap_or_else(|| "quarterly".to_string()),
        ),
        current_value: Set(current_value),
        interest_earned: Set(current_value - principal),
        tax_deducted: Set(params.tax_deducted.unwrap_or(0.0)),
        is_auto_renew: Set(params.is_auto_renew.unwrap_or(false)),
        renewal_instructions: Set(params.renewal_instructions),
        nominee: Set(params.nominee),
        certificate_number: Set(params.certificate_number),
        status: Set(params.status.unwrap_or_else(|| "active".to_string())),
        sync_status: Set("pending".to_string()),
        last_synced_at: Set(None),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now)),
        game_building_id: Set(None),
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
    Json(params): Json<UpdateFixedDepositRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = fixed_deposits::Entity::find_by_id(&id)
        .filter(fixed_deposits::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };
    let mut active = existing.into_active_model();
    if let Some(v) = params.bank_id {
        active.bank_id = Set(Some(v));
    }
    if let Some(v) = params.name {
        active.name = Set(v);
    }
    if let Some(v) = params.fd_type {
        active.fd_type = Set(v);
    }
    if let Some(v) = params.principal_amount {
        active.principal_amount = Set(v);
    }
    if let Some(v) = params.interest_rate {
        active.interest_rate = Set(v);
    }
    if let Some(v) = params.tenure_months {
        active.tenure_months = Set(v);
    }
    if let Some(v) = params.start_date {
        active.start_date = Set(v);
    }
    if let Some(v) = params.maturity_date {
        active.maturity_date = Set(v);
    }
    if let Some(v) = params.compounding_frequency {
        active.compounding_frequency = Set(v);
    }
    if let Some(v) = params.tax_deducted {
        active.tax_deducted = Set(v);
    }
    if let Some(v) = params.is_auto_renew {
        active.is_auto_renew = Set(v);
    }
    if let Some(v) = params.renewal_instructions {
        active.renewal_instructions = Set(Some(v));
    }
    if let Some(v) = params.nominee {
        active.nominee = Set(Some(v));
    }
    if let Some(v) = params.certificate_number {
        active.certificate_number = Set(Some(v));
    }
    if let Some(v) = params.status {
        active.status = Set(v);
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
    let existing = fixed_deposits::Entity::find_by_id(&id)
        .filter(fixed_deposits::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    if existing.is_none() {
        return not_found();
    }
    fixed_deposits::Entity::delete_by_id(&id)
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
    let rows = fixed_deposits::Entity::find()
        .filter(fixed_deposits::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .all(&*db)
        .await?;
    let s = FixedDepositSummary {
        total_invested: rows.iter().map(|r| r.principal_amount).sum(),
        total_value: rows.iter().map(|r| r.current_value).sum(),
        total_interest: rows.iter().map(|r| r.interest_earned).sum(),
        active_count: rows.iter().filter(|r| r.status == "active").count() as i64,
    };
    format::json(s)
}