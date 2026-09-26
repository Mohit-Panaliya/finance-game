use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

use crate::models::expenses::{
    self, CreateExpenseRequest, ExpenseCategoryTotal, ExpenseMonthTotal, ExpenseSummary,
    UpdateExpenseRequest,
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/expenses")
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

    let mut query = expenses::Entity::find().filter(expenses::Column::UserId.eq(super::uid(&ctx, &auth).await?));
    if !search.is_empty() {
        query = query.filter(
            Condition::any()
                .add(expenses::Column::Title.contains(search))
                .add(expenses::Column::Category.contains(search)),
        );
    }
    query = query.order_by_desc(expenses::Column::ExpenseDate);

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
    let item = expenses::Entity::find_by_id(&id)
        .filter(expenses::Column::UserId.eq(super::uid(&ctx, &auth).await?))
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
    Json(params): Json<CreateExpenseRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let now = Utc::now().to_rfc3339();
    let active = expenses::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        user_id: Set(super::uid(&ctx, &auth).await?),
        title: Set(params.title),
        description: Set(params.description),
        amount: Set(params.amount),
        currency: Set(params.currency.unwrap_or_else(|| "INR".to_string())),
        expense_type: Set(params.expense_type),
        category: Set(params.category),
        expense_date: Set(params.expense_date),
        is_recurring: Set(params.is_recurring.unwrap_or(false)),
        recurrence: Set(params.recurrence),
        recurrence_end_date: Set(params.recurrence_end_date),
        payment_method: Set(
            params
                .payment_method
                .unwrap_or_else(|| "cash".to_string()),
        ),
        bank_id: Set(params.bank_id),
        credit_card_id: Set(params.credit_card_id),
        tags: Set(params.tags),
        receipt_url: Set(params.receipt_url),
        location: Set(params.location),
        is_fixed: Set(params.is_fixed.unwrap_or(false)),
        priority: Set(params.priority.unwrap_or_else(|| "medium".to_string())),
        sync_status: Set("pending".to_string()),
        last_synced_at: Set(None),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now)),
        game_troop_id: Set(None),
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
    Json(params): Json<UpdateExpenseRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let existing = expenses::Entity::find_by_id(&id)
        .filter(expenses::Column::UserId.eq(super::uid(&ctx, &auth).await?))
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
    if let Some(v) = params.expense_type {
        active.expense_type = Set(v);
    }
    if let Some(v) = params.category {
        active.category = Set(v);
    }
    if let Some(v) = params.expense_date {
        active.expense_date = Set(v);
    }
    if let Some(v) = params.is_recurring {
        active.is_recurring = Set(v);
    }
    if let Some(v) = params.recurrence {
        active.recurrence = Set(Some(v));
    }
    if let Some(v) = params.recurrence_end_date {
        active.recurrence_end_date = Set(Some(v));
    }
    if let Some(v) = params.payment_method {
        active.payment_method = Set(v);
    }
    if let Some(v) = params.bank_id {
        active.bank_id = Set(Some(v));
    }
    if let Some(v) = params.credit_card_id {
        active.credit_card_id = Set(Some(v));
    }
    if let Some(v) = params.tags {
        active.tags = Set(Some(v));
    }
    if let Some(v) = params.receipt_url {
        active.receipt_url = Set(Some(v));
    }
    if let Some(v) = params.location {
        active.location = Set(Some(v));
    }
    if let Some(v) = params.is_fixed {
        active.is_fixed = Set(v);
    }
    if let Some(v) = params.priority {
        active.priority = Set(v);
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
    let existing = expenses::Entity::find_by_id(&id)
        .filter(expenses::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .one(&*db)
        .await?;
    if existing.is_none() {
        return not_found();
    }
    expenses::Entity::delete_by_id(&id)
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
    let rows = expenses::Entity::find()
        .filter(expenses::Column::UserId.eq(super::uid(&ctx, &auth).await?))
        .all(&*db)
        .await?;

    let mut total = 0.0;
    let mut fixed_total = 0.0;
    let mut variable_total = 0.0;
    let mut by_category: Vec<ExpenseCategoryTotal> = Vec::new();
    let mut by_month: Vec<ExpenseMonthTotal> = Vec::new();

    for r in &rows {
        total += r.amount;
        if r.is_fixed {
            fixed_total += r.amount;
        } else {
            variable_total += r.amount;
        }

        let cat = by_category
            .iter_mut()
            .find(|c| c.category == r.category);
        match cat {
            Some(c) => {
                c.total += r.amount;
                c.count += 1;
            }
            None => by_category.push(ExpenseCategoryTotal {
                category: r.category.clone(),
                total: r.amount,
                count: 1,
            }),
        }

        let month = r.expense_date.get(..7).unwrap_or("").to_string();
        if !month.is_empty() {
            let m = by_month.iter_mut().find(|m| m.month == month);
            match m {
                Some(m) => m.total += r.amount,
                None => by_month.push(ExpenseMonthTotal {
                    month,
                    total: r.amount,
                }),
            }
        }
    }

    by_category.sort_by(|a, b| b.total.partial_cmp(&a.total).unwrap());
    by_month.sort_by(|a, b| a.month.cmp(&b.month));

    let s = ExpenseSummary {
        total_amount: total,
        fixed_total,
        variable_total,
        by_category,
        by_month,
    };
    format::json(s)
}