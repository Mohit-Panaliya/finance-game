use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use validator::Validate;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "investments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i64,
    pub name: String,
    pub investment_type: String,
    pub instrument: String,
    pub symbol: Option<String>,
    pub invested_amount: f64,
    pub current_value: f64,
    pub units: Option<f64>,
    pub unit_price: Option<f64>,
    pub purchase_date: String,
    pub purchase_price: Option<f64>,
    pub broker_platform: Option<String>,
    pub account_ref: Option<String>,
    pub expected_return: Option<f64>,
    pub actual_return: Option<f64>,
    pub annualized_return: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub risk_level: String,
    pub is_liquid: bool,
    pub lock_in_until: Option<String>,
    pub tax_saving: bool,
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
pub struct CreateInvestmentRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub investment_type: String,
    pub instrument: String,
    pub symbol: Option<String>,
    pub invested_amount: f64,
    pub current_value: Option<f64>,
    pub units: Option<f64>,
    pub unit_price: Option<f64>,
    pub purchase_date: String,
    pub purchase_price: Option<f64>,
    pub broker_platform: Option<String>,
    pub account_ref: Option<String>,
    pub expected_return: Option<f64>,
    pub actual_return: Option<f64>,
    pub annualized_return: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub risk_level: Option<String>,
    pub is_liquid: Option<bool>,
    pub lock_in_until: Option<String>,
    pub tax_saving: Option<bool>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl CreateInvestmentRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            name: Set(self.name),
            investment_type: Set(self.investment_type),
            instrument: Set(self.instrument),
            symbol: Set(self.symbol),
            invested_amount: Set(self.invested_amount),
            current_value: Set(self.current_value.unwrap_or(0.0)),
            units: Set(self.units),
            unit_price: Set(self.unit_price),
            purchase_date: Set(self.purchase_date),
            purchase_price: Set(self.purchase_price),
            broker_platform: Set(self.broker_platform),
            account_ref: Set(self.account_ref),
            expected_return: Set(self.expected_return),
            actual_return: Set(self.actual_return),
            annualized_return: Set(self.annualized_return),
            dividend_yield: Set(self.dividend_yield),
            risk_level: Set(self.risk_level.unwrap_or_else(|| "moderate".to_string())),
            is_liquid: Set(self.is_liquid.unwrap_or(true)),
            lock_in_until: Set(self.lock_in_until),
            tax_saving: Set(self.tax_saving.unwrap_or(false)),
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
pub struct UpdateInvestmentRequest {
    pub name: Option<String>,
    pub investment_type: Option<String>,
    pub instrument: Option<String>,
    pub symbol: Option<String>,
    pub invested_amount: Option<f64>,
    pub current_value: Option<f64>,
    pub units: Option<f64>,
    pub unit_price: Option<f64>,
    pub purchase_date: Option<String>,
    pub purchase_price: Option<f64>,
    pub broker_platform: Option<String>,
    pub account_ref: Option<String>,
    pub expected_return: Option<f64>,
    pub actual_return: Option<f64>,
    pub annualized_return: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub risk_level: Option<String>,
    pub is_liquid: Option<bool>,
    pub lock_in_until: Option<String>,
    pub tax_saving: Option<bool>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl UpdateInvestmentRequest {
    pub fn apply(self, current: &Model) -> ActiveModel {
        let mut am = current.clone().into_active_model();
        if let Some(v) = self.name {
            am.name = Set(v);
        }
        if let Some(v) = self.investment_type {
            am.investment_type = Set(v);
        }
        if let Some(v) = self.instrument {
            am.instrument = Set(v);
        }
        if let Some(v) = self.symbol {
            am.symbol = Set(Some(v));
        }
        if let Some(v) = self.invested_amount {
            am.invested_amount = Set(v);
        }
        if let Some(v) = self.current_value {
            am.current_value = Set(v);
        }
        if let Some(v) = self.units {
            am.units = Set(Some(v));
        }
        if let Some(v) = self.unit_price {
            am.unit_price = Set(Some(v));
        }
        if let Some(v) = self.purchase_date {
            am.purchase_date = Set(v);
        }
        if let Some(v) = self.purchase_price {
            am.purchase_price = Set(Some(v));
        }
        if let Some(v) = self.broker_platform {
            am.broker_platform = Set(Some(v));
        }
        if let Some(v) = self.account_ref {
            am.account_ref = Set(Some(v));
        }
        if let Some(v) = self.expected_return {
            am.expected_return = Set(Some(v));
        }
        if let Some(v) = self.actual_return {
            am.actual_return = Set(Some(v));
        }
        if let Some(v) = self.annualized_return {
            am.annualized_return = Set(Some(v));
        }
        if let Some(v) = self.dividend_yield {
            am.dividend_yield = Set(Some(v));
        }
        if let Some(v) = self.risk_level {
            am.risk_level = Set(v);
        }
        if let Some(v) = self.is_liquid {
            am.is_liquid = Set(v);
        }
        if let Some(v) = self.lock_in_until {
            am.lock_in_until = Set(Some(v));
        }
        if let Some(v) = self.tax_saving {
            am.tax_saving = Set(v);
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
pub struct InvestmentResponse {
    pub id: String,
    pub user_id: i64,
    pub name: String,
    pub investment_type: String,
    pub instrument: String,
    pub symbol: Option<String>,
    pub invested_amount: f64,
    pub current_value: f64,
    pub units: Option<f64>,
    pub unit_price: Option<f64>,
    pub purchase_date: String,
    pub purchase_price: Option<f64>,
    pub broker_platform: Option<String>,
    pub account_ref: Option<String>,
    pub expected_return: Option<f64>,
    pub actual_return: Option<f64>,
    pub annualized_return: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub risk_level: String,
    pub is_liquid: bool,
    pub lock_in_until: Option<String>,
    pub tax_saving: bool,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub gain_loss: f64,
    pub gain_pct: Option<f64>,
    pub xirr_estimate: Option<f64>,
}

impl From<Model> for InvestmentResponse {
    fn from(m: Model) -> Self {
        let gain_loss = m.current_value - m.invested_amount;
        let gain_pct = if m.invested_amount > 0.0 {
            Some(gain_loss / m.invested_amount * 100.0)
        } else {
            None
        };
        let years = chrono::NaiveDate::parse_from_str(&m.purchase_date, "%Y-%m-%d")
            .ok()
            .map(|d| (chrono::Utc::now().date_naive() - d).num_days() as f64 / 365.0)
            .filter(|y| *y > 0.0);
        let xirr_estimate = match (years, gain_pct) {
            (Some(y), Some(p)) => Some(((1.0 + p / 100.0).powf(1.0 / y) - 1.0) * 100.0),
            _ => m.annualized_return,
        };
        Self {
            id: m.id,
            user_id: m.user_id,
            name: m.name,
            investment_type: m.investment_type,
            instrument: m.instrument,
            symbol: m.symbol,
            invested_amount: m.invested_amount,
            current_value: m.current_value,
            units: m.units,
            unit_price: m.unit_price,
            purchase_date: m.purchase_date,
            purchase_price: m.purchase_price,
            broker_platform: m.broker_platform,
            account_ref: m.account_ref,
            expected_return: m.expected_return,
            actual_return: m.actual_return,
            annualized_return: m.annualized_return,
            dividend_yield: m.dividend_yield,
            risk_level: m.risk_level,
            is_liquid: m.is_liquid,
            lock_in_until: m.lock_in_until,
            tax_saving: m.tax_saving,
            tags: m.tags,
            notes: m.notes,
            sync_status: m.sync_status,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
            gain_loss,
            gain_pct,
            xirr_estimate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentSummary {
    pub total_invested: f64,
    pub total_value: f64,
    pub total_gain_loss: f64,
    pub count: i64,
}
