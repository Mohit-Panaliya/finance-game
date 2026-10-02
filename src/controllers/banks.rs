use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};

use crate::models::banks::{self, BankSummary, CreateBankRequest, UpdateBankRequest};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/banks")
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

    let mut query =
        banks::Entity::find().filter(banks::Column::UserId.eq(super::uid(&ctx, &auth).await?));
    if !search.is_empty() {
        query = query.filter(
            Condition::any()
                .add(banks::Column::Name.contains(search))
                .add(banks::Column::BankType.contains(search))
                .add(banks::Column::AccountNumber.contains(search)),
        );
    }
    query = query.order_by_desc(banks::Column::UpdatedAt);

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
    let item = banks::Entity::find_by_id(id)
        .filter(banks::Column::UserId.eq(super::uid(&ctx, &auth).await?))
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
    Json(params): Json<CreateBankRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let now = Utc::now().to_rfc3339();
    let active = banks::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(super::uid(&ctx, &auth).await?),
        name: Set(params.name),
        bank_type: Set(params.bank_type),
        account_number: Set(params.account_number),
        ifsc_code: Set(params.ifsc_code),
        branch: Set(params.branch),
        current_balance: Set(params.current_balance.unwrap_or(0.0)),
        currency: Set(params.currency.unwrap_or_else(|| "INR".to_string())),
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
    Json(params): Json<UpdateBankRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = banks::Entity::find_by_id(id)
        .filter(banks::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };
    let mut active = existing.into_active_model();
    if let Some(v) = params.name {
        active.name = Set(v);
    }
    if let Some(v) = params.bank_type {
        active.bank_type = Set(v);
    }
    if let Some(v) = params.account_number {
        active.account_number = Set(v);
    }
    if let Some(v) = params.ifsc_code {
        active.ifsc_code = Set(Some(v));
    }
    if let Some(v) = params.branch {
        active.branch = Set(Some(v));
    }
    if let Some(v) = params.current_balance {
        active.current_balance = Set(v);
    }
    if let Some(v) = params.currency {
        active.currency = Set(v);
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
    let existing = banks::Entity::find_by_id(&id)
        .filter(banks::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    if existing.is_none() {
        return not_found();
    }
    banks::Entity::delete_by_id(&id)
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
    let rows = banks::Entity::find()
        .filter(banks::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .all(&*db)
        .await?;
    let total_balance: f64 = rows.iter().map(|r| r.current_balance).sum();
    let s = BankSummary {
        total_balance,
        count: rows.len() as i64,
    };
    format::json(s)
}
