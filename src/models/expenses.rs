use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "expenses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub expense_type: String,
    pub category: String,
    pub expense_date: String,
    pub is_recurring: bool,
    pub recurrence: Option<String>,
    pub recurrence_end_date: Option<String>,
    pub payment_method: String,
    pub bank_id: Option<String>,
    pub credit_card_id: Option<String>,
    pub tags: Option<String>,
    pub receipt_url: Option<String>,
    pub location: Option<String>,
    pub is_fixed: bool,
    pub priority: String,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateExpenseRequest {
    #[validate(length(min = 1, message = "title is required"))]
    pub title: String,
    pub description: Option<String>,
    pub amount: f64,
    pub currency: Option<String>,
    pub expense_type: String,
    pub category: String,
    pub expense_date: String,
    pub is_recurring: Option<bool>,
    pub recurrence: Option<String>,
    pub recurrence_end_date: Option<String>,
    pub payment_method: Option<String>,
    pub bank_id: Option<String>,
    pub credit_card_id: Option<String>,
    pub tags: Option<String>,
    pub receipt_url: Option<String>,
    pub location: Option<String>,
    pub is_fixed: Option<bool>,
    pub priority: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl CreateExpenseRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            title: Set(self.title),
            description: Set(self.description),
            amount: Set(self.amount),
            currency: Set(self.currency.unwrap_or_else(|| "INR".to_string())),
            expense_type: Set(self.expense_type),
            category: Set(self.category),
            expense_date: Set(self.expense_date),
            is_recurring: Set(self.is_recurring.unwrap_or(false)),
            recurrence: Set(self.recurrence),
            recurrence_end_date: Set(self.recurrence_end_date),
            payment_method: Set(self.payment_method.unwrap_or_else(|| "cash".to_string())),
            bank_id: Set(self.bank_id),
            credit_card_id: Set(self.credit_card_id),
            tags: Set(self.tags),
            receipt_url: Set(self.receipt_url),
            location: Set(self.location),
            is_fixed: Set(self.is_fixed.unwrap_or(false)),
            priority: Set(self.priority.unwrap_or_else(|| "medium".to_string())),
            sync_status: Set(self.sync_status.unwrap_or_else(|| "synced".to_string())),
            last_synced_at: Set(self.last_synced_at),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateExpenseRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub expense_type: Option<String>,
    pub category: Option<String>,
    pub expense_date: Option<String>,
    pub is_recurring: Option<bool>,
    pub recurrence: Option<String>,
    pub recurrence_end_date: Option<String>,
    pub payment_method: Option<String>,
    pub bank_id: Option<String>,
    pub credit_card_id: Option<String>,
    pub tags: Option<String>,
    pub receipt_url: Option<String>,
    pub location: Option<String>,
    pub is_fixed: Option<bool>,
    pub priority: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl UpdateExpenseRequest {
    pub fn apply(self, current: &Model) -> ActiveModel {
        let mut am = current.clone().into_active_model();
        if let Some(v) = self.title {
            am.title = Set(v);
        }
        if let Some(v) = self.description {
            am.description = Set(Some(v));
        }
        if let Some(v) = self.amount {
            am.amount = Set(v);
        }
        if let Some(v) = self.currency {
            am.currency = Set(v);
        }
        if let Some(v) = self.expense_type {
            am.expense_type = Set(v);
        }
        if let Some(v) = self.category {
            am.category = Set(v);
        }
        if let Some(v) = self.expense_date {
            am.expense_date = Set(v);
        }
        if let Some(v) = self.is_recurring {
            am.is_recurring = Set(v);
        }
        if let Some(v) = self.recurrence {
            am.recurrence = Set(Some(v));
        }
        if let Some(v) = self.recurrence_end_date {
            am.recurrence_end_date = Set(Some(v));
        }
        if let Some(v) = self.payment_method {
            am.payment_method = Set(v);
        }
        if let Some(v) = self.bank_id {
            am.bank_id = Set(Some(v));
        }
        if let Some(v) = self.credit_card_id {
            am.credit_card_id = Set(Some(v));
        }
        if let Some(v) = self.tags {
            am.tags = Set(Some(v));
        }
        if let Some(v) = self.receipt_url {
            am.receipt_url = Set(Some(v));
        }
        if let Some(v) = self.location {
            am.location = Set(Some(v));
        }
        if let Some(v) = self.is_fixed {
            am.is_fixed = Set(v);
        }
        if let Some(v) = self.priority {
            am.priority = Set(v);
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
pub struct ExpenseResponse {
    pub id: String,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub expense_type: String,
    pub category: String,
    pub expense_date: String,
    pub is_recurring: bool,
    pub recurrence: Option<String>,
    pub recurrence_end_date: Option<String>,
    pub payment_method: String,
    pub bank_id: Option<String>,
    pub credit_card_id: Option<String>,
    pub tags: Option<String>,
    pub receipt_url: Option<String>,
    pub location: Option<String>,
    pub is_fixed: bool,
    pub priority: String,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Model> for ExpenseResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            user_id: m.user_id,
            title: m.title,
            description: m.description,
            amount: m.amount,
            currency: m.currency,
            expense_type: m.expense_type,
            category: m.category,
            expense_date: m.expense_date,
            is_recurring: m.is_recurring,
            recurrence: m.recurrence,
            recurrence_end_date: m.recurrence_end_date,
            payment_method: m.payment_method,
            bank_id: m.bank_id,
            credit_card_id: m.credit_card_id,
            tags: m.tags,
            receipt_url: m.receipt_url,
            location: m.location,
            is_fixed: m.is_fixed,
            priority: m.priority,
            sync_status: m.sync_status,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseCategoryTotal {
    pub category: String,
    pub total: f64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseMonthTotal {
    pub month: String,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseSummary {
    pub total_amount: f64,
    pub fixed_total: f64,
    pub variable_total: f64,
    pub by_category: Vec<ExpenseCategoryTotal>,
    pub by_month: Vec<ExpenseMonthTotal>,
}
