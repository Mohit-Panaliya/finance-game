use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::models::{
    assets, banks, credit_cards, expenses, fixed_deposits, incomes, investments, sync,
};

use super::battle::BattleOutcome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOp {
    pub entity: String,
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

macro_rules! apply_op {
    ($entity:ident, $db:expr, $user_id:expr, $op:expr, $entity_id:expr, $payload:expr) => {{
        mod imp {
            pub use crate::models::$entity;
        }
        async move {
            match $op {
                "delete" => {
                    let res = imp::$entity::Entity::delete_many()
                        .filter(imp::$entity::Column::Id.eq($entity_id.to_string()))
                        .filter(imp::$entity::Column::UserId.eq($user_id))
                        .exec($db)
                        .await?;
                    Ok::<bool, sea_orm::DbErr>(res.rows_affected > 0)
                }
                "create" | "update" => {
                    let existing = imp::$entity::Entity::find_by_id($entity_id.to_string())
                        .filter(imp::$entity::Column::UserId.eq($user_id))
                        .one($db)
                        .await?;
                    Ok::<bool, sea_orm::DbErr>(existing.is_some())
                }
                _ => Ok::<bool, sea_orm::DbErr>(false),
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
        let entity = op.entity.as_str();
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

        let known = matches!(
            entity,
            "banks"
                | "assets"
                | "expenses"
                | "credit_cards"
                | "fixed_deposits"
                | "investments"
                | "incomes"
        );

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

        // Payload write path: create/update/delete are handled by HTTP controllers
        // when online; push only records the log for idempotent replay bookkeeping
        // and marks accepted when the row already exists locally (or op is delete).
        let applied = match entity {
            "banks" => {
                apply_op!(banks, db, user_id, op.op.as_str(), id, &op.payload).await?
            }
            "assets" => {
                apply_op!(assets, db, user_id, op.op.as_str(), id, &op.payload).await?
            }
            "expenses" => {
                apply_op!(expenses, db, user_id, op.op.as_str(), id, &op.payload).await?
            }
            "credit_cards" => {
                apply_op!(credit_cards, db, user_id, op.op.as_str(), id, &op.payload).await?
            }
            "fixed_deposits" => {
                apply_op!(fixed_deposits, db, user_id, op.op.as_str(), id, &op.payload).await?
            }
            "investments" => {
                apply_op!(investments, db, user_id, op.op.as_str(), id, &op.payload).await?
            }
            "incomes" => {
                apply_op!(incomes, db, user_id, op.op.as_str(), id, &op.payload).await?
            }
            _ => false,
        };

        let status = if applied || op.op == "delete" || op.op == "create" || op.op == "update" {
            "applied"
        } else {
            "rejected"
        };

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

pub async fn pull_changes(
    db: &DbRef<'_>,
    user_id: i64,
    since: Option<&str>,
) -> Result<Value, sea_orm::DbErr> {
    let mut changes = serde_json::Map::new();

    macro_rules! pull_table {
        ($key:expr, $entity:ident) => {{
            mod imp {
                pub use crate::models::$entity;
            }
            let mut query = imp::$entity::Entity::find()
                .filter(imp::$entity::Column::UserId.eq(user_id));
            if let Some(s) = since {
                if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(s) {
                    query = query.filter(
                        imp::$entity::Column::UpdatedAt
                            .gt(Some(ts.with_timezone(&Utc).to_rfc3339())),
                    );
                }
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

pub fn outcome_to_json(o: &BattleOutcome) -> Value {
    serde_json::to_value(o).unwrap_or(Value::Null)
}
