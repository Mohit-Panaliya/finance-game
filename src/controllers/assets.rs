use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

use crate::models::assets::{self, AssetSummary, CreateAssetRequest, UpdateAssetRequest};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/assets")
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

    let mut query = assets::Entity::find().filter(assets::Column::UserId.eq(super::uid(&ctx, &auth).await?));
    if !search.is_empty() {
        query = query.filter(
            Condition::any()
                .add(assets::Column::Name.contains(search))
                .add(assets::Column::AssetType.contains(search))
                .add(assets::Column::Category.contains(search)),
        );
    }
    query = query.order_by_desc(assets::Column::UpdatedAt);

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
    let item = assets::Entity::find_by_id(&id)
        .filter(assets::Column::UserId.eq(super::uid(&ctx, &auth).await?))
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
    Json(params): Json<CreateAssetRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let now = Utc::now().to_rfc3339();
    let active = assets::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(super::uid(&ctx, &auth).await?),
        name: Set(params.name),
        asset_type: Set(params.asset_type),
        category: Set(params.category),
        purchase_price: Set(params.purchase_price.unwrap_or(0.0)),
        current_value: Set(params.current_value.unwrap_or(0.0)),
        purchase_date: Set(params.purchase_date),
        location: Set(params.location),
        description: Set(params.description),
        documents: Set(params.documents),
        roi_percentage: Set(params.roi_percentage),
        annual_income: Set(params.annual_income.unwrap_or(0.0)),
        depreciation_rate: Set(params.depreciation_rate),
        is_liquid: Set(params.is_liquid.unwrap_or(false)),
        risk_level: Set(params.risk_level.unwrap_or_else(|| "moderate".to_string())),
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
    Json(params): Json<UpdateAssetRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = assets::Entity::find_by_id(&id)
        .filter(assets::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };
    let mut active = existing.into_active_model();
    if let Some(v) = params.name {
        active.name = Set(v);
    }
    if let Some(v) = params.asset_type {
        active.asset_type = Set(v);
    }
    if let Some(v) = params.category {
        active.category = Set(v);
    }
    if let Some(v) = params.purchase_price {
        active.purchase_price = Set(v);
    }
    if let Some(v) = params.current_value {
        active.current_value = Set(v);
    }
    if let Some(v) = params.purchase_date {
        active.purchase_date = Set(v);
    }
    if let Some(v) = params.location {
        active.location = Set(Some(v));
    }
    if let Some(v) = params.description {
        active.description = Set(Some(v));
    }
    if let Some(v) = params.documents {
        active.documents = Set(Some(v));
    }
    if let Some(v) = params.roi_percentage {
        active.roi_percentage = Set(Some(v));
    }
    if let Some(v) = params.annual_income {
        active.annual_income = Set(v);
    }
    if let Some(v) = params.depreciation_rate {
        active.depreciation_rate = Set(Some(v));
    }
    if let Some(v) = params.is_liquid {
        active.is_liquid = Set(v);
    }
    if let Some(v) = params.risk_level {
        active.risk_level = Set(v);
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
    let existing = assets::Entity::find_by_id(&id)
        .filter(assets::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    if existing.is_none() {
        return not_found();
    }
    assets::Entity::delete_by_id(&id)
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
    let rows = assets::Entity::find()
        .filter(assets::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .all(&*db)
        .await?;
    let total_value: f64 = rows.iter().map(|r| r.current_value).sum();
    let total_invested: f64 = rows.iter().map(|r| r.purchase_price).sum();
    let s = AssetSummary {
        total_value,
        total_invested,
        total_gain_loss: total_value - total_invested,
        count: rows.len() as i64,
    };
    format::json(s)
}