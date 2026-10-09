use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};
use validator::Validate;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Money still outstanding on a debt row. Guarded so a hand-edited row where
/// `settled_amount` exceeds `amount` reports zero instead of a negative balance.
pub fn outstanding(amount: f64, settled_amount: f64) -> f64 {
    if !amount.is_finite() || !settled_amount.is_finite() {
        return 0.0;
    }
    (amount - settled_amount).max(0.0)
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "debts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i64,
    /// `lent` = the counterparty owes the user, `borrowed` = the user owes them.
    pub direction: String,
    pub counterparty: String,
    pub amount: f64,
    pub settled_amount: f64,
    pub currency: String,
    /// `loan`, `advance`, `shared` or `other`.
    pub kind: String,
    /// Annual interest percentage for a loan. Zero means "not a loan".
    pub interest_rate: f64,
    /// Monthly instalment for a loan. Zero means no instalment tracked.
    pub emi_amount: f64,
    /// Number of instalments agreed for a loan. Zero means not a loan.
    pub tenure_months: i64,
    /// Institution that lent or borrowed the money, distinct from `counterparty`.
    pub lender: Option<String>,
    /// The account this debt was booked against, when it is known.
    pub account_id: Option<String>,
    pub occurred_date: String,
    pub due_date: Option<String>,
    pub note: Option<String>,
    pub settled_date: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn is_lent(&self) -> bool {
        self.direction.eq_ignore_ascii_case("lent")
    }

    pub fn outstanding(&self) -> f64 {
        outstanding(self.amount, self.settled_amount)
    }

    pub fn is_settled(&self) -> bool {
        self.outstanding() <= f64::EPSILON
    }

    /// A borrowed row that carries any loan detail. Loan fields are only
    /// meaningful on money the user owes, so a lent row never counts as a loan
    /// even if stale loan values were left on it.
    pub fn is_loan(&self) -> bool {
        !self.is_lent()
            && (self.interest_rate > 0.0
                || self.emi_amount > 0.0
                || self.tenure_months > 0
                || self
                    .lender
                    .as_deref()
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDebtRequest {
    /// Defaults to `lent`; an unrecognised value is stored verbatim so a future
    /// client can add kinds without a migration.
    pub direction: Option<String>,
    #[validate(length(min = 1, message = "counterparty is required"))]
    pub counterparty: String,
    pub amount: Option<f64>,
    pub settled_amount: Option<f64>,
    pub currency: Option<String>,
    pub kind: Option<String>,
    pub interest_rate: Option<f64>,
    pub emi_amount: Option<f64>,
    pub tenure_months: Option<i64>,
    pub lender: Option<String>,
    pub account_id: Option<String>,
    pub occurred_date: Option<String>,
    pub due_date: Option<String>,
    pub note: Option<String>,
    pub settled_date: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl CreateDebtRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            direction: Set(self.direction.unwrap_or_else(|| "lent".to_string())),
            counterparty: Set(self.counterparty),
            amount: Set(self.amount.unwrap_or(0.0)),
            settled_amount: Set(self.settled_amount.unwrap_or(0.0)),
            currency: Set(self.currency.unwrap_or_else(|| "INR".to_string())),
            kind: Set(self.kind.unwrap_or_else(|| "loan".to_string())),
            interest_rate: Set(self.interest_rate.unwrap_or(0.0)),
            emi_amount: Set(self.emi_amount.unwrap_or(0.0)),
            tenure_months: Set(self.tenure_months.unwrap_or(0)),
            lender: Set(self.lender),
            account_id: Set(self.account_id),
            occurred_date: Set(self
                .occurred_date
                .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string())),
            due_date: Set(self.due_date),
            note: Set(self.note),
            settled_date: Set(self.settled_date),
            sync_status: Set(self.sync_status.unwrap_or_else(|| "synced".to_string())),
            last_synced_at: Set(self.last_synced_at),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateDebtRequest {
    pub direction: Option<String>,
    pub counterparty: Option<String>,
    pub amount: Option<f64>,
    pub settled_amount: Option<f64>,
    pub currency: Option<String>,
    pub kind: Option<String>,
    pub interest_rate: Option<f64>,
    pub emi_amount: Option<f64>,
    pub tenure_months: Option<i64>,
    pub lender: Option<String>,
    pub account_id: Option<String>,
    pub occurred_date: Option<String>,
    pub due_date: Option<String>,
    pub note: Option<String>,
    pub settled_date: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
}

impl UpdateDebtRequest {
    pub fn apply(self, current: &Model) -> ActiveModel {
        let mut am = current.clone().into_active_model();
        if let Some(v) = self.direction {
            am.direction = Set(v);
        }
        if let Some(v) = self.counterparty {
            am.counterparty = Set(v);
        }
        if let Some(v) = self.amount {
            am.amount = Set(v);
        }
        if let Some(v) = self.settled_amount {
            am.settled_amount = Set(v);
        }
        if let Some(v) = self.currency {
            am.currency = Set(v);
        }
        if let Some(v) = self.kind {
            am.kind = Set(v);
        }
        if let Some(v) = self.interest_rate {
            am.interest_rate = Set(v);
        }
        if let Some(v) = self.emi_amount {
            am.emi_amount = Set(v);
        }
        if let Some(v) = self.tenure_months {
            am.tenure_months = Set(v);
        }
        if let Some(v) = self.lender {
            am.lender = Set(Some(v));
        }
        if let Some(v) = self.account_id {
            am.account_id = Set(Some(v));
        }
        if let Some(v) = self.occurred_date {
            am.occurred_date = Set(v);
        }
        if let Some(v) = self.due_date {
            am.due_date = Set(Some(v));
        }
        if let Some(v) = self.note {
            am.note = Set(Some(v));
        }
        if let Some(v) = self.settled_date {
            am.settled_date = Set(Some(v));
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
pub struct DebtResponse {
    pub id: String,
    pub user_id: i64,
    pub direction: String,
    pub counterparty: String,
    pub amount: f64,
    pub settled_amount: f64,
    /// `amount - settled_amount`, never negative.
    pub outstanding: f64,
    pub currency: String,
    pub kind: String,
    pub interest_rate: f64,
    pub emi_amount: f64,
    pub tenure_months: i64,
    pub lender: Option<String>,
    /// True when this borrowed row carries any loan detail.
    pub is_loan: bool,
    pub account_id: Option<String>,
    pub occurred_date: String,
    pub due_date: Option<String>,
    pub note: Option<String>,
    pub settled_date: Option<String>,
    pub is_settled: bool,
    /// Negative once the due date has passed and money is still outstanding.
    pub days_to_due: Option<i64>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn days_between(date: &str, today: chrono::NaiveDate) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .map(|d| (d - today).num_days())
}

impl From<Model> for DebtResponse {
    fn from(m: Model) -> Self {
        let today = chrono::Utc::now().date_naive();
        let open = m.outstanding();
        let is_loan = m.is_loan();
        Self {
            outstanding: open,
            is_settled: open <= f64::EPSILON,
            days_to_due: m.due_date.as_deref().and_then(|d| days_between(d, today)),
            id: m.id,
            user_id: m.user_id,
            direction: m.direction,
            counterparty: m.counterparty,
            amount: m.amount,
            settled_amount: m.settled_amount,
            currency: m.currency,
            kind: m.kind,
            interest_rate: m.interest_rate,
            emi_amount: m.emi_amount,
            tenure_months: m.tenure_months,
            lender: m.lender,
            is_loan,
            account_id: m.account_id,
            occurred_date: m.occurred_date,
            due_date: m.due_date,
            note: m.note,
            settled_date: m.settled_date,
            sync_status: m.sync_status,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// Aggregate view of the two-sided book. `owed_to_me` is what other people owe
/// the user and `i_owe` is what the user owes them; `net` is the difference,
/// which is the only figure worth acting on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtSummary {
    pub owed_to_me: f64,
    pub i_owe: f64,
    pub net: f64,
    /// Every outstanding amount the user owes, loans included. Same figure as
    /// `i_owe`, surfaced under a name the liabilities view reads directly.
    pub total_owed: f64,
    /// Monthly instalments across all still-open borrowed loans.
    pub monthly_emi_outflow: f64,
    /// How many borrowed rows carry loan detail.
    pub loan_count: i64,
    pub lent_count: i64,
    pub borrowed_count: i64,
    pub settled_count: i64,
    pub overdue_count: i64,
    pub total_count: i64,
}

impl DebtSummary {
    pub fn from_rows(rows: &[Model], today: chrono::NaiveDate) -> Self {
        let mut owed_to_me = 0.0;
        let mut i_owe = 0.0;
        let mut lent_count = 0;
        let mut borrowed_count = 0;
        let mut settled_count = 0;
        let mut overdue_count = 0;
        let mut monthly_emi_outflow = 0.0;
        let mut loan_count = 0;

        for row in rows {
            let open = row.outstanding();
            if row.is_settled() {
                settled_count += 1;
            }
            if row.is_lent() {
                lent_count += 1;
                owed_to_me += open;
            } else {
                borrowed_count += 1;
                i_owe += open;
                if row.is_loan() {
                    loan_count += 1;
                    if open > f64::EPSILON {
                        monthly_emi_outflow += row.emi_amount;
                    }
                }
            }
            if open > f64::EPSILON {
                if let Some(days) = row.due_date.as_deref().and_then(|d| days_between(d, today)) {
                    if days < 0 {
                        overdue_count += 1;
                    }
                }
            }
        }

        Self {
            owed_to_me,
            i_owe,
            net: owed_to_me - i_owe,
            total_owed: i_owe,
            monthly_emi_outflow,
            loan_count,
            lent_count,
            borrowed_count,
            settled_count,
            overdue_count,
            total_count: rows.len() as i64,
        }
    }
}
