use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::models::{
    assets, banks, credit_cards, expenses, fixed_deposits, incomes, investments,
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/analysis", get(analysis))
        .add("/overview", get(stats))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewStats {
    pub net_worth: f64,
    pub yearly_income: f64,
    pub monthly_expense: f64,
    pub total_gain: f64,
    pub invested: f64,
    pub roi: f64,
}

async fn overview_stats(
    db: &sea_orm_turso::TursoConnection,
    user_id: i64,
) -> Result<OverviewStats> {
    let banks = banks::Entity::find()
        .filter(banks::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let assets = assets::Entity::find()
        .filter(assets::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let fds = fixed_deposits::Entity::find()
        .filter(fixed_deposits::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let investments = investments::Entity::find()
        .filter(investments::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let expenses = expenses::Entity::find()
        .filter(expenses::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let incomes = incomes::Entity::find()
        .filter(incomes::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    let bank_balance: f64 = banks.iter().map(|b| b.current_balance).sum();
    let asset_value: f64 = assets.iter().map(|a| a.current_value).sum();
    let fd_value: f64 = fds.iter().map(|f| f.current_value).sum();
    let inv_value: f64 = investments.iter().map(|i| i.current_value).sum();
    let invested = bank_balance + asset_value + fd_value + inv_value;
    let total_income: f64 = incomes.iter().map(|i| i.amount).sum();
    let total_expense: f64 = expenses.iter().map(|e| e.amount).sum();
    let net_worth = invested;
    let total_gain = total_income - total_expense;
    let roi = if invested > 0.0 { (total_gain / invested) * 100.0 } else { 0.0 };
    let monthly_expense = total_expense / 12.0;

    Ok(OverviewStats {
        net_worth,
        yearly_income: total_income,
        monthly_expense,
        total_gain,
        invested,
        roi,
    })
}

#[debug_handler]
async fn stats(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = super::uid(&ctx, &auth).await?;
    let s = overview_stats(&*db, user_id).await?;
    format::json(s)
}

// ============ ANALYSIS ENDPOINT ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    /// comma-separated, e.g. "housing,food" (axum query can't deserialize seqs)
    pub categories: Option<String>,
    pub entity_types: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorthBreakdown {
    pub banks: f64,
    pub assets: f64,
    pub fixed_deposits: f64,
    pub investments: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyIncomeByType {
    pub salary: f64,
    pub business: f64,
    pub investment: f64,
    pub other: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyExpenseByCategory {
    pub month: String,
    pub category: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROICalculation {
    pub total_invested: f64,
    pub total_current_value: f64,
    pub total_gain_loss: f64,
    pub roi_percentage: f64,
    pub by_type: Vec<InvestmentTypeROI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentTypeROI {
    pub investment_type: String,
    pub invested: f64,
    pub current_value: f64,
    pub gain_loss: f64,
    pub roi_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowPoint {
    pub month: String,
    pub income: f64,
    pub expense: f64,
    pub net: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopItem {
    pub label: String,
    pub amount: f64,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FDMaturity {
    pub id: String,
    pub name: String,
    pub principal: f64,
    pub current_value: f64,
    pub maturity_date: String,
    pub days_to_maturity: i64,
    pub interest_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCardUtilization {
    pub card_name: String,
    pub limit: f64,
    pub balance: f64,
    pub available: f64,
    pub utilization_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsRate {
    pub total_income: f64,
    pub total_expense: f64,
    pub savings: f64,
    pub savings_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub net_worth: NetWorthBreakdown,
    pub yearly_income_by_type: YearlyIncomeByType,
    pub monthly_expenses: Vec<MonthlyExpenseByCategory>,
    pub roi: ROICalculation,
    pub cash_flow: Vec<CashFlowPoint>,
    pub top_5_expenses: Vec<TopItem>,
    pub top_5_income_sources: Vec<TopItem>,
    pub investment_performance: Vec<InvestmentTypeROI>,
    pub fd_maturity_timeline: Vec<FDMaturity>,
    pub credit_card_utilization: Vec<CreditCardUtilization>,
    pub savings_rate: SavingsRate,
}

fn month_key(date: &str) -> String {
    if date.len() >= 7 {
        date[..7].to_string()
    } else {
        "".to_string()
    }
}

async fn fetch_incomes(
    db: &sea_orm_turso::TursoConnection,
    user_id: i64,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<incomes::Model>> {
    let mut q = incomes::Entity::find().filter(incomes::Column::UserId.eq(user_id));
    if let Some(s) = start {
        q = q.filter(incomes::Column::IncomeDate.gte(s));
    }
    if let Some(e) = end {
        q = q.filter(incomes::Column::IncomeDate.lte(e));
    }
    q.all(db).await.map_err(|e| Error::string(&e.to_string()))
}

async fn fetch_expenses(
    db: &sea_orm_turso::TursoConnection,
    user_id: i64,
    start: Option<&str>,
    end: Option<&str>,
    categories: Option<&Vec<String>>,
) -> Result<Vec<expenses::Model>> {
    let mut q = expenses::Entity::find().filter(expenses::Column::UserId.eq(user_id));
    if let Some(s) = start {
        q = q.filter(expenses::Column::ExpenseDate.gte(s));
    }
    if let Some(e) = end {
        q = q.filter(expenses::Column::ExpenseDate.lte(e));
    }
    if let Some(cats) = categories {
        if !cats.is_empty() {
            q = q.filter(expenses::Column::Category.is_in(cats.clone()));
        }
    }
    q.all(db).await.map_err(|e| Error::string(&e.to_string()))
}

#[debug_handler]
async fn analysis(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<AnalysisRequest>,
) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = super::uid(&ctx, &auth).await?;

    let start = params.start_date.as_deref();
    let end = params.end_date.as_deref();
    let cats: Option<Vec<String>> = match params.categories.as_deref() {
        Some(raw) => {
            let v: Vec<String> = raw
                .split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(str::to_string)
                .collect();
            if v.is_empty() { None } else { Some(v) }
        }
        None => None,
    };

    // Fetch all data in parallel
    let (banks_list, assets_list, fds_list, investments_list, incomes_list, expenses_list, credit_cards_list) = tokio::join!(
        banks::Entity::find().filter(banks::Column::UserId.eq(user_id)).all(&*db),
        assets::Entity::find().filter(assets::Column::UserId.eq(user_id)).all(&*db),
        fixed_deposits::Entity::find().filter(fixed_deposits::Column::UserId.eq(user_id)).all(&*db),
        investments::Entity::find().filter(investments::Column::UserId.eq(user_id)).all(&*db),
        fetch_incomes(&*db, user_id, start, end),
        fetch_expenses(&*db, user_id, start, end, cats.as_ref()),
        credit_cards::Entity::find().filter(credit_cards::Column::UserId.eq(user_id)).all(&*db),
    );

    let banks = banks_list?;
    let assets = assets_list?;
    let fds = fds_list?;
    let investments = investments_list?;
    let incomes = incomes_list?;
    let expenses = expenses_list?;
    let credit_cards = credit_cards_list?;

    // Net worth breakdown (finance tables only)
    let bank_balance: f64 = banks.iter().map(|b| b.current_balance).sum();
    let asset_value: f64 = assets.iter().map(|a| a.current_value).sum();
    let fd_value: f64 = fds.iter().map(|f| f.current_value).sum();
    let inv_value: f64 = investments.iter().map(|i| i.current_value).sum();
    let net_worth = NetWorthBreakdown {
        banks: bank_balance,
        assets: asset_value,
        fixed_deposits: fd_value,
        investments: inv_value,
        total: bank_balance + asset_value + fd_value + inv_value,
    };

    // Yearly income by type
    let mut salary = 0.0;
    let mut business = 0.0;
    let mut investment_income = 0.0;
    let mut other_income = 0.0;
    for inc in &incomes {
        let annualized = if inc.is_recurring {
            match inc.recurrence.as_deref() {
                Some("weekly") => inc.amount * inc.frequency_multiplier as f64 * 52.0,
                Some("biweekly") | Some("fortnightly") => inc.amount * inc.frequency_multiplier as f64 * 26.0,
                Some("monthly") => inc.amount * inc.frequency_multiplier as f64 * 12.0,
                Some("quarterly") => inc.amount * inc.frequency_multiplier as f64 * 4.0,
                Some("yearly") | Some("annually") => inc.amount * inc.frequency_multiplier as f64,
                _ => inc.amount * inc.frequency_multiplier.max(1) as f64,
            }
        } else {
            inc.amount * inc.frequency_multiplier.max(1) as f64
        };
        match inc.income_type.to_lowercase().as_str() {
            "salary" => salary += annualized,
            "business" => business += annualized,
            "investment" => investment_income += annualized,
            _ => other_income += annualized,
        }
    }
    let yearly_income_by_type = YearlyIncomeByType {
        salary,
        business,
        investment: investment_income,
        other: other_income,
        total: salary + business + investment_income + other_income,
    };

    // Monthly expense by category
    let mut monthly_expense_map: std::collections::HashMap<String, std::collections::HashMap<String, f64>> = std::collections::HashMap::new();
    for exp in &expenses {
        let month = month_key(&exp.expense_date);
        if month.is_empty() { continue; }
        let entry = monthly_expense_map.entry(month).or_default();
        *entry.entry(exp.category.clone()).or_default() += exp.amount;
    }
    let mut monthly_expenses: Vec<MonthlyExpenseByCategory> = Vec::new();
    for (month, cats) in monthly_expense_map {
        for (category, amount) in cats {
            monthly_expenses.push(MonthlyExpenseByCategory { month: month.clone(), category, amount });
        }
    }
    monthly_expenses.sort_by(|a, b| a.month.cmp(&b.month).then(a.category.cmp(&b.category)));

    // ROI calculations
    let total_invested: f64 = investments.iter().map(|i| i.invested_amount).sum();
    let total_current_value: f64 = investments.iter().map(|i| i.current_value).sum();
    let total_gain_loss = total_current_value - total_invested;
    let roi_percentage = if total_invested > 0.0 { (total_gain_loss / total_invested) * 100.0 } else { 0.0 };

    let mut by_type_map: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
    for inv in &investments {
        let entry = by_type_map.entry(inv.investment_type.clone()).or_default();
        entry.0 += inv.invested_amount;
        entry.1 += inv.current_value;
    }
    let by_type: Vec<InvestmentTypeROI> = by_type_map
        .into_iter()
        .map(|(investment_type, (invested, current_value))| {
            let gain_loss = current_value - invested;
            let roi_pct = if invested > 0.0 { (gain_loss / invested) * 100.0 } else { 0.0 };
            InvestmentTypeROI { investment_type, invested, current_value, gain_loss, roi_pct }
        })
        .collect();

    let roi = ROICalculation {
        total_invested,
        total_current_value,
        total_gain_loss,
        roi_percentage,
        by_type,
    };

    // Cash flow monthly trend
    let mut income_by_month: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut expense_by_month: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for inc in &incomes {
        let month = month_key(&inc.income_date);
        if !month.is_empty() {
            let net = inc.amount - inc.tax_withheld;
            *income_by_month.entry(month).or_default() += net;
        }
    }
    for exp in &expenses {
        let month = month_key(&exp.expense_date);
        if !month.is_empty() {
            *expense_by_month.entry(month).or_default() += exp.amount;
        }
    }
    let mut all_months: std::collections::HashSet<String> = std::collections::HashSet::new();
    all_months.extend(income_by_month.keys().cloned());
    all_months.extend(expense_by_month.keys().cloned());
    let mut cash_flow: Vec<CashFlowPoint> = all_months
        .into_iter()
        .map(|month| {
            let income = income_by_month.get(&month).copied().unwrap_or(0.0);
            let expense = expense_by_month.get(&month).copied().unwrap_or(0.0);
            CashFlowPoint { month, income, expense, net: income - expense }
        })
        .collect();
    cash_flow.sort_by(|a, b| a.month.cmp(&b.month));

    // Top 5 expenses
    let mut expense_by_title: std::collections::HashMap<String, (f64, String)> = std::collections::HashMap::new();
    for exp in &expenses {
        let e = expense_by_title.entry(exp.title.clone()).or_default(); e.0 += exp.amount;
        expense_by_title.get_mut(&exp.title).unwrap().1 = exp.category.clone();
    }
    let mut top_expenses: Vec<_> = expense_by_title
        .into_iter()
        .map(|(label, (amount, category))| TopItem { label, amount, category: Some(category) })
        .collect();
    top_expenses.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
    let top_5_expenses = top_expenses.into_iter().take(5).collect();

    // Top 5 income sources
    let mut income_by_source: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for inc in &incomes {
        *income_by_source.entry(inc.source.clone()).or_default() += inc.amount - inc.tax_withheld;
    }
    let mut top_income: Vec<_> = income_by_source
        .into_iter()
        .map(|(label, amount)| TopItem { label, amount, category: None })
        .collect();
    top_income.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
    let top_5_income_sources = top_income.into_iter().take(5).collect();

    // Investment performance (gain/loss by type)
    let investment_performance = roi.by_type.clone();

    // FD maturity timeline
    let fd_maturity_timeline: Vec<FDMaturity> = fds
        .into_iter()
        .map(|f| {
            let today = chrono::Utc::now().date_naive();
            let days_to_maturity = chrono::NaiveDate::parse_from_str(&f.maturity_date, "%Y-%m-%d")
                .map(|d| (d - today).num_days())
                .unwrap_or(0);
            FDMaturity {
                id: f.id,
                name: f.name,
                principal: f.principal_amount,
                current_value: f.current_value,
                maturity_date: f.maturity_date,
                days_to_maturity,
                interest_rate: f.interest_rate,
            }
        })
        .collect();

    // Credit card utilization
    let credit_card_utilization: Vec<CreditCardUtilization> = credit_cards
        .into_iter()
        .map(|c| {
            let utilization_pct = if c.credit_limit > 0.0 {
                (c.current_balance / c.credit_limit) * 100.0
            } else { 0.0 };
            CreditCardUtilization {
                card_name: c.name,
                limit: c.credit_limit,
                balance: c.current_balance,
                available: c.available_credit,
                utilization_pct,
            }
        })
        .collect();

    // Savings rate
    let total_income: f64 = incomes.iter().map(|i| i.amount - i.tax_withheld).sum();
    let total_expense: f64 = expenses.iter().map(|e| e.amount).sum();
    let savings = total_income - total_expense;
    let savings_rate_pct = if total_income > 0.0 { (savings / total_income) * 100.0 } else { 0.0 };
    let savings_rate = SavingsRate {
        total_income,
        total_expense,
        savings,
        savings_rate_pct,
    };

    let response = AnalysisResponse {
        net_worth,
        yearly_income_by_type,
        monthly_expenses,
        roi,
        cash_flow,
        top_5_expenses,
        top_5_income_sources,
        investment_performance,
        fd_maturity_timeline,
        credit_card_utilization,
        savings_rate,
    };

    format::json(response)
}
