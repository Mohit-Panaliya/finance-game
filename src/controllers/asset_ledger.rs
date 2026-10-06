//! Asset depreciation and the buy list.
//!
//! Endpoints:
//!   GET    /api/assets/{id}/depreciation   schedule + current book value
//!   POST   /api/assets/{id}/depreciation   regenerate the schedule
//!   GET    /api/buy-list                   items, optional `?status=`
//!   POST   /api/buy-list                   add an intended purchase
//!   GET    /api/buy-list/{id}              one item
//!   PUT    /api/buy-list/{id}              edit
//!   DELETE /api/buy-list/{id}              remove
//!   POST   /api/buy-list/{id}/convert      buy it: expense + asset, debits the bank
//!
//! The convert endpoint is the interesting one. A wanted laptop is not an asset until
//! it is paid for, so the item sits on the buy list touching no balance. On conversion
//! it becomes both an expense (money left the bank) and a depreciating asset (so its
//! worth falls over time) — the honest double entry, rather than pretending the money
//! vanished or pretending nothing was bought.

use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::json;

use crate::depreciation_service::{self as svc, DepreciationView};
use crate::models::{asset_buy_list, assets, expenses};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/buy-list")
        .add("/", get(list).post(create))
        .add("/{id}", get(show).put(update).delete(remove))
        .add("/{id}/convert", post(convert))
}

/// Mounted from `assets::routes()` so depreciation lives under the asset it belongs to.
pub fn depreciation_routes() -> Routes {
    Routes::new().prefix("/api/assets").add(
        "/{id}/depreciation",
        get(show_depreciation).post(rebuild_depreciation),
    )
}

/// `YYYY-MM-DD`, which is what every date column in this schema stores.
fn today() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

// ── depreciation ────────────────────────────────────────────────────────────

/// Load an asset the caller owns, or `None`. Ownership is checked on every path so a
/// guessed UUID cannot read someone else's schedule.
async fn owned_asset(
    db: &impl ConnectionTrait,
    user_id: i64,
    id: &str,
) -> Result<Option<assets::Model>, DbErr> {
    Ok(assets::Entity::find_by_id(id)
        .filter(assets::Column::UserId.eq(user_id))
        .one(db)
        .await?)
}

#[debug_handler]
async fn show_depreciation(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let user_id = super::uid(&ctx, &auth).await?;
    let Some(asset) = owned_asset(&*db, user_id, &id).await? else {
        return not_found();
    };
    let view: DepreciationView = svc::view(&*db, user_id, &asset, &today()).await?;
    format::json(view)
}

#[derive(Debug, Deserialize)]
pub struct RebuildRequest {
    /// Overrides the asset's stored method/life/salvage for this regeneration.
    pub depreciation_method: Option<String>,
    pub useful_life_months: Option<i64>,
    pub salvage_value: Option<f64>,
    pub depreciation_start_date: Option<String>,
}

#[debug_handler]
async fn rebuild_depreciation(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<RebuildRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let user_id = super::uid(&ctx, &auth).await?;
    let Some(asset) = owned_asset(&*db, user_id, &id).await? else {
        return not_found();
    };

    let mut am = asset.clone().into_active_model();
    if let Some(v) = params.depreciation_method {
        am.depreciation_method = Set(v);
    }
    if let Some(v) = params.useful_life_months {
        am.useful_life_months = Set(Some(v));
    }
    if let Some(v) = params.salvage_value {
        // Salvage above cost would make the schedule unreachable, so clamp at the edge.
        am.salvage_value = Set(v.min(asset.purchase_price.max(0.0)));
    }
    if let Some(v) = params.depreciation_start_date {
        am.depreciation_start_date = Set(Some(v));
    }
    am.updated_at = Set(Some(Utc::now().to_rfc3339()));
    let saved = am.update(&*db).await?;

    // A bad combination (no useful life, salvage above cost) is reported, not hidden.
    svc::rebuild(&*db, user_id, &saved, &today())
        .await
        .map_err(|e| Error::string(&e))?;
    let view: DepreciationView = svc::view(&*db, user_id, &saved, &today()).await?;
    format::json(view)
}

// ── buy list ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

#[debug_handler]
async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<ListQuery>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let user_id = super::uid(&ctx, &auth).await?;
    let mut q = asset_buy_list::Entity::find().filter(asset_buy_list::Column::UserId.eq(user_id));
    if let Some(status) = params.status.filter(|s| !s.is_empty()) {
        q = q.filter(asset_buy_list::Column::Status.eq(status));
    }
    let mut rows = q.all(&*db).await?;

    // Priority is a word, so it is ordered in Rust where the mapping is explicit; within
    // one priority, soonest-first then cheapest is the order a buyer actually works in.
    rows.sort_by(|a, b| {
        asset_buy_list::priority_rank(&a.priority)
            .cmp(&asset_buy_list::priority_rank(&b.priority))
            .then_with(|| {
                a.target_date
                    .as_deref()
                    .unwrap_or("9999")
                    .cmp(b.target_date.as_deref().unwrap_or("9999"))
            })
            .then_with(|| a.estimated_cost.total_cmp(&b.estimated_cost))
    });

    let planned: Vec<&asset_buy_list::Model> = rows
        .iter()
        .filter(|r| r.status == asset_buy_list::PLANNED)
        .collect();
    let total: f64 = planned.iter().map(|r| r.estimated_cost).sum();

    format::json(json!({
        "items": rows,
        "planned_count": planned.len(),
        "planned_total": (total * 100.0).round() / 100.0,
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
    let user_id = super::uid(&ctx, &auth).await?;
    match find_item(&*db, user_id, &id).await? {
        Some(v) => format::json(v),
        None => not_found(),
    }
}

async fn find_item(
    db: &impl ConnectionTrait,
    user_id: i64,
    id: &str,
) -> Result<Option<asset_buy_list::Model>, DbErr> {
    Ok(asset_buy_list::Entity::find_by_id(id)
        .filter(asset_buy_list::Column::UserId.eq(user_id))
        .one(db)
        .await?)
}

#[derive(Debug, Deserialize)]
pub struct CreateBuyListRequest {
    pub title: String,
    pub description: Option<String>,
    pub asset_type: Option<String>,
    pub estimated_cost: Option<f64>,
    pub priority: Option<String>,
    pub target_date: Option<String>,
    pub url: Option<String>,
    pub shop: Option<String>,
    /// Which account would pay for it. Recorded now so conversion does not have to ask.
    pub bank_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBuyListRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub asset_type: Option<String>,
    pub estimated_cost: Option<f64>,
    pub priority: Option<String>,
    pub target_date: Option<String>,
    pub url: Option<String>,
    pub shop: Option<String>,
    pub status: Option<String>,
    /// Unlink uses an empty string, matching the income/expense convention.
    pub bank_id: Option<String>,
}

#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateBuyListRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let user_id = super::uid(&ctx, &auth).await?;
    let now = Utc::now().to_rfc3339();
    let saved = asset_buy_list::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(user_id),
        title: Set(params.title),
        description: Set(params.description),
        asset_type: Set(params.asset_type.unwrap_or_else(|| "Other".into())),
        estimated_cost: Set(params.estimated_cost.unwrap_or(0.0).max(0.0)),
        priority: Set(params.priority.unwrap_or_else(|| "medium".into())),
        target_date: Set(params.target_date),
        url: Set(params.url),
        shop: Set(params.shop),
        status: Set(asset_buy_list::PLANNED.into()),
        bank_id: Set(params.bank_id),
        converted_asset_id: Set(None),
        converted_at: Set(None),
        sync_status: Set("pending".into()),
        last_synced_at: Set(None),
        created_at: Set(now.clone()),
        updated_at: Set(Some(now)),
    }
    .insert(&*db)
    .await?;
    format::json(saved)
}

#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<UpdateBuyListRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let user_id = super::uid(&ctx, &auth).await?;
    let Some(existing) = find_item(&*db, user_id, &id).await? else {
        return not_found();
    };
    // A bought item is a record of what happened. Editing the wish afterwards would
    // rewrite history, so everything but `status` is frozen once it is converted.
    if existing.status != asset_buy_list::PLANNED {
        return format::json(existing);
    }

    let mut am = existing.into_active_model();
    if let Some(v) = params.title {
        am.title = Set(v);
    }
    if let Some(v) = params.description {
        am.description = Set(Some(v));
    }
    if let Some(v) = params.asset_type {
        am.asset_type = Set(v);
    }
    if let Some(v) = params.estimated_cost {
        am.estimated_cost = Set(v.max(0.0));
    }
    if let Some(v) = params.priority {
        am.priority = Set(v);
    }
    if let Some(v) = params.target_date {
        am.target_date = Set(Some(v));
    }
    if let Some(v) = params.url {
        am.url = Set(Some(v));
    }
    if let Some(v) = params.shop {
        am.shop = Set(Some(v));
    }
    if let Some(v) = params.bank_id {
        am.bank_id = Set((!v.is_empty()).then_some(v));
    }
    if let Some(v) = params
        .status
        .filter(|s| matches!(s.as_str(), "planned" | "purchased" | "dropped"))
    {
        am.status = Set(v);
    }
    am.sync_status = Set("pending".into());
    am.updated_at = Set(Some(Utc::now().to_rfc3339()));
    let saved = am.update(&*db).await?;
    format::json(saved)
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
    let user_id = super::uid(&ctx, &auth).await?;
    let Some(existing) = find_item(&*db, user_id, &id).await? else {
        return not_found();
    };
    asset_buy_list::Entity::delete_by_id(&existing.id)
        .exec(&*db)
        .await?;
    format::json(json!({ "ok": true }))
}

#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    /// What was actually paid, which is rarely the estimate.
    pub actual_cost: Option<f64>,
    pub purchase_date: Option<String>,
    /// Override the account the money left.
    pub bank_id: Option<String>,
    pub asset_name: Option<String>,
    pub depreciation_method: Option<String>,
    pub useful_life_months: Option<i64>,
    pub salvage_value: Option<f64>,
    pub category: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
}

#[debug_handler]
async fn convert(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<ConvertRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let user_id = super::uid(&ctx, &auth).await?;
    let Some(item) = find_item(&*db, user_id, &id).await? else {
        return not_found();
    };
    // Converting twice would debit the bank twice for one purchase.
    if item.status == asset_buy_list::PURCHASED {
        return format::json(json!({ "error": "already converted", "item": item }));
    }

    let paid = params.actual_cost.unwrap_or(item.estimated_cost).max(0.0);
    let purchased_on = params.purchase_date.unwrap_or_else(today);
    let bank_id = params.bank_id.or_else(|| item.bank_id.clone());
    let name = params.asset_name.unwrap_or_else(|| item.title.clone());
    let category = params
        .category
        .clone()
        .unwrap_or_else(|| item.asset_type.clone());
    let description = params
        .description
        .clone()
        .or_else(|| item.description.clone());
    let now = Utc::now().to_rfc3339();

    // 1. the expense — the money actually left the account
    if paid > 0.0 {
        expenses::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            title: Set(name.clone()),
            description: Set(description.clone()),
            amount: Set(paid),
            currency: Set("USD".into()),
            expense_type: Set("one_time".into()),
            category: Set(category.clone()),
            expense_date: Set(purchased_on.clone()),
            is_recurring: Set(false),
            recurrence: Set(None),
            recurrence_end_date: Set(None),
            payment_method: Set(item.shop.clone().unwrap_or_else(|| "buy_list".into())),
            bank_id: Set(bank_id.clone()),
            credit_card_id: Set(None),
            tags: Set(Some("buy_list".into())),
            receipt_url: Set(item.url.clone()),
            location: Set(params.location.clone()),
            is_fixed: Set(false),
            priority: Set(item.priority.clone()),
            sync_status: Set("pending".into()),
            last_synced_at: Set(None),
            created_at: Set(Some(now.clone())),
            updated_at: Set(Some(now.clone())),
        }
        .insert(&*db)
        .await?;
    }

    // 2. the asset — so its worth falls over time instead of staying frozen at cost
    let asset = assets::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(user_id),
        name: Set(name),
        asset_type: Set(item.asset_type.clone()),
        category: Set(category),
        purchase_price: Set(paid),
        current_value: Set(paid),
        purchase_date: Set(purchased_on.clone()),
        location: Set(params.location),
        description: Set(description),
        documents: Set(None),
        roi_percentage: Set(None),
        annual_income: Set(0.0),
        depreciation_rate: Set(None),
        is_liquid: Set(false),
        risk_level: Set("moderate".into()),
        sync_status: Set("pending".into()),
        last_synced_at: Set(None),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now.clone())),
        bank_id: Set(bank_id.clone()),
        depreciation_method: Set(params
            .depreciation_method
            .clone()
            .unwrap_or_else(|| "straight_line".into())),
        useful_life_months: Set(params.useful_life_months),
        salvage_value: Set(params.salvage_value.unwrap_or(0.0).min(paid.max(0.0))),
        depreciation_start_date: Set(Some(purchased_on)),
    }
    .insert(&*db)
    .await?;

    // 3. move the money, through the ledger helper so the bank total can never drift
    //    from the expense recorded above
    crate::ledger::on_purchase_created(&*db, user_id, bank_id.as_ref(), paid)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    // 4. close out the list item
    let mut am = item.into_active_model();
    am.status = Set(asset_buy_list::PURCHASED.into());
    am.converted_asset_id = Set(Some(asset.id.clone()));
    am.converted_at = Set(Some(now));
    am.sync_status = Set("pending".into());
    am.updated_at = Set(Some(Utc::now().to_rfc3339()));
    let saved = am.update(&*db).await?;

    let depreciation: DepreciationView = svc::view(&*db, user_id, &asset, &today()).await?;
    format::json(json!({
        "item": saved,
        "asset": asset,
        "expense_recorded": paid > 0.0,
        "amount": paid,
        "depreciation": depreciation,
    }))
}
