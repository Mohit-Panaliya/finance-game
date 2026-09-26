use chrono::Datelike;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn days_until_day(day_of_month: i64) -> i64 {
    let today = chrono::Utc::now().date_naive();
    let target = today
        .with_day(day_of_month.clamp(1, 28) as u32)
        .unwrap_or(today);
    if target >= today {
        (target - today).num_days()
    } else {
        let next_month = target
            .with_month(target.month() % 12 + 1)
            .and_then(|d| if target.month() == 12 { d.with_year(target.year() + 1) } else { Some(d) })
            .unwrap_or(target);
        (next_month - today).num_days()
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "credit_cards")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i64,
    pub name: String,
    pub bank_name: String,
    pub card_type: String,
    pub last_four_digits: String,
    pub credit_limit: f64,
    pub current_balance: f64,
    pub available_credit: f64,
    pub interest_rate: f64,
    pub billing_cycle_day: i64,
    pub due_date_day: i64,
    pub annual_fee: f64,
    pub reward_program: Option<String>,
    pub reward_points: i64,
    pub is_active: bool,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub game_building_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCreditCardRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub bank_name: String,
    pub card_type: String,
    pub last_four_digits: String,
    pub credit_limit: Option<f64>,
    pub current_balance: Option<f64>,
    pub available_credit: Option<f64>,
    pub interest_rate: Option<f64>,
    pub billing_cycle_day: Option<i64>,
    pub due_date_day: Option<i64>,
    pub annual_fee: Option<f64>,
    pub reward_program: Option<String>,
    pub reward_points: Option<i64>,
    pub is_active: Option<bool>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
    pub game_building_id: Option<String>,
}

impl CreateCreditCardRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        let credit_limit = self.credit_limit.unwrap_or(0.0);
        let current_balance = self.current_balance.unwrap_or(0.0);
        let available_credit = self
            .available_credit
            .unwrap_or_else(|| (credit_limit - current_balance).max(0.0));
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            name: Set(self.name),
            bank_name: Set(self.bank_name),
            card_type: Set(self.card_type),
            last_four_digits: Set(self.last_four_digits),
            credit_limit: Set(credit_limit),
            current_balance: Set(current_balance),
            available_credit: Set(available_credit),
            interest_rate: Set(self.interest_rate.unwrap_or(0.0)),
            billing_cycle_day: Set(self.billing_cycle_day.unwrap_or(1)),
            due_date_day: Set(self.due_date_day.unwrap_or(5)),
            annual_fee: Set(self.annual_fee.unwrap_or(0.0)),
            reward_program: Set(self.reward_program),
            reward_points: Set(self.reward_points.unwrap_or(0)),
            is_active: Set(self.is_active.unwrap_or(true)),
            sync_status: Set(self.sync_status.unwrap_or_else(|| "synced".to_string())),
            last_synced_at: Set(self.last_synced_at),
            game_building_id: Set(self.game_building_id),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCreditCardRequest {
    pub name: Option<String>,
    pub bank_name: Option<String>,
    pub card_type: Option<String>,
    pub last_four_digits: Option<String>,
    pub credit_limit: Option<f64>,
    pub current_balance: Option<f64>,
    pub available_credit: Option<f64>,
    pub interest_rate: Option<f64>,
    pub billing_cycle_day: Option<i64>,
    pub due_date_day: Option<i64>,
    pub annual_fee: Option<f64>,
    pub reward_program: Option<String>,
    pub reward_points: Option<i64>,
    pub is_active: Option<bool>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
    pub game_building_id: Option<String>,
}

impl UpdateCreditCardRequest {
    pub fn apply(self, current: &Model) -> ActiveModel {
        let mut am = current.clone().into_active_model();
        if let Some(v) = self.name {
            am.name = Set(v);
        }
        if let Some(v) = self.bank_name {
            am.bank_name = Set(v);
        }
        if let Some(v) = self.card_type {
            am.card_type = Set(v);
        }
        if let Some(v) = self.last_four_digits {
            am.last_four_digits = Set(v);
        }
        if let Some(v) = self.credit_limit {
            am.credit_limit = Set(v);
        }
        if let Some(v) = self.current_balance {
            am.current_balance = Set(v);
        }
        if let Some(v) = self.available_credit {
            am.available_credit = Set(v);
        }
        if let Some(v) = self.interest_rate {
            am.interest_rate = Set(v);
        }
        if let Some(v) = self.billing_cycle_day {
            am.billing_cycle_day = Set(v);
        }
        if let Some(v) = self.due_date_day {
            am.due_date_day = Set(v);
        }
        if let Some(v) = self.annual_fee {
            am.annual_fee = Set(v);
        }
        if let Some(v) = self.reward_program {
            am.reward_program = Set(Some(v));
        }
        if let Some(v) = self.reward_points {
            am.reward_points = Set(v);
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
        if let Some(v) = self.game_building_id {
            am.game_building_id = Set(Some(v));
        }
        am.updated_at = Set(Some(now()));
        am
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCardResponse {
    pub id: String,
    pub user_id: i64,
    pub name: String,
    pub bank_name: String,
    pub card_type: String,
    pub last_four_digits: String,
    pub credit_limit: f64,
    pub current_balance: f64,
    pub available_credit: f64,
    pub interest_rate: f64,
    pub billing_cycle_day: i64,
    pub due_date_day: i64,
    pub annual_fee: f64,
    pub reward_program: Option<String>,
    pub reward_points: i64,
    pub is_active: bool,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub game_building_id: Option<String>,
    pub utilization_pct: Option<f64>,
    pub days_until_due: i64,
    pub estimated_monthly_interest: f64,
}

impl From<Model> for CreditCardResponse {
    fn from(m: Model) -> Self {
        let utilization_pct = if m.credit_limit > 0.0 {
            Some(m.current_balance / m.credit_limit * 100.0)
        } else {
            None
        };
        let days_until_due = days_until_day(m.due_date_day);
        let estimated_monthly_interest = m.current_balance * (m.interest_rate / 100.0) / 12.0;
        Self {
            id: m.id,
            user_id: m.user_id,
            name: m.name,
            bank_name: m.bank_name,
            card_type: m.card_type,
            last_four_digits: m.last_four_digits,
            credit_limit: m.credit_limit,
            current_balance: m.current_balance,
            available_credit: m.available_credit,
            interest_rate: m.interest_rate,
            billing_cycle_day: m.billing_cycle_day,
            due_date_day: m.due_date_day,
            annual_fee: m.annual_fee,
            reward_program: m.reward_program,
            reward_points: m.reward_points,
            is_active: m.is_active,
            sync_status: m.sync_status,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
            game_building_id: m.game_building_id,
            utilization_pct,
            days_until_due,
            estimated_monthly_interest,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCardSummary {
    pub total_limit: f64,
    pub total_balance: f64,
    pub total_available: f64,
    pub count: i64,
}
