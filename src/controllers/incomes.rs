use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

use crate::models::incomes::{
    self, CreateIncomeRequest, IncomeSummary, UpdateIncomeRequest,
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/incomes")
        .add("/summary", get(summary))
        .add("/", get(list).post(create))
        .add("/{id}", get(show).put(update).delete(remove))
}


fn annualize(recurrence: &str, amount: f64) -> f64 {
    match recurrence {
        "weekly" => amount * 52.0,
        "biweekly" => amount * 26.0,
        "monthly" => amount * 12.0,
        "quarterly" => amount * 4.0,
        "yearly" | "annual" => amount,
        "one_time" => amount,
        _ => amount,
    }
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

    let mut query = incomes::Entity::find().filter(incomes::Column::UserId.eq(super::uid(&ctx, &auth).await?));
    if !search.is_empty() {
        query = query.filter(
            Condition::any()
                .add(incomes::Column::Title.contains(search))
                .add(incomes::Column::Source.contains(search))
                .add(incomes::Column::IncomeType.contains(search)),
        );
    }
    query = query.order_by_desc(incomes::Column::IncomeDate);

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
    let item = incomes::Entity::find_by_id(&id)
        .filter(incomes::Column::UserId.eq(super::uid(&ctx, &auth).await?))
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
    Json(params): Json<CreateIncomeRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let now = Utc::now().to_rfc3339();
    let active = incomes::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(super::uid(&ctx, &auth).await?),
        title: Set(params.title),
        description: Set(params.description),
        amount: Set(params.amount),
        currency: Set(params.currency.unwrap_or_else(|| "INR".to_string())),
        income_type: Set(params.income_type),
        source: Set(params.source),
        income_date: Set(params.income_date),
        is_recurring: Set(params.is_recurring.unwrap_or(false)),
        recurrence: Set(params.recurrence.clone()),
        frequency_multiplier: Set(
            params
                .frequency_multiplier
                .unwrap_or_else(|| match params.recurrence.as_deref() {
                    Some("weekly") => 52,
                    Some("biweekly") => 26,
                    Some("monthly") => 12,
                    Some("quarterly") => 4,
                    _ => 1,
                }),
        ),
        is_gross: Set(params.is_gross.unwrap_or(true)),
        tax_withheld: Set(params.tax_withheld.unwrap_or(0.0)),
        bank_id: Set(params.bank_id),
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
    Json(params): Json<UpdateIncomeRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = incomes::Entity::find_by_id(&id)
        .filter(incomes::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };
    let mut active = existing.into_active_model();
    if let Some(v) = params.title {
        active.title = Set(v);
    }
    if let Some(v) = params.description {
        active.description = Set(Some(v));
    }
    if let Some(v) = params.amount {
        active.amount = Set(v);
    }
    if let Some(v) = params.currency {
        active.currency = Set(v);
    }
    if let Some(v) = params.income_type {
        active.income_type = Set(v);
    }
    if let Some(v) = params.source {
        active.source = Set(v);
    }
    if let Some(v) = params.income_date {
        active.income_date = Set(v);
    }
    if let Some(v) = params.is_recurring {
        active.is_recurring = Set(v);
    }
    if let Some(v) = params.recurrence {
        active.recurrence = Set(Some(v));
    }
    if let Some(v) = params.frequency_multiplier {
        active.frequency_multiplier = Set(v);
    }
    if let Some(v) = params.is_gross {
        active.is_gross = Set(v);
    }
    if let Some(v) = params.tax_withheld {
        active.tax_withheld = Set(v);
    }
    if let Some(v) = params.bank_id {
        active.bank_id = Set(Some(v));
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
    let existing = incomes::Entity::find_by_id(&id)
        .filter(incomes::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    if existing.is_none() {
        return not_found();
    }
    incomes::Entity::delete_by_id(&id)
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
    let rows = incomes::Entity::find()
        .filter(incomes::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .all(&*db)
        .await?;

    let year = Utc::now().format("%Y").to_string();
    let mut yearly = 0.0;
    let mut monthly = 0.0;
    let mut received_this_year = 0.0;
    let mut by_type_map: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();

    for r in &rows {
        *by_type_map.entry(r.income_type.clone()).or_insert(0.0) += r.amount;
        if r.income_date.starts_with(&year) {
            received_this_year += r.amount;
        }
        if r.is_recurring {
            let rec = r.recurrence.as_deref().unwrap_or("monthly");
            let annualized = if r.frequency_multiplier > 1 {
                r.amount * r.frequency_multiplier as f64
            } else {
                annualize(rec, r.amount)
            };
            yearly += annualized;
            monthly += annualized / 12.0;
        }
    }

    let by_type = by_type_map
        .into_iter()
        .map(|(income_type, total)| crate::models::incomes::IncomeTypeTotal { income_type, total })
        .collect();

    let s = IncomeSummary {
        yearly_income: yearly,
        monthly_income: monthly,
        received_this_year,
        by_type,
    };
    format::json(s)
}