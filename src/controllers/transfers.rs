use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};

use crate::ledger;
use crate::models::banks;
use crate::models::transfers::{self, CreateTransferRequest, TransferResponse};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/transfers")
        .add("/", get(list).post(create))
        .add("/{id}", get(show).delete(remove))
}

#[debug_handler]
async fn list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<crate::db_backend::FinanceDb>()
        .unwrap();
    let uid = super::uid(&ctx, &auth).await?;
    let items = transfers::Entity::find()
        .filter(transfers::Column::UserId.eq(uid))
        .order_by_desc(transfers::Column::CreatedAt)
        .all(&*db)
        .await?;
    format::json(items)
}

#[debug_handler]
async fn show(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<crate::db_backend::FinanceDb>()
        .unwrap();
    let uid = super::uid(&ctx, &auth).await?;
    let item = transfers::Entity::find_by_id(id)
        .filter(transfers::Column::UserId.eq(uid))
        .one(&*db)
        .await?;
    match item {
        Some(item) => format::json(TransferResponse::from(item)),
        None => not_found(),
    }
}

#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateTransferRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<crate::db_backend::FinanceDb>()
        .unwrap();
    let uid = super::uid(&ctx, &auth).await?;

    if params.from_bank_id == params.to_bank_id {
        return Err(Error::BadRequest(
            "source and destination accounts must be different".to_string(),
        ));
    }
    if !params.amount.is_finite() || params.amount <= 0.0 {
        return Err(Error::BadRequest(
            "amount must be greater than zero".to_string(),
        ));
    }

    // Verify both banks belong to the user
    let from_bank = banks::Entity::find_by_id(&params.from_bank_id)
        .filter(banks::Column::UserId.eq(uid))
        .one(&*db)
        .await?;
    if from_bank.is_none() {
        return not_found();
    }
    let to_bank = banks::Entity::find_by_id(&params.to_bank_id)
        .filter(banks::Column::UserId.eq(uid))
        .one(&*db)
        .await?;
    if to_bank.is_none() {
        return not_found();
    }

    let txn = db
        .begin()
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    // Debit source, credit destination
    ledger::adjust_bank(&txn, uid, Some(&params.from_bank_id), -params.amount)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    ledger::adjust_bank(&txn, uid, Some(&params.to_bank_id), params.amount)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    // Record the transfer
    let active = params.into_active(uid);
    let item = active
        .insert(&txn)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    if let Err(e) = txn.commit().await {
        return Err(Error::string(&e.to_string()));
    }

    format::json(TransferResponse::from(item))
}

#[debug_handler]
async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<crate::db_backend::FinanceDb>()
        .unwrap();
    let uid = super::uid(&ctx, &auth).await?;
    let existing = transfers::Entity::find_by_id(&id)
        .filter(transfers::Column::UserId.eq(uid))
        .one(&*db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };

    let txn = db
        .begin()
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    // Reverse the balance changes
    ledger::adjust_bank(&txn, uid, Some(&existing.from_bank_id), existing.amount)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    ledger::adjust_bank(&txn, uid, Some(&existing.to_bank_id), -existing.amount)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    transfers::Entity::delete_by_id(&id)
        .exec(&txn)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    if let Err(e) = txn.commit().await {
        return Err(Error::string(&e.to_string()));
    }

    format::json(serde_json::json!({ "ok": true }))
}
