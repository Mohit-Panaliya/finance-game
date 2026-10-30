use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "transfers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i64,
    pub from_bank_id: String,
    pub to_bank_id: String,
    pub amount: f64,
    pub currency: String,
    pub transfer_date: String,
    pub description: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTransferRequest {
    pub from_bank_id: String,
    pub to_bank_id: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub transfer_date: Option<String>,
    pub description: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl CreateTransferRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            from_bank_id: Set(self.from_bank_id),
            to_bank_id: Set(self.to_bank_id),
            amount: Set(self.amount),
            currency: Set(self.currency.unwrap_or_else(|| "INR".to_string())),
            transfer_date: Set(self
                .transfer_date
                .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string())),
            description: Set(self.description),
            sync_status: Set(self.sync_status.unwrap_or_else(|| "synced".to_string())),
            last_synced_at: Set(self.last_synced_at),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub id: String,
    pub user_id: i64,
    pub from_bank_id: String,
    pub to_bank_id: String,
    pub amount: f64,
    pub currency: String,
    pub transfer_date: String,
    pub description: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Model> for TransferResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            user_id: m.user_id,
            from_bank_id: m.from_bank_id,
            to_bank_id: m.to_bank_id,
            amount: m.amount,
            currency: m.currency,
            transfer_date: m.transfer_date,
            description: m.description,
            sync_status: m.sync_status,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}
