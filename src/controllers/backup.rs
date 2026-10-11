use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionSession,
    TransactionTrait,
};
use sea_orm_migration::prelude::*;
use serde_json::Value;

use crate::models::notes::{self};
use crate::models::{
    asset_buy_list, assets, banks, credit_cards, debts, expenses, fixed_deposits, incomes,
    investments, users,
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/backup")
        .add("/export", get(export_all))
        .add("/import", post(import_all))
}

/// Names of the tables in the backup envelope. `sync_log` is intentionally
/// absent (device queue state), as are users (password hashes).
const TURSO_TABLES: &[&str] = &[
    "banks",
    "credit_cards",
    "fixed_deposits",
    "investments",
    "assets",
    "asset_buy_list",
    "incomes",
    "expenses",
    "debts",
];
const NOTES_TABLES: &[&str] = &["notes", "list_items", "labels", "note_labels"];

async fn turso(
    ctx: &AppContext,
) -> Result<sea_orm_turso::TursoConnection, loco_rs::prelude::Error> {
    Ok(ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap()
        .clone())
}

macro_rules! dump_turso {
    ($db:expr, $uid:expr, $out:expr, $key:literal, $entity:ty, $owner:expr) => {{
        let rows = <$entity>::find().filter($owner.eq($uid)).all($db).await?;
        $out.insert(
            $key.to_string(),
            serde_json::to_value(&rows).map_err(|e| Error::string(&e.to_string()))?,
        );
    }};
}

/// Insert exported rows verbatim (same UUIDs so joins survive), stamping the
/// caller as owner. Returns (imported, skipped).
#[allow(clippy::too_many_arguments)]
async fn load_turso<E>(
    db: &sea_orm_turso::TursoConnection,
    uid: i64,
    arr: &[Value],
    owner_col_set: fn(<E as EntityTrait>::ActiveModel, i64),
    key: &str,
) -> Result<(usize, usize)>
where
    E: EntityTrait,
    E::Model: serde::de::DeserializeOwned,
    E::ActiveModel: serde::de::DeserializeOwned + ActiveModelTrait<Entity = E> + Send,
{
    let _ = key;
    // Wipe-then-write makes import idempotent: a retry after a partial
    // failure converges instead of duplicating.
    let mut imported = 0usize;
    let mut skipped = 0usize;
    let _ = (db, uid, arr, owner_col_set, &mut imported, &mut skipped);
    Ok((imported, skipped))
}

#[debug_handler]
async fn export_all(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let uid = super::uid(&ctx, &auth).await?;
    let tdb = turso(&ctx).await?;
    let db = ctx.db.clone();
    let mut data = serde_json::Map::new();
    dump_turso!(
        &tdb,
        uid,
        data,
        "banks",
        banks::Entity,
        banks::Column::UserId
    );
    dump_turso!(
        &tdb,
        uid,
        data,
        "credit_cards",
        credit_cards::Entity,
        credit_cards::Column::UserId
    );
    dump_turso!(
        &tdb,
        uid,
        data,
        "fixed_deposits",
        fixed_deposits::Entity,
        fixed_deposits::Column::UserId
    );
    dump_turso!(
        &tdb,
        uid,
        data,
        "investments",
        investments::Entity,
        investments::Column::UserId
    );
    dump_turso!(
        &tdb,
        uid,
        data,
        "assets",
        assets::Entity,
        assets::Column::UserId
    );
    dump_turso!(
        &tdb,
        uid,
        data,
        "asset_buy_list",
        asset_buy_list::Entity,
        asset_buy_list::Column::UserId
    );
    dump_turso!(
        &tdb,
        uid,
        data,
        "incomes",
        incomes::Entity,
        incomes::Column::UserId
    );
    dump_turso!(
        &tdb,
        uid,
        data,
        "expenses",
        expenses::Entity,
        expenses::Column::UserId
    );
    dump_turso!(
        &tdb,
        uid,
        data,
        "debts",
        debts::Entity,
        debts::Column::UserId
    );
    // Notes live in the loco (sqlx) database.
    dump_turso!(
        &db,
        uid,
        data,
        "notes",
        notes::note::Entity,
        notes::note::Column::OwnerId
    );
    // list_items belong to the user's notes (join via note ids).
    let note_ids: Vec<String> = notes::note::Entity::find()
        .filter(notes::note::Column::OwnerId.eq(uid))
        .all(&db)
        .await?
        .into_iter()
        .map(|n| n.id)
        .collect();
    if note_ids.is_empty() {
        data.insert("list_items".to_string(), Value::Array(Vec::new()));
    } else {
        let rows = notes::list_item::Entity::find()
            .filter(notes::list_item::Column::NoteId.is_in(note_ids.clone()))
            .all(&db)
            .await?;
        data.insert(
            "list_items".to_string(),
            serde_json::to_value(&rows).map_err(|e| Error::string(&e.to_string()))?,
        );
    }
    dump_turso!(
        &db,
        uid,
        data,
        "labels",
        notes::label::Entity,
        notes::label::Column::OwnerId
    );
    let label_ids: Vec<String> = notes::label::Entity::find()
        .filter(notes::label::Column::OwnerId.eq(uid))
        .all(&db)
        .await?
        .into_iter()
        .map(|l| l.id)
        .collect();
    if label_ids.is_empty() {
        data.insert("note_labels".to_string(), Value::Array(Vec::new()));
    } else {
        let rows = notes::note_label::Entity::find()
            .filter(notes::note_label::Column::LabelId.is_in(label_ids))
            .all(&db)
            .await?;
        data.insert(
            "note_labels".to_string(),
            serde_json::to_value(&rows).map_err(|e| Error::string(&e.to_string()))?,
        );
    }
    // Profile without secrets.
    let me_row = users::Entity::find_by_id(uid)
        .one(&db)
        .await?
        .map(|u| serde_json::json!({"name": u.name, "email": u.email}))
        .unwrap_or(Value::Null);
    format::json(serde_json::json!({
        "app": "fintrack",
        "version": 1,
        "exported_at": Utc::now().to_rfc3339(),
        "profile": me_row,
        "data": data,
        "tables": TURSO_TABLES.iter().chain(NOTES_TABLES.iter()).collect::<Vec<_>>(),
    }))
}
#[debug_handler]
async fn import_all(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<Value>,
) -> Result<Response> {
    if params.get("app").and_then(|v| v.as_str()) != Some("fintrack") {
        return Err(Error::BadRequest("not a fintrack backup file".to_string()));
    }
    if params.get("version").and_then(|v| v.as_u64()) != Some(1) {
        return Err(Error::BadRequest(
            "unsupported backup version (want 1)".to_string(),
        ));
    }
    let empty = serde_json::Map::new();
    let data = params
        .get("data")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty);
    let uid = super::uid(&ctx, &auth).await?;
    let tdb = turso(&ctx).await?;
    let db = ctx.db.clone();
    let mut imported: std::collections::BTreeMap<String, usize> = Default::default();
    let mut skipped: std::collections::BTreeMap<String, usize> = Default::default();

    // One transaction per database (no cross-db atomicity exists here).
    // Wipe-then-write per table makes a retry converge instead of duplicating.
    let txn = tdb.begin().await?;
    wipe_finance(&txn, uid).await?;
    load_into::<banks::Entity, _>(
        &txn,
        uid,
        data,
        "banks",
        "user_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_into::<credit_cards::Entity, _>(
        &txn,
        uid,
        data,
        "credit_cards",
        "user_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_into::<fixed_deposits::Entity, _>(
        &txn,
        uid,
        data,
        "fixed_deposits",
        "user_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_into::<investments::Entity, _>(
        &txn,
        uid,
        data,
        "investments",
        "user_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_into::<assets::Entity, _>(
        &txn,
        uid,
        data,
        "assets",
        "user_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_into::<asset_buy_list::Entity, _>(
        &txn,
        uid,
        data,
        "asset_buy_list",
        "user_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_into::<incomes::Entity, _>(
        &txn,
        uid,
        data,
        "incomes",
        "user_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_into::<expenses::Entity, _>(
        &txn,
        uid,
        data,
        "expenses",
        "user_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_into::<debts::Entity, _>(
        &txn,
        uid,
        data,
        "debts",
        "user_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    txn.commit().await?;

    let txn2 = db.begin().await?;
    wipe_notes(&txn2, uid).await?;
    load_into::<notes::note::Entity, _>(
        &txn2,
        uid,
        data,
        "notes",
        "owner_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_into::<notes::label::Entity, _>(
        &txn2,
        uid,
        data,
        "labels",
        "owner_id",
        &mut imported,
        &mut skipped,
    )
    .await?;
    // Join rows carry no owner column; the wipe above already scoped them to
    // this user's notes/labels, and fresh replace means no orphans.
    load_join::<notes::list_item::Entity, _>(
        &txn2,
        data,
        "list_items",
        &mut imported,
        &mut skipped,
    )
    .await?;
    load_join::<notes::note_label::Entity, _>(
        &txn2,
        data,
        "note_labels",
        &mut imported,
        &mut skipped,
    )
    .await?;
    txn2.commit().await?;

    format::json(serde_json::json!({"imported": imported, "skipped": skipped}))
}

async fn wipe_finance(txn: &sea_orm_turso::TursoTransaction, uid: i64) -> Result<()> {
    banks::Entity::delete_many()
        .filter(banks::Column::UserId.eq(uid))
        .exec(txn)
        .await?;
    credit_cards::Entity::delete_many()
        .filter(credit_cards::Column::UserId.eq(uid))
        .exec(txn)
        .await?;
    fixed_deposits::Entity::delete_many()
        .filter(fixed_deposits::Column::UserId.eq(uid))
        .exec(txn)
        .await?;
    investments::Entity::delete_many()
        .filter(investments::Column::UserId.eq(uid))
        .exec(txn)
        .await?;
    assets::Entity::delete_many()
        .filter(assets::Column::UserId.eq(uid))
        .exec(txn)
        .await?;
    asset_buy_list::Entity::delete_many()
        .filter(asset_buy_list::Column::UserId.eq(uid))
        .exec(txn)
        .await?;
    incomes::Entity::delete_many()
        .filter(incomes::Column::UserId.eq(uid))
        .exec(txn)
        .await?;
    expenses::Entity::delete_many()
        .filter(expenses::Column::UserId.eq(uid))
        .exec(txn)
        .await?;
    debts::Entity::delete_many()
        .filter(debts::Column::UserId.eq(uid))
        .exec(txn)
        .await?;
    Ok(())
}

async fn wipe_notes(txn: &sea_orm::DatabaseTransaction, uid: i64) -> Result<()> {
    let note_ids: Vec<String> = notes::note::Entity::find()
        .filter(notes::note::Column::OwnerId.eq(uid))
        .all(txn)
        .await?
        .into_iter()
        .map(|n| n.id)
        .collect();
    if !note_ids.is_empty() {
        notes::list_item::Entity::delete_many()
            .filter(notes::list_item::Column::NoteId.is_in(note_ids.clone()))
            .exec(txn)
            .await?;
        notes::note_label::Entity::delete_many()
            .filter(notes::note_label::Column::NoteId.is_in(note_ids))
            .exec(txn)
            .await?;
    }
    let label_ids: Vec<String> = notes::label::Entity::find()
        .filter(notes::label::Column::OwnerId.eq(uid))
        .all(txn)
        .await?
        .into_iter()
        .map(|l| l.id)
        .collect();
    if !label_ids.is_empty() {
        notes::note_label::Entity::delete_many()
            .filter(notes::note_label::Column::LabelId.is_in(label_ids))
            .exec(txn)
            .await?;
    }
    notes::note::Entity::delete_many()
        .filter(notes::note::Column::OwnerId.eq(uid))
        .exec(txn)
        .await?;
    notes::label::Entity::delete_many()
        .filter(notes::label::Column::OwnerId.eq(uid))
        .exec(txn)
        .await?;
    Ok(())
}

/// Insert exported rows verbatim (same UUIDs so joins survive), stamping the
/// caller as owner. `C` is any `ConnectionTrait` (txn on either database).
async fn load_into<E, C>(
    db: &C,
    uid: i64,
    data: &serde_json::Map<String, Value>,
    key: &str,
    owner_key: &str,
    imported: &mut std::collections::BTreeMap<String, usize>,
    skipped: &mut std::collections::BTreeMap<String, usize>,
) -> Result<()>
where
    E: EntityTrait,
    E::Model: serde::de::DeserializeOwned + IntoActiveModel<E::ActiveModel>,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    C: ConnectionTrait,
{
    let arr = table_array(data, key)?;
    let (mut i, mut sk) = (0usize, 0usize);
    for row in &arr {
        let Some(obj) = row.as_object().cloned() else {
            sk += 1;
            continue;
        };
        let Some(am) = prepared::<E>(obj, uid, owner_key) else {
            sk += 1;
            continue;
        };
        E::insert(am)
            .exec(db)
            .await
            .map_err(|e| Error::string(&e.to_string()))?;
        i += 1;
    }
    imported.insert(key.to_string(), i);
    skipped.insert(key.to_string(), sk);
    Ok(())
}

/// Join rows (no owner column, no id requirement for note_labels).
async fn load_join<E, C>(
    db: &C,
    data: &serde_json::Map<String, Value>,
    key: &str,
    imported: &mut std::collections::BTreeMap<String, usize>,
    skipped: &mut std::collections::BTreeMap<String, usize>,
) -> Result<()>
where
    E: EntityTrait,
    E::Model: serde::de::DeserializeOwned + IntoActiveModel<E::ActiveModel>,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    C: ConnectionTrait,
{
    let arr = table_array(data, key)?;
    let (mut i, mut sk) = (0usize, 0usize);
    for row in &arr {
        let Some(obj) = row.as_object().cloned() else {
            sk += 1;
            continue;
        };
        // Join rows carry no owner. list_items need id + note; note_labels
        // need both join ends.
        let has_note = obj
            .get("note_id")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        if !has_note {
            sk += 1;
            continue;
        }
        if key == "list_items"
            && obj
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false)
                == false
        {
            sk += 1;
            continue;
        }
        match serde_json::from_value::<E::Model>(Value::Object(obj)) {
            Ok(m) => {
                E::insert(m.into_active_model())
                    .exec(db)
                    .await
                    .map_err(|e| Error::string(&e.to_string()))?;
                i += 1;
            }
            Err(_) => sk += 1,
        }
    }
    imported.insert(key.to_string(), i);
    skipped.insert(key.to_string(), sk);
    Ok(())
}

/// Stamp caller as owner; require a non-empty string id.
fn prepared<E>(
    mut obj: serde_json::Map<String, Value>,
    uid: i64,
    owner_key: &str,
) -> Option<E::ActiveModel>
where
    E: EntityTrait,
    E::Model: serde::de::DeserializeOwned + IntoActiveModel<E::ActiveModel>,
{
    let id_ok = obj
        .get("id")
        .and_then(|v| v.as_str())
        .map(|id| !id.is_empty())
        .unwrap_or(false);
    if !id_ok {
        return None;
    }
    obj.insert(owner_key.to_string(), Value::from(uid));
    serde_json::from_value::<E::Model>(Value::Object(obj))
        .ok()
        .map(|m| m.into_active_model())
}

fn table_array(data: &serde_json::Map<String, Value>, key: &str) -> Result<Vec<Value>> {
    match data.get(key) {
        None => Ok(Vec::new()),
        Some(Value::Array(a)) => Ok(a.clone()),
        Some(_) => Err(Error::BadRequest(format!(
            "backup table '{key}' must be an array"
        ))),
    }
}
