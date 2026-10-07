export type EntityType =
  | 'banks'
  | 'assets'
  | 'expenses'
  | 'credit-cards'
  | 'fixed-deposits'
  | 'investments'
  | 'incomes'

/** The 5 entity groups that own real money (credit cards are liabilities). */
export type AccountEntityType = Extract<EntityType, 'banks' | 'assets' | 'credit-cards' | 'fixed-deposits' | 'investments'>

export interface FinanceRow {
  id?: number | string
  [key: string]: unknown
}

/** Shape returned by every `GET /api/{entity}` list endpoint. */
export interface Paged<T> {
  data: T[]
  total?: number
  page?: number
  perPage?: number
}

/**
 * One line of `GET /api/banks/{id}/statement` or `GET /api/credit-cards/{id}/statement`.
 * Newest first; `balance_after` is walked backwards from the account's stored balance.
 */
export interface StatementEntry {
  entry_type: 'income' | 'expense' | 'asset' | 'investment'
  id: string
  title: string
  amount: number
  signed_amount: number
  occurred_on: string
  balance_after: number
}

export interface User {
  id?: number | string
  email?: string
  name?: string
  username?: string
}

/**
 * One queued mutation. `entity` must be the snake_case table name
 * (`credit_cards`, `fixed_deposits`, …) and `entity_id` is required by the server.
 */
export interface SyncOp {
  id: string
  entity: string
  entity_id: string
  op: 'create' | 'update' | 'delete'
  payload: unknown
  client_ts: string
}

/* ------------------------------------------------------------------ *
 * GET /api/analysis
 * ------------------------------------------------------------------ */

export interface NetWorthBreakdown {
  banks: number
  assets: number
  fixed_deposits: number
  investments: number
  total: number
}

export interface YearlyIncomeByType {
  salary: number
  business: number
  investment: number
  other: number
  total: number
}

export interface MonthlyExpenseByCategory {
  month: string
  category: string
  amount: number
}

export interface InvestmentTypeRoi {
  investment_type: string
  invested: number
  current_value: number
  gain_loss: number
  roi_pct: number
}

export interface RoiCalculation {
  total_invested: number
  total_current_value: number
  total_gain_loss: number
  roi_percentage: number
  by_type: InvestmentTypeRoi[]
}

export interface CashFlowPoint {
  month: string
  income: number
  expense: number
  net: number
}

export interface TopItem {
  label: string
  amount: number
  category: string | null
}

export interface FdMaturity {
  id: string
  name: string
  principal: number
  current_value: number
  maturity_date: string
  days_to_maturity: number
  interest_rate: number
}

export interface CreditCardUtilization {
  card_name: string
  limit: number
  balance: number
  available: number
  utilization_pct: number
}

export interface SavingsRate {
  total_income: number
  total_expense: number
  savings: number
  savings_rate_pct: number
}

export interface AnalysisResponse {
  net_worth: NetWorthBreakdown
  yearly_income_by_type: YearlyIncomeByType
  monthly_expenses: MonthlyExpenseByCategory[]
  roi: RoiCalculation
  cash_flow: CashFlowPoint[]
  top_5_expenses: TopItem[]
  top_5_income_sources: TopItem[]
  investment_performance: InvestmentTypeRoi[]
  fd_maturity_timeline: FdMaturity[]
  credit_card_utilization: CreditCardUtilization[]
  savings_rate: SavingsRate
  calendar: DayFlow[]
  account_attribution: AccountFlow[]
  expense_categories: CategoryTotal[]
  debts: DebtSummary
}

/* ------------------------------------------------------------------ *
 * GET /api/overview
 * ------------------------------------------------------------------ */

export interface OverviewResponse {
  net_worth: number
  yearly_income: number
  monthly_expense: number
  total_gain: number
  invested: number
  roi: number
}

/* ------------------------------------------------------------------ *
 * GET /api/{entity}/summary
 * ------------------------------------------------------------------ */

export interface BankSummary {
  total_balance: number
  count: number
}

export interface AssetSummary {
  total_value: number
  total_invested: number
  total_gain_loss: number
  count: number
}

export interface CreditCardSummary {
  total_limit: number
  total_balance: number
  total_available: number
  count: number
}

export interface FixedDepositSummary {
  total_invested: number
  total_value: number
  total_interest: number
  active_count: number
}

export interface InvestmentSummary {
  total_invested: number
  total_value: number
  total_gain_loss: number
  count: number
}

export interface IncomeTypeTotal {
  income_type: string
  total: number
}

export interface IncomeSummary {
  yearly_income: number
  monthly_income: number
  received_this_year: number
  by_type: IncomeTypeTotal[]
}

export interface ExpenseCategoryTotal {
  category: string
  total: number
  count: number
}

export interface ExpenseMonthTotal {
  month: string
  total: number
}

export interface ExpenseSummary {
  total_amount: number
  fixed_total: number
  variable_total: number
  by_category: ExpenseCategoryTotal[]
  by_month: ExpenseMonthTotal[]
}

export interface EntitySummaryMap {
  banks: BankSummary
  assets: AssetSummary
  'credit-cards': CreditCardSummary
  'fixed-deposits': FixedDepositSummary
  investments: InvestmentSummary
  incomes: IncomeSummary
  expenses: ExpenseSummary
}

/** Result of `POST /api/sync/push`. */
export interface SyncPushResponse {
  ok?: boolean
  results?: Array<{
    entity: string
    entity_id: string
    status: string
    server_ts: string
  }>
}

/** Result of `GET /api/sync/pull`. */
export interface SyncPullResponse {
  changes?: Array<{
    entity: string
    entity_id: string
    op: string
    payload: string | null
    server_ts: string | null
  }>
  server_ts?: string
}
/** One row of `GET /api/debts`. */
export interface DebtRow extends FinanceRow {
  direction: string
  counterparty: string
  amount: number
  settled_amount: number
  outstanding: number
  currency: string
  kind: string
  account_id: string | null
  occurred_date: string
  due_date: string | null
  note: string | null
  settled_date: string | null
  is_settled: boolean
  days_to_due: number | null
}

/** Aggregate two-sided debt book. */
export interface DebtSummary {
  owed_to_me: number
  i_owe: number
  net: number
  lent_count: number
  borrowed_count: number
  settled_count: number
  overdue_count: number
  total_count: number
}

/** One calendar cell: totals for a single `YYYY-MM-DD`. */
export interface DayFlow {
  date: string
  income: number
  expense: number
  net: number
}

/** Which account a rupee moved through; unattributed rows share one bucket. */
export interface AccountFlow {
  account_id: string | null
  account_name: string
  income: number
  expense: number
  net: number
  count: number
}

export interface CategoryTotal {
  category: string
  total: number
  count: number
}
