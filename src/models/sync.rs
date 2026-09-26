use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sync_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub user_id: i64,
    pub entity: String,
    pub entity_id: String,
    pub op: String,
    pub payload: Option<String>,
    pub client_ts: Option<String>,
    pub server_ts: Option<String>,
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntryRequest {
    pub entity: String,
    pub entity_id: String,
    pub op: String,
    pub payload: Option<String>,
    pub client_ts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntryResponse {
    pub id: i64,
    pub user_id: i64,
    pub entity: String,
    pub entity_id: String,
    pub op: String,
    pub payload: Option<String>,
    pub client_ts: Option<String>,
    pub server_ts: Option<String>,
    pub status: String,
}

impl From<Model> for SyncEntryResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            user_id: m.user_id,
            entity: m.entity,
            entity_id: m.entity_id,
            op: m.op,
            payload: m.payload,
            client_ts: m.client_ts,
            server_ts: m.server_ts,
            status: m.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPushRequest {
    pub entries: Vec<SyncEntryRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPushResponse {
    pub applied: i64,
    pub conflicts: i64,
    pub results: Vec<SyncEntryResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntryResult {
    pub entity: String,
    pub entity_id: String,
    pub status: String,
    pub server_ts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPullResponse {
    pub changes: Vec<SyncChange>,
    pub server_ts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncChange {
    pub entity: String,
    pub entity_id: String,
    pub op: String,
    pub payload: Option<String>,
    pub server_ts: Option<String>,
}
