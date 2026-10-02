use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "banks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i64,
    pub name: String,
    pub bank_type: String,
    pub account_number: String,
    pub ifsc_code: Option<String>,
    pub branch: Option<String>,
    pub current_balance: f64,
    pub currency: String,
    pub is_active: bool,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateBankRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub bank_type: String,
    pub account_number: String,
    pub ifsc_code: Option<String>,
    pub branch: Option<String>,
    pub current_balance: Option<f64>,
    pub currency: Option<String>,
    pub is_active: Option<bool>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl CreateBankRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            name: Set(self.name),
            bank_type: Set(self.bank_type),
            account_number: Set(self.account_number),
            ifsc_code: Set(self.ifsc_code),
            branch: Set(self.branch),
            current_balance: Set(self.current_balance.unwrap_or(0.0)),
            currency: Set(self.currency.unwrap_or_else(|| "INR".to_string())),
            is_active: Set(self.is_active.unwrap_or(true)),
            sync_status: Set(self.sync_status.unwrap_or_else(|| "synced".to_string())),
            last_synced_at: Set(self.last_synced_at),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateBankRequest {
    pub name: Option<String>,
    pub bank_type: Option<String>,
    pub account_number: Option<String>,
    pub ifsc_code: Option<String>,
    pub branch: Option<String>,
    pub current_balance: Option<f64>,
    pub currency: Option<String>,
    pub is_active: Option<bool>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl UpdateBankRequest {
    pub fn apply(self, current: &Model) -> ActiveModel {
        let mut am = current.clone().into_active_model();
        if let Some(v) = self.name {
            am.name = Set(v);
        }
        if let Some(v) = self.bank_type {
            am.bank_type = Set(v);
        }
        if let Some(v) = self.account_number {
            am.account_number = Set(v);
        }
        if let Some(v) = self.ifsc_code {
            am.ifsc_code = Set(Some(v));
        }
        if let Some(v) = self.branch {
            am.branch = Set(Some(v));
        }
        if let Some(v) = self.current_balance {
            am.current_balance = Set(v);
        }
        if let Some(v) = self.currency {
            am.currency = Set(v);
        }
        if let Some(v) = self.is_active {
            am.is_active = Set(v);
        }
        if let Some(v) = self.sync_status {
            am.sync_status = Set(v);
        }
        if let Some(v) = self.last_synced_at {
            am.last_synced_at = Set(Some(v));
        }
        am.updated_at = Set(Some(now()));
        am
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankResponse {
    pub id: String,
    pub user_id: i64,
    pub name: String,
    pub bank_type: String,
    pub account_number: String,
    pub ifsc_code: Option<String>,
    pub branch: Option<String>,
    pub current_balance: f64,
    pub currency: String,
    pub is_active: bool,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Model> for BankResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            user_id: m.user_id,
            name: m.name,
            bank_type: m.bank_type,
            account_number: m.account_number,
            ifsc_code: m.ifsc_code,
            branch: m.branch,
            current_balance: m.current_balance,
            currency: m.currency,
            is_active: m.is_active,
            sync_status: m.sync_status,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankSummary {
    pub total_balance: f64,
    pub count: i64,
}
