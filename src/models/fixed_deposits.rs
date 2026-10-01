use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use validator::Validate;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn periods_per_year(freq: &str) -> f64 {
    match freq {
        "daily" => 365.0,
        "monthly" => 12.0,
        "semi_annually" | "semiannual" | "half_yearly" => 2.0,
        "yearly" | "annually" => 1.0,
        _ => 4.0,
    }
}

fn compound_value(principal: f64, rate_pct: f64, years: f64, freq: &str) -> f64 {
    let n = periods_per_year(freq);
    principal * (1.0 + (rate_pct / 100.0) / n).powf(n * years)
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "fixed_deposits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i64,
    pub bank_id: Option<String>,
    pub name: String,
    pub fd_type: String,
    pub principal_amount: f64,
    pub interest_rate: f64,
    pub tenure_months: i64,
    pub start_date: String,
    pub maturity_date: String,
    pub compounding_frequency: String,
    pub current_value: f64,
    pub interest_earned: f64,
    pub tax_deducted: f64,
    pub is_auto_renew: bool,
    pub renewal_instructions: Option<String>,
    pub nominee: Option<String>,
    pub certificate_number: Option<String>,
    pub status: String,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateFixedDepositRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub bank_id: Option<String>,
    pub fd_type: Option<String>,
    pub principal_amount: f64,
    pub interest_rate: f64,
    pub tenure_months: i64,
    pub start_date: String,
    pub maturity_date: String,
    pub compounding_frequency: Option<String>,
    pub current_value: Option<f64>,
    pub interest_earned: Option<f64>,
    pub tax_deducted: Option<f64>,
    pub is_auto_renew: Option<bool>,
    pub renewal_instructions: Option<String>,
    pub nominee: Option<String>,
    pub certificate_number: Option<String>,
    pub status: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl CreateFixedDepositRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            bank_id: Set(self.bank_id),
            name: Set(self.name),
            fd_type: Set(self.fd_type.unwrap_or_else(|| "regular".to_string())),
            principal_amount: Set(self.principal_amount),
            interest_rate: Set(self.interest_rate),
            tenure_months: Set(self.tenure_months),
            start_date: Set(self.start_date),
            maturity_date: Set(self.maturity_date),
            compounding_frequency: Set(
                self.compounding_frequency
                    .unwrap_or_else(|| "quarterly".to_string()),
            ),
            current_value: Set(self.current_value.unwrap_or(0.0)),
            interest_earned: Set(self.interest_earned.unwrap_or(0.0)),
            tax_deducted: Set(self.tax_deducted.unwrap_or(0.0)),
            is_auto_renew: Set(self.is_auto_renew.unwrap_or(false)),
            renewal_instructions: Set(self.renewal_instructions),
            nominee: Set(self.nominee),
            certificate_number: Set(self.certificate_number),
            status: Set(self.status.unwrap_or_else(|| "active".to_string())),
            sync_status: Set(self.sync_status.unwrap_or_else(|| "synced".to_string())),
            last_synced_at: Set(self.last_synced_at),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateFixedDepositRequest {
    pub name: Option<String>,
    pub bank_id: Option<String>,
    pub fd_type: Option<String>,
    pub principal_amount: Option<f64>,
    pub interest_rate: Option<f64>,
    pub tenure_months: Option<i64>,
    pub start_date: Option<String>,
    pub maturity_date: Option<String>,
    pub compounding_frequency: Option<String>,
    pub current_value: Option<f64>,
    pub interest_earned: Option<f64>,
    pub tax_deducted: Option<f64>,
    pub is_auto_renew: Option<bool>,
    pub renewal_instructions: Option<String>,
    pub nominee: Option<String>,
    pub certificate_number: Option<String>,
    pub status: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl UpdateFixedDepositRequest {
    pub fn apply(self, current: &Model) -> ActiveModel {
        let mut am = current.clone().into_active_model();
        if let Some(v) = self.name {
            am.name = Set(v);
        }
        if let Some(v) = self.bank_id {
            am.bank_id = Set(Some(v));
        }
        if let Some(v) = self.fd_type {
            am.fd_type = Set(v);
        }
        if let Some(v) = self.principal_amount {
            am.principal_amount = Set(v);
        }
        if let Some(v) = self.interest_rate {
            am.interest_rate = Set(v);
        }
        if let Some(v) = self.tenure_months {
            am.tenure_months = Set(v);
        }
        if let Some(v) = self.start_date {
            am.start_date = Set(v);
        }
        if let Some(v) = self.maturity_date {
            am.maturity_date = Set(v);
        }
        if let Some(v) = self.compounding_frequency {
            am.compounding_frequency = Set(v);
        }
        if let Some(v) = self.current_value {
            am.current_value = Set(v);
        }
        if let Some(v) = self.interest_earned {
            am.interest_earned = Set(v);
        }
        if let Some(v) = self.tax_deducted {
            am.tax_deducted = Set(v);
        }
        if let Some(v) = self.is_auto_renew {
            am.is_auto_renew = Set(v);
        }
        if let Some(v) = self.renewal_instructions {
            am.renewal_instructions = Set(Some(v));
        }
        if let Some(v) = self.nominee {
            am.nominee = Set(Some(v));
        }
        if let Some(v) = self.certificate_number {
            am.certificate_number = Set(Some(v));
        }
        if let Some(v) = self.status {
            am.status = Set(v);
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
pub struct FixedDepositResponse {
    pub id: String,
    pub user_id: i64,
    pub bank_id: Option<String>,
    pub name: String,
    pub fd_type: String,
    pub principal_amount: f64,
    pub interest_rate: f64,
    pub tenure_months: i64,
    pub start_date: String,
    pub maturity_date: String,
    pub compounding_frequency: String,
    pub current_value: f64,
    pub interest_earned: f64,
    pub tax_deducted: f64,
    pub is_auto_renew: bool,
    pub renewal_instructions: Option<String>,
    pub nominee: Option<String>,
    pub certificate_number: Option<String>,
    pub status: String,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub days_to_maturity: i64,
    pub projected_maturity_value: f64,
}

impl From<Model> for FixedDepositResponse {
    fn from(m: Model) -> Self {
        let today = chrono::Utc::now().date_naive();
        let days_to_maturity = chrono::NaiveDate::parse_from_str(&m.maturity_date, "%Y-%m-%d")
            .map(|d| (d - today).num_days())
            .unwrap_or(0);
        let years = m.tenure_months as f64 / 12.0;
        let projected_maturity_value =
            compound_value(m.principal_amount, m.interest_rate, years, &m.compounding_frequency);
        Self {
            id: m.id,
            user_id: m.user_id,
            bank_id: m.bank_id,
            name: m.name,
            fd_type: m.fd_type,
            principal_amount: m.principal_amount,
            interest_rate: m.interest_rate,
            tenure_months: m.tenure_months,
            start_date: m.start_date,
            maturity_date: m.maturity_date,
            compounding_frequency: m.compounding_frequency,
            current_value: m.current_value,
            interest_earned: m.interest_earned,
            tax_deducted: m.tax_deducted,
            is_auto_renew: m.is_auto_renew,
            renewal_instructions: m.renewal_instructions,
            nominee: m.nominee,
            certificate_number: m.certificate_number,
            status: m.status,
            sync_status: m.sync_status,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
            days_to_maturity,
            projected_maturity_value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedDepositSummary {
    pub total_invested: f64,
    pub total_value: f64,
    pub total_interest: f64,
    pub active_count: i64,
}
