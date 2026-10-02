use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn annualize(amount: f64, recurrence: Option<&str>, multiplier: i64) -> f64 {
    let m = amount * multiplier.max(1) as f64;
    match recurrence.unwrap_or("") {
        "weekly" => m * 52.0,
        "biweekly" | "fortnightly" => m * 26.0,
        "monthly" => m * 12.0,
        "quarterly" => m * 4.0,
        "yearly" | "annually" => m,
        _ if multiplier > 1 => m * 12.0,
        _ => m,
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "incomes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub income_type: String,
    pub source: String,
    pub income_date: String,
    pub is_recurring: bool,
    pub recurrence: Option<String>,
    pub frequency_multiplier: i64,
    pub is_gross: bool,
    pub tax_withheld: f64,
    pub bank_id: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateIncomeRequest {
    #[validate(length(min = 1, message = "title is required"))]
    pub title: String,
    pub description: Option<String>,
    pub amount: f64,
    pub currency: Option<String>,
    pub income_type: String,
    pub source: String,
    pub income_date: String,
    pub is_recurring: Option<bool>,
    pub recurrence: Option<String>,
    pub frequency_multiplier: Option<i64>,
    pub is_gross: Option<bool>,
    pub tax_withheld: Option<f64>,
    pub bank_id: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl CreateIncomeRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            title: Set(self.title),
            description: Set(self.description),
            amount: Set(self.amount),
            currency: Set(self.currency.unwrap_or_else(|| "INR".to_string())),
            income_type: Set(self.income_type),
            source: Set(self.source),
            income_date: Set(self.income_date),
            is_recurring: Set(self.is_recurring.unwrap_or(false)),
            recurrence: Set(self.recurrence),
            frequency_multiplier: Set(self.frequency_multiplier.unwrap_or(1)),
            is_gross: Set(self.is_gross.unwrap_or(true)),
            tax_withheld: Set(self.tax_withheld.unwrap_or(0.0)),
            bank_id: Set(self.bank_id),
            tags: Set(self.tags),
            notes: Set(self.notes),
            sync_status: Set(self.sync_status.unwrap_or_else(|| "synced".to_string())),
            last_synced_at: Set(self.last_synced_at),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateIncomeRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub income_type: Option<String>,
    pub source: Option<String>,
    pub income_date: Option<String>,
    pub is_recurring: Option<bool>,
    pub recurrence: Option<String>,
    pub frequency_multiplier: Option<i64>,
    pub is_gross: Option<bool>,
    pub tax_withheld: Option<f64>,
    pub bank_id: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl UpdateIncomeRequest {
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
        if let Some(v) = self.income_type {
            am.income_type = Set(v);
        }
        if let Some(v) = self.source {
            am.source = Set(v);
        }
        if let Some(v) = self.income_date {
            am.income_date = Set(v);
        }
        if let Some(v) = self.is_recurring {
            am.is_recurring = Set(v);
        }
        if let Some(v) = self.recurrence {
            am.recurrence = Set(Some(v));
        }
        if let Some(v) = self.frequency_multiplier {
            am.frequency_multiplier = Set(v);
        }
        if let Some(v) = self.is_gross {
            am.is_gross = Set(v);
        }
        if let Some(v) = self.tax_withheld {
            am.tax_withheld = Set(v);
        }
        if let Some(v) = self.bank_id {
            am.bank_id = Set(Some(v));
        }
        if let Some(v) = self.tags {
            am.tags = Set(Some(v));
        }
        if let Some(v) = self.notes {
            am.notes = Set(Some(v));
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
pub struct IncomeResponse {
    pub id: String,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub income_type: String,
    pub source: String,
    pub income_date: String,
    pub is_recurring: bool,
    pub recurrence: Option<String>,
    pub frequency_multiplier: i64,
    pub is_gross: bool,
    pub tax_withheld: f64,
    pub bank_id: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub annualized: f64,
    pub monthly_equivalent: f64,
    pub net_amount: f64,
}

impl From<Model> for IncomeResponse {
    fn from(m: Model) -> Self {
        let net_amount = m.amount - m.tax_withheld;
        let annualized = if m.is_recurring {
            annualize(m.amount, m.recurrence.as_deref(), m.frequency_multiplier)
        } else {
            m.amount * m.frequency_multiplier.max(1) as f64
        };
        let monthly_equivalent = annualized / 12.0;
        Self {
            id: m.id,
            user_id: m.user_id,
            title: m.title,
            description: m.description,
            amount: m.amount,
            currency: m.currency,
            income_type: m.income_type,
            source: m.source,
            income_date: m.income_date,
            is_recurring: m.is_recurring,
            recurrence: m.recurrence,
            frequency_multiplier: m.frequency_multiplier,
            is_gross: m.is_gross,
            tax_withheld: m.tax_withheld,
            bank_id: m.bank_id,
            tags: m.tags,
            notes: m.notes,
            sync_status: m.sync_status,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
            annualized,
            monthly_equivalent,
            net_amount,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeSummary {
    pub yearly_income: f64,
    pub monthly_income: f64,
    pub received_this_year: f64,
    pub by_type: Vec<IncomeTypeTotal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeTypeTotal {
    pub income_type: String,
    pub total: f64,
}
