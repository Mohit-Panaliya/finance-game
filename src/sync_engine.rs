use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use validator::Validate;

use crate::models::{
    assets, banks, credit_cards, expenses, fixed_deposits, incomes, investments, sync,
};

/// Entities the sync whitelist accepts, in canonical snake_case form.
/// Client payloads are normalised (`-` -> `_`) before matching, so a client
/// may send either `credit_cards` or `credit-cards`.
const SYNC_ENTITIES: [&str; 7] = [
    "banks",
    "assets",
    "expenses",
    "credit_cards",
    "fixed_deposits",
    "investments",
    "incomes",
];

fn normalise_entity(entity: &str) -> String {
    entity.trim().replace('-', "_")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOp {
    pub entity: String,
    #[serde(default)]
    pub entity_id: String,
    pub op: String,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub client_ts: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncOpResult {
    pub entity: String,
    pub entity_id: String,
    pub status: String,
    pub server_ts: String,
}

pub type DbRef<'a> = sea_orm_turso::TursoConnection;

async fn already_applied(
    db: &DbRef<'_>,
    user_id: i64,
    entity: &str,
    entity_id: &str,
    client_ts: Option<&str>,
) -> Result<bool, sea_orm::DbErr> {
    if let Some(ts) = client_ts {
        let hit = sync::Entity::find()
            .filter(sync::Column::UserId.eq(user_id))
            .filter(sync::Column::Entity.eq(entity.to_string()))
            .filter(sync::Column::EntityId.eq(entity_id.to_string()))
            .filter(sync::Column::ClientTs.eq(ts.to_string()))
            .one(db)
            .await?;
        return Ok(hit.is_some());
    }
    Ok(false)
}

async fn log_sync(
    db: &DbRef<'_>,
    user_id: i64,
    entity: &str,
    entity_id: &str,
    op: &str,
    payload: &Value,
    client_ts: Option<String>,
    status: &str,
) -> Result<(), sea_orm::DbErr> {
    let server_ts = Utc::now().to_rfc3339();
    sync::ActiveModel {
        user_id: Set(user_id),
        entity: Set(entity.to_string()),
        entity_id: Set(entity_id.to_string()),
        op: Set(op.to_string()),
        payload: Set(Some(payload.to_string())),
        client_ts: Set(client_ts),
        server_ts: Set(Some(server_ts)),
        status: Set(status.to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}

/// Outcome of applying a single queued op. A `bool` is not enough: an op can
/// be refused because it did nothing (`NotApplied`) or because the payload was
/// unusable (`RejectedInvalidPayload`), and those need different statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The row was inserted, updated or deleted.
    Applied,
    /// The payload did not deserialise, or failed `validator::Validate`.
    RejectedInvalidPayload,
    /// Nothing was written: unknown op verb, missing row, or no row matched.
    NotApplied,
}

impl ApplyOutcome {
    pub fn status(self) -> &'static str {
        match self {
            ApplyOutcome::Applied => "applied",
            ApplyOutcome::RejectedInvalidPayload => "rejected_invalid_payload",
            ApplyOutcome::NotApplied => "rejected",
        }
    }
}

/// Apply one queued op for one entity.
///
/// `create`  — insert with the client-supplied id; if that id already exists for
///             the user it is merged onto the existing row instead of erroring.
/// `update`  — merge onto the row with that id *and* user; a missing row is
///             reported as not applied rather than creating a phantom.
/// `delete`  — always scoped by user.
///
/// Deserialisation/validation failures resolve to `RejectedInvalidPayload`
/// instead of `Err`, so one bad payload cannot abort the rest of the batch.
macro_rules! apply_op {
    ($entity:ident, $create_req:ty, $update_req:ty, $db:expr, $user_id:expr, $op:expr, $entity_id:expr, $payload:expr) => {{
        async move {
            match $op {
                "delete" => {
                    let res = $entity::Entity::delete_many()
                        .filter($entity::Column::Id.eq($entity_id.to_string()))
                        .filter($entity::Column::UserId.eq($user_id))
                        .exec($db)
                        .await?;
                    Ok::<ApplyOutcome, sea_orm::DbErr>(if res.rows_affected > 0 {
                        ApplyOutcome::Applied
                    } else {
                        ApplyOutcome::NotApplied
                    })
                }
                "create" => {
                    let existing = $entity::Entity::find_by_id($entity_id.to_string())
                        .filter($entity::Column::UserId.eq($user_id))
                        .one($db)
                        .await?;
                    let payload = Value::clone($payload);
                    match existing {
                        // Upsert: the id is already taken by this user, so merge
                        // the queued fields onto the stored row.
                        Some(current) => {
                            let req = match serde_json::from_value::<$update_req>(payload) {
                                Ok(req) => req,
                                Err(e) => {
                                    tracing::warn!(
                                        entity = stringify!($entity),
                                        entity_id = $entity_id,
                                        op = $op,
                                        error = %e,
                                        "sync create payload rejected (upsert)"
                                    );
                                    return Ok(ApplyOutcome::RejectedInvalidPayload);
                                }
                            };
                            req.apply(&current).update($db).await?;
                            Ok(ApplyOutcome::Applied)
                        }
                        None => {
                            let req = match serde_json::from_value::<$create_req>(payload) {
                                Ok(req) => req,
                                Err(e) => {
                                    tracing::warn!(
                                        entity = stringify!($entity),
                                        entity_id = $entity_id,
                                        op = $op,
                                        error = %e,
                                        "sync create payload did not deserialize"
                                    );
                                    return Ok(ApplyOutcome::RejectedInvalidPayload);
                                }
                            };
                            if let Err(e) = req.validate() {
                                tracing::warn!(
                                    entity = stringify!($entity),
                                    entity_id = $entity_id,
                                    op = $op,
                                    error = %e,
                                    "sync create payload failed validation"
                                );
                                return Ok(ApplyOutcome::RejectedInvalidPayload);
                            }
                            let mut active = req.into_active($user_id);
                            // Keep the client-supplied id so a later queued
                            // update/delete for the same record matches.
                            active.id = Set($entity_id.to_string());
                            active.insert($db).await?;
                            Ok(ApplyOutcome::Applied)
                        }
                    }
                }
                "update" => {
                    let current = match $entity::Entity::find_by_id($entity_id.to_string())
                        .filter($entity::Column::UserId.eq($user_id))
                        .one($db)
                        .await?
                    {
                        Some(current) => current,
                        None => {
                            tracing::warn!(
                                entity = stringify!($entity),
                                entity_id = $entity_id,
                                op = $op,
                                "sync update target not found for user"
                            );
                            return Ok(ApplyOutcome::NotApplied);
                        }
                    };
                    let req =
                        match serde_json::from_value::<$update_req>(Value::clone($payload)) {
                            Ok(req) => req,
                            Err(e) => {
                                tracing::warn!(
                                    entity = stringify!($entity),
                                    entity_id = $entity_id,
                                    op = $op,
                                    error = %e,
                                    "sync update payload did not deserialize"
                                );
                                return Ok(ApplyOutcome::RejectedInvalidPayload);
                            }
                        };
                    req.apply(&current).update($db).await?;
                    Ok(ApplyOutcome::Applied)
                }
                _ => Ok(ApplyOutcome::NotApplied),
            }
        }
    }};
}

pub async fn apply_push(
    db: &DbRef<'_>,
    user_id: i64,
    ops: &[SyncOp],
) -> Result<Vec<SyncOpResult>, sea_orm::DbErr> {
    let mut results = Vec::new();
    for op in ops {
        let normalised = normalise_entity(&op.entity);
        let entity = normalised.as_str();
        let id = op.entity_id.as_str();
        let client_ts = op.client_ts.as_deref();
        let server_ts = Utc::now().to_rfc3339();

        if already_applied(db, user_id, entity, id, client_ts).await? {
            results.push(SyncOpResult {
                entity: entity.to_string(),
                entity_id: id.to_string(),
                status: "skipped_duplicate".to_string(),
                server_ts: server_ts.clone(),
            });
            continue;
        }

        let known = SYNC_ENTITIES.contains(&entity);

        if !known {
            log_sync(
                db,
                user_id,
                entity,
                id,
                &op.op,
                &op.payload,
                op.client_ts.clone(),
                "rejected",
            )
            .await?;
            results.push(SyncOpResult {
                entity: entity.to_string(),
                entity_id: id.to_string(),
                status: "rejected_unknown_entity".to_string(),
                server_ts: server_ts.clone(),
            });
            continue;
        }

        // Payload write path: create/update/delete are applied to the entity's
        // own table here (queued while the device was offline), so a queued
        // mutation is never silently dropped. Only an op that genuinely did
        // something — or that failed in a way the client should see — is
        // reported as anything other than rejected.
        let outcome = match entity {
            "banks" => {
                apply_op!(
                    banks,
                    banks::CreateBankRequest,
                    banks::UpdateBankRequest,
                    db,
                    user_id,
                    op.op.as_str(),
                    id,
                    &op.payload
                )
                .await?
            }
            "assets" => {
                apply_op!(
                    assets,
                    assets::CreateAssetRequest,
                    assets::UpdateAssetRequest,
                    db,
                    user_id,
                    op.op.as_str(),
                    id,
                    &op.payload
                )
                .await?
            }
            "expenses" => {
                apply_op!(
                    expenses,
                    expenses::CreateExpenseRequest,
                    expenses::UpdateExpenseRequest,
                    db,
                    user_id,
                    op.op.as_str(),
                    id,
                    &op.payload
                )
                .await?
            }
            "credit_cards" => {
                apply_op!(
                    credit_cards,
                    credit_cards::CreateCreditCardRequest,
                    credit_cards::UpdateCreditCardRequest,
                    db,
                    user_id,
                    op.op.as_str(),
                    id,
                    &op.payload
                )
                .await?
            }
            "fixed_deposits" => {
                apply_op!(
                    fixed_deposits,
                    fixed_deposits::CreateFixedDepositRequest,
                    fixed_deposits::UpdateFixedDepositRequest,
                    db,
                    user_id,
                    op.op.as_str(),
                    id,
                    &op.payload
                )
                .await?
            }
            "investments" => {
                apply_op!(
                    investments,
                    investments::CreateInvestmentRequest,
                    investments::UpdateInvestmentRequest,
                    db,
                    user_id,
                    op.op.as_str(),
                    id,
                    &op.payload
                )
                .await?
            }
            "incomes" => {
                apply_op!(
                    incomes,
                    incomes::CreateIncomeRequest,
                    incomes::UpdateIncomeRequest,
                    db,
                    user_id,
                    op.op.as_str(),
                    id,
                    &op.payload
                )
                .await?
            }
            _ => ApplyOutcome::NotApplied,
        };

        let status = outcome.status();

        log_sync(
            db,
            user_id,
            entity,
            id,
            &op.op,
            &op.payload,
            op.client_ts.clone(),
            status,
        )
        .await?;

        results.push(SyncOpResult {
            entity: entity.to_string(),
            entity_id: id.to_string(),
            status: status.to_string(),
            server_ts,
        });
    }
    Ok(results)
}

/// `updated_at` is stored as an RFC3339 string, so a client-supplied cursor
/// has to be normalised to that form. Clients send either an RFC3339 string or
/// a numeric epoch value (milliseconds when large, seconds when small); an
/// unparseable cursor is ignored, which degrades to a full pull.
fn since_rfc3339(since: Option<&str>) -> Option<String> {
    let raw = since?.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(raw) {
        return Some(ts.with_timezone(&Utc).to_rfc3339());
    }
    let n: i64 = raw.parse().ok()?;
    let secs = if n.abs() > 100_000_000_000 {
        n / 1000
    } else {
        n
    };
    use chrono::TimeZone;
    Utc.timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.to_rfc3339())
}

pub async fn pull_changes(
    db: &DbRef<'_>,
    user_id: i64,
    since: Option<&str>,
) -> Result<Value, sea_orm::DbErr> {
    let mut changes = serde_json::Map::new();
    let since_ts = since_rfc3339(since);

    macro_rules! pull_table {
        ($key:expr, $entity:ident) => {{
            mod imp {
                pub use crate::models::$entity;
            }
            let mut query = imp::$entity::Entity::find()
                .filter(imp::$entity::Column::UserId.eq(user_id));
            if let Some(ts) = since_ts.clone() {
                query = query.filter(imp::$entity::Column::UpdatedAt.gt(Some(ts)));
            }
            let rows = query.all(db).await?;
            if !rows.is_empty() {
                changes.insert(
                    $key.to_string(),
                    serde_json::to_value(&rows).unwrap_or(Value::Null),
                );
            }
        }};
    }

    pull_table!("banks", banks);
    pull_table!("assets", assets);
    pull_table!("expenses", expenses);
    pull_table!("credit_cards", credit_cards);
    pull_table!("fixed_deposits", fixed_deposits);
    pull_table!("investments", investments);
    pull_table!("incomes", incomes);

    let sync_rows = sync::Entity::find()
        .filter(sync::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    if !sync_rows.is_empty() {
        changes.insert(
            "sync_log".to_string(),
            serde_json::to_value(&sync_rows).unwrap_or(Value::Null),
        );
    }

    Ok(serde_json::json!({
        "changes": Value::Object(changes),
        "server_ts": Utc::now().to_rfc3339(),
    }))
}
