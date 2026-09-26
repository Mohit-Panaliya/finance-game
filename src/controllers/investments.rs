use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

use crate::models::investments::{
    self, CreateInvestmentRequest, InvestmentSummary, UpdateInvestmentRequest,
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/investments")
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
        investments::Entity::find().filter(investments::Column::UserId.eq(super::uid(&ctx, &auth).await?));
    if !search.is_empty() {
        query = query.filter(
            Condition::any()
                .add(investments::Column::Name.contains(search))
                .add(investments::Column::InvestmentType.contains(search))
                .add(investments::Column::Instrument.contains(search)),
        );
    }
    query = query.order_by_desc(investments::Column::UpdatedAt);

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
    let item = investments::Entity::find_by_id(&id)
        .filter(investments::Column::UserId.eq(super::uid(&ctx, &auth).await?))
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
    Json(params): Json<CreateInvestmentRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let now = Utc::now().to_rfc3339();
    let active = investments::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(super::uid(&ctx, &auth).await?),
        name: Set(params.name),
        investment_type: Set(params.investment_type),
        instrument: Set(params.instrument),
        symbol: Set(params.symbol),
        invested_amount: Set(params.invested_amount),
        current_value: Set(params.current_value.unwrap_or(0.0)),
        units: Set(params.units),
        unit_price: Set(params.unit_price),
        purchase_date: Set(params.purchase_date),
        purchase_price: Set(params.purchase_price),
        broker_platform: Set(params.broker_platform),
        account_ref: Set(params.account_ref),
        expected_return: Set(params.expected_return),
        actual_return: Set(params.actual_return),
        annualized_return: Set(params.annualized_return),
        dividend_yield: Set(params.dividend_yield),
        risk_level: Set(params.risk_level.unwrap_or_else(|| "moderate".to_string())),
        is_liquid: Set(params.is_liquid.unwrap_or(true)),
        lock_in_until: Set(params.lock_in_until),
        tax_saving: Set(params.tax_saving.unwrap_or(false)),
        tags: Set(params.tags),
        notes: Set(params.notes),
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
    Json(params): Json<UpdateInvestmentRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = investments::Entity::find_by_id(&id)
        .filter(investments::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };
    let mut active = existing.into_active_model();
    if let Some(v) = params.name {
        active.name = Set(v);
    }
    if let Some(v) = params.investment_type {
        active.investment_type = Set(v);
    }
    if let Some(v) = params.instrument {
        active.instrument = Set(v);
    }
    if let Some(v) = params.symbol {
        active.symbol = Set(Some(v));
    }
    if let Some(v) = params.invested_amount {
        active.invested_amount = Set(v);
    }
    if let Some(v) = params.current_value {
        active.current_value = Set(v);
    }
    if let Some(v) = params.units {
        active.units = Set(Some(v));
    }
    if let Some(v) = params.unit_price {
        active.unit_price = Set(Some(v));
    }
    if let Some(v) = params.purchase_date {
        active.purchase_date = Set(v);
    }
    if let Some(v) = params.purchase_price {
        active.purchase_price = Set(Some(v));
    }
    if let Some(v) = params.broker_platform {
        active.broker_platform = Set(Some(v));
    }
    if let Some(v) = params.account_ref {
        active.account_ref = Set(Some(v));
    }
    if let Some(v) = params.expected_return {
        active.expected_return = Set(Some(v));
    }
    if let Some(v) = params.actual_return {
        active.actual_return = Set(Some(v));
    }
    if let Some(v) = params.annualized_return {
        active.annualized_return = Set(Some(v));
    }
    if let Some(v) = params.dividend_yield {
        active.dividend_yield = Set(Some(v));
    }
    if let Some(v) = params.risk_level {
        active.risk_level = Set(v);
    }
    if let Some(v) = params.is_liquid {
        active.is_liquid = Set(v);
    }
    if let Some(v) = params.lock_in_until {
        active.lock_in_until = Set(Some(v));
    }
    if let Some(v) = params.tax_saving {
        active.tax_saving = Set(v);
    }
    if let Some(v) = params.tags {
        active.tags = Set(Some(v));
    }
    if let Some(v) = params.notes {
        active.notes = Set(Some(v));
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
    let existing = investments::Entity::find_by_id(&id)
        .filter(investments::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    if existing.is_none() {
        return not_found();
    }
    investments::Entity::delete_by_id(&id)
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
    let rows = investments::Entity::find()
        .filter(investments::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .all(&*db)
        .await?;
    let total_invested: f64 = rows.iter().map(|r| r.invested_amount).sum();
    let total_value: f64 = rows.iter().map(|r| r.current_value).sum();
    let s = InvestmentSummary {
        total_invested,
        total_value,
        total_gain_loss: total_value - total_invested,
        count: rows.len() as i64,
    };
    format::json(s)
}