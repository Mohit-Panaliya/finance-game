import {
  businessOutline,
  cardOutline,
  cashOutline,
  pieChartOutline,
  pricetagOutline,
  statsChartOutline,
  walletOutline
} from 'ionicons/icons'
import type { EntityType, EntitySummaryMap, FinanceRow } from '@/types'

export type FieldKind = 'text' | 'number' | 'date' | 'select' | 'money' | 'bool'

export interface FieldDef {
  /** Backend column name. This is sent verbatim in create/update payloads. */
  key: string
  label: string
  kind: FieldKind
  options?: Array<{ value: string; label: string }>
  /** Options resolved at render time from the user's own rows. */
  dynamic?: 'accounts' | 'cards'
  placeholder?: string
  /** Non-Option field on the backend `Create*Request` → must be sent, or the POST 422s. */
  required?: boolean
  /** Rendered after the input as a small hint. */
  hint?: string
  suffix?: string
}

export interface EntityConfig {
  entity: EntityType
  label: string
  singular: string
  icon: string
  accent: string
  /** Ordered fallback keys for "the money in this row". */
  valueKeys: string[]
  /** Primary/secondary columns shown in a list row. */
  displayKeys: [string, string]
  /** Column rendered as the row subtitle (date-ish), if any. */
  subKeys: string[]
  /** Summary field that carries the headline total for this entity. */
  summaryTotalKey: string
  fields: FieldDef[]
}

const CURRENCY_OPTIONS = [
  { value: 'INR', label: 'INR — Indian Rupee' },
  { value: 'USD', label: 'USD — US Dollar' },
  { value: 'EUR', label: 'EUR — Euro' },
  { value: 'GBP', label: 'GBP — Pound Sterling' },
  { value: 'AED', label: 'AED — UAE Dirham' },
  { value: 'SGD', label: 'SGD — Singapore Dollar' },
  { value: 'AUD', label: 'AUD — Australian Dollar' },
  { value: 'CAD', label: 'CAD — Canadian Dollar' },
  { value: 'JPY', label: 'JPY — Japanese Yen' }
]

const RECURRENCE_OPTIONS = [
  { value: 'weekly', label: 'Weekly' },
  { value: 'biweekly', label: 'Fortnightly' },
  { value: 'monthly', label: 'Monthly' },
  { value: 'quarterly', label: 'Quarterly' },
  { value: 'yearly', label: 'Yearly' }
]

const BOOL_OPTIONS = [
  { value: 'true', label: 'Yes' },
  { value: 'false', label: 'No' }
]

const RISK_OPTIONS = [
  { value: 'low', label: 'Low' },
  { value: 'moderate', label: 'Moderate' },
  { value: 'high', label: 'High' }
]

export const ENTITY_CONFIGS: Record<EntityType, EntityConfig> = {
  banks: {
    entity: 'banks',
    label: 'Banks',
    singular: 'Bank account',
    icon: walletOutline,
    accent: '#2f6fd0',
    valueKeys: ['current_balance'],
    displayKeys: ['name', 'current_balance'],
    subKeys: ['bank_type', 'account_number'],
    summaryTotalKey: 'total_balance',
    fields: [
      { key: 'name', label: 'Account name', kind: 'text', required: true, placeholder: 'e.g. Primary savings' },
      {
        key: 'bank_type',
        label: 'Account type',
        kind: 'select',
        required: true,
        options: [
          { value: 'savings', label: 'Savings' },
          { value: 'current', label: 'Current account' },
          { value: 'salary', label: 'Salary account' },
          { value: 'fixed', label: 'Fixed deposit account' },
          { value: 'nre', label: 'NRE / FCNR' }
        ]
      },
      {
        key: 'account_number',
        label: 'Account number',
        kind: 'text',
        required: true,
        placeholder: 'e.g. 50100234567890'
      },
      { key: 'ifsc_code', label: 'IFSC / SWIFT', kind: 'text', placeholder: 'e.g. HDFC0001234' },
      { key: 'branch', label: 'Branch', kind: 'text', placeholder: 'e.g. Koramangala' },
      { key: 'current_balance', label: 'Current balance', kind: 'money' },
      { key: 'currency', label: 'Currency', kind: 'select', options: CURRENCY_OPTIONS },
      { key: 'is_active', label: 'Active', kind: 'bool', options: BOOL_OPTIONS }
    ]
  },

  assets: {
    entity: 'assets',
    label: 'Assets',
    singular: 'Asset',
    icon: businessOutline,
    accent: '#1f8a70',
    valueKeys: ['current_value', 'purchase_price'],
    displayKeys: ['name', 'current_value'],
    subKeys: ['asset_type', 'category'],
    summaryTotalKey: 'total_value',
    fields: [
      { key: 'name', label: 'Asset name', kind: 'text', required: true, placeholder: 'e.g. Apartment' },
      {
        key: 'asset_type',
        label: 'Asset type',
        kind: 'select',
        required: true,
        options: [
          { value: 'property', label: 'Property' },
          { value: 'vehicle', label: 'Vehicle' },
          { value: 'land', label: 'Land' },
          { value: 'gold', label: 'Gold / Metal' },
          { value: 'collectible', label: 'Collectible' },
          { value: 'equipment', label: 'Equipment' },
          { value: 'other', label: 'Other' }
        ]
      },
      {
        key: 'category',
        label: 'Category',
        kind: 'select',
        required: true,
        options: [
          { value: 'fixed', label: 'Fixed asset' },
          { value: 'current', label: 'Current asset' },
          { value: 'intangible', label: 'Intangible' },
          { value: 'liquid', label: 'Liquid' }
        ]
      },
      { key: 'purchase_price', label: 'Purchase price', kind: 'money' },
      { key: 'current_value', label: 'Current value', kind: 'money' },
      { key: 'purchase_date', label: 'Purchase date', kind: 'date', required: true },
      { key: 'location', label: 'Location', kind: 'text', placeholder: 'City or address' },
      { key: 'description', label: 'Description', kind: 'text' },
      { key: 'annual_income', label: 'Annual income / rent', kind: 'money' },
      { key: 'depreciation_rate', label: 'Depreciation rate', kind: 'number', suffix: '%' },
      { key: 'is_liquid', label: 'Liquid', kind: 'bool', options: BOOL_OPTIONS },
      { key: 'risk_level', label: 'Risk level', kind: 'select', options: RISK_OPTIONS }
    ]
  },

  'fixed-deposits': {
    entity: 'fixed-deposits',
    label: 'Deposits',
    singular: 'Fixed deposit',
    icon: cashOutline,
    accent: '#7a5cc4',
    valueKeys: ['principal_amount', 'current_value'],
    displayKeys: ['name', 'principal_amount'],
    subKeys: ['fd_type', 'maturity_date'],
    summaryTotalKey: 'total_value',
    fields: [
      { key: 'name', label: 'Deposit name', kind: 'text', required: true, placeholder: 'e.g. SBI 3-year FD' },
      { key: 'bank_id', label: 'Bank account reference', kind: 'text', hint: 'Optional id of a linked bank account' },
      {
        key: 'fd_type',
        label: 'Deposit type',
        kind: 'select',
        options: [
          { value: 'regular', label: 'Regular' },
          { value: 'recurring', label: 'Recurring' },
          { value: 'tax_saver', label: 'Tax saver' },
          { value: 'special', label: 'Special' }
        ]
      },
      { key: 'principal_amount', label: 'Principal', kind: 'money', required: true },
      { key: 'interest_rate', label: 'Interest rate', kind: 'number', required: true, suffix: '%', placeholder: '6.75' },
      { key: 'tenure_months', label: 'Tenure', kind: 'number', required: true, suffix: 'months', placeholder: '36' },
      { key: 'start_date', label: 'Start date', kind: 'date', required: true },
      { key: 'maturity_date', label: 'Maturity date', kind: 'date', required: true },
      {
        key: 'compounding_frequency',
        label: 'Compounding',
        kind: 'select',
        options: [
          { value: 'monthly', label: 'Monthly' },
          { value: 'quarterly', label: 'Quarterly' },
          { value: 'semi_annually', label: 'Half yearly' },
          { value: 'yearly', label: 'Yearly' }
        ]
      },
      { key: 'current_value', label: 'Current value', kind: 'money' },
      { key: 'interest_earned', label: 'Interest earned', kind: 'money' },
      { key: 'tax_deducted', label: 'Tax deducted', kind: 'money' },
      { key: 'nominee', label: 'Nominee', kind: 'text' },
      { key: 'certificate_number', label: 'Certificate number', kind: 'text' },
      { key: 'is_auto_renew', label: 'Auto renew', kind: 'bool', options: BOOL_OPTIONS },
      {
        key: 'status',
        label: 'Status',
        kind: 'select',
        options: [
          { value: 'active', label: 'Active' },
          { value: 'matured', label: 'Matured' },
          { value: 'closed', label: 'Closed' }
        ]
      }
    ]
  },

  investments: {
    entity: 'investments',
    label: 'Investments',
    singular: 'Investment',
    icon: pieChartOutline,
    accent: '#c97316',
    valueKeys: ['current_value', 'invested_amount'],
    displayKeys: ['name', 'current_value'],
    subKeys: ['investment_type', 'instrument'],
    summaryTotalKey: 'total_value',
    fields: [
      { key: 'name', label: 'Investment name', kind: 'text', required: true, placeholder: 'e.g. Nifty 50 index fund' },
      {
        key: 'investment_type',
        label: 'Type',
        kind: 'select',
        required: true,
        options: [
          { value: 'stocks', label: 'Stocks' },
          { value: 'equity', label: 'Equity' },
          { value: 'mutual_fund', label: 'Mutual fund' },
          { value: 'bonds', label: 'Bonds' },
          { value: 'real_estate', label: 'Real estate' },
          { value: 'crypto', label: 'Crypto' },
          { value: 'insurance', label: 'Insurance' },
          { value: 'other', label: 'Other' }
        ]
      },
      {
        key: 'instrument',
        label: 'Instrument',
        kind: 'text',
        required: true,
        placeholder: 'e.g. Equity — large cap'
      },
      { key: 'symbol', label: 'Symbol / ISIN', kind: 'text', placeholder: 'e.g. INF204K01K3' },
      { key: 'invested_amount', label: 'Amount invested', kind: 'money', required: true },
      { key: 'current_value', label: 'Current value', kind: 'money' },
      { key: 'units', label: 'Units', kind: 'number' },
      { key: 'unit_price', label: 'Unit price', kind: 'number' },
      { key: 'purchase_date', label: 'Purchase date', kind: 'date', required: true },
      { key: 'purchase_price', label: 'Purchase price', kind: 'number' },
      { key: 'broker_platform', label: 'Broker / platform', kind: 'text' },
      { key: 'account_ref', label: 'Account reference', kind: 'text' },
      { key: 'expected_return', label: 'Expected return', kind: 'number', suffix: '%' },
      { key: 'actual_return', label: 'Actual return', kind: 'money' },
      { key: 'annualized_return', label: 'Annualised return', kind: 'number', suffix: '%' },
      { key: 'dividend_yield', label: 'Dividend yield', kind: 'number', suffix: '%' },
      { key: 'risk_level', label: 'Risk level', kind: 'select', options: RISK_OPTIONS },
      { key: 'is_liquid', label: 'Liquid', kind: 'bool', options: BOOL_OPTIONS },
      { key: 'tax_saving', label: 'Tax saving', kind: 'bool', options: BOOL_OPTIONS },
      { key: 'lock_in_until', label: 'Locked until', kind: 'date' },
      { key: 'notes', label: 'Notes', kind: 'text' }
    ]
  },

  'credit-cards': {
    entity: 'credit-cards',
    label: 'Cards',
    singular: 'Card',
    icon: cardOutline,
    accent: '#c2443a',
    valueKeys: ['current_balance', 'credit_limit'],
    displayKeys: ['name', 'current_balance'],
    subKeys: ['bank_name', 'card_type'],
    summaryTotalKey: 'total_balance',
    fields: [
      { key: 'name', label: 'Card name', kind: 'text', required: true, placeholder: 'e.g. Titanium card' },
      { key: 'bank_name', label: 'Issuer', kind: 'text', required: true, placeholder: 'e.g. HDFC Bank' },
      {
        key: 'card_type',
        label: 'Card type',
        kind: 'select',
        required: true,
        options: [
          { value: 'credit', label: 'Credit' },
          { value: 'debit', label: 'Debit' },
          { value: 'prepaid', label: 'Prepaid' }
        ]
      },
      { key: 'last_four_digits', label: 'Last 4 digits', kind: 'text', required: true, placeholder: '4242' },
      { key: 'credit_limit', label: 'Credit limit', kind: 'money' },
      { key: 'current_balance', label: 'Outstanding balance', kind: 'money' },
      { key: 'available_credit', label: 'Available credit', kind: 'money' },
      { key: 'interest_rate', label: 'Interest rate', kind: 'number', suffix: '%' },
      { key: 'billing_cycle_day', label: 'Billing cycle day', kind: 'number', placeholder: '1', suffix: 'of month' },
      { key: 'due_date_day', label: 'Due day', kind: 'number', placeholder: '20', suffix: 'of month' },
      { key: 'annual_fee', label: 'Annual fee', kind: 'money' },
      { key: 'reward_program', label: 'Reward programme', kind: 'text' },
      { key: 'reward_points', label: 'Reward points', kind: 'number' },
      { key: 'is_active', label: 'Active', kind: 'bool', options: BOOL_OPTIONS }
    ]
  },

  incomes: {
    entity: 'incomes',
    label: 'Income',
    singular: 'Income',
    icon: statsChartOutline,
    accent: '#1d8f4a',
    valueKeys: ['amount'],
    displayKeys: ['title', 'amount'],
    subKeys: ['source', 'income_type'],
    summaryTotalKey: 'yearly_income',
    fields: [
      { key: 'title', label: 'Description', kind: 'text', required: true, placeholder: 'e.g. Monthly salary' },
      { key: 'amount', label: 'Amount', kind: 'money', required: true },
      {
        key: 'income_type',
        label: 'Category',
        kind: 'select',
        required: true,
        options: [
          { value: 'salary', label: 'Salary' },
          { value: 'business', label: 'Business' },
          { value: 'investment', label: 'Investment' },
          { value: 'rental', label: 'Rental' },
          { value: 'interest', label: 'Interest' },
          { value: 'freelance', label: 'Freelance' },
          { value: 'other', label: 'Other' }
        ]
      },
      { key: 'source', label: 'Source', kind: 'text', required: true, placeholder: 'e.g. Employer name' },
      { key: 'income_date', label: 'Received date', kind: 'date', required: true },
      { key: 'is_recurring', label: 'Recurring', kind: 'bool', options: BOOL_OPTIONS },
      { key: 'recurrence', label: 'Frequency', kind: 'select', options: RECURRENCE_OPTIONS },
      { key: 'frequency_multiplier', label: 'Times per period', kind: 'number', placeholder: '1' },
      { key: 'is_gross', label: 'Gross (before tax)', kind: 'bool', options: BOOL_OPTIONS },
      { key: 'tax_withheld', label: 'Tax withheld', kind: 'money' },
      { key: 'currency', label: 'Currency', kind: 'select', options: CURRENCY_OPTIONS },
      { key: 'description', label: 'Notes', kind: 'text' },
      { key: 'bank_id', label: 'Credited to', kind: 'select', dynamic: 'accounts' }
    ]
  },

  expenses: {
    entity: 'expenses',
    label: 'Expenses',
    singular: 'Expense',
    icon: pricetagOutline,
    accent: '#c2443a',
    valueKeys: ['amount'],
    displayKeys: ['title', 'amount'],
    subKeys: ['category', 'payment_method'],
    summaryTotalKey: 'total_amount',
    fields: [
      { key: 'title', label: 'Description', kind: 'text', required: true, placeholder: 'e.g. Monthly rent' },
      { key: 'amount', label: 'Amount', kind: 'money', required: true },
      {
        key: 'expense_type',
        label: 'Type',
        kind: 'select',
        required: true,
        options: [
          { value: 'need', label: 'Need' },
          { value: 'want', label: 'Want' },
          { value: 'savings', label: 'Savings' },
          { value: 'debt', label: 'Debt repayment' },
          { value: 'investment', label: 'Investment' }
        ]
      },
      {
        key: 'category',
        label: 'Category',
        kind: 'select',
        required: true,
        options: [
          { value: 'housing', label: 'Housing' },
          { value: 'food', label: 'Food' },
          { value: 'transport', label: 'Transport' },
          { value: 'utilities', label: 'Utilities' },
          { value: 'health', label: 'Health' },
          { value: 'education', label: 'Education' },
          { value: 'entertainment', label: 'Entertainment' },
          { value: 'shopping', label: 'Shopping' },
          { value: 'travel', label: 'Travel' },
          { value: 'insurance', label: 'Insurance' },
          { value: 'tax', label: 'Tax' },
          { value: 'other', label: 'Other' }
        ]
      },
      { key: 'expense_date', label: 'Date', kind: 'date', required: true },
      { key: 'is_recurring', label: 'Recurring', kind: 'bool', options: BOOL_OPTIONS },
      { key: 'recurrence', label: 'Frequency', kind: 'select', options: RECURRENCE_OPTIONS },
      { key: 'recurrence_end_date', label: 'Recurs until', kind: 'date' },
      {
        key: 'payment_method',
        label: 'Payment method',
        kind: 'select',
        options: [
          { value: 'cash', label: 'Cash' },
          { value: 'card', label: 'Card' },
          { value: 'bank_transfer', label: 'Bank transfer' },
          { value: 'upi', label: 'UPI' },
          { value: 'cheque', label: 'Cheque' },
          { value: 'other', label: 'Other' }
        ]
      },
      { key: 'is_fixed', label: 'Fixed cost', kind: 'bool', options: BOOL_OPTIONS },
      {
        key: 'priority',
        label: 'Priority',
        kind: 'select',
        options: [
          { value: 'low', label: 'Low' },
          { value: 'medium', label: 'Medium' },
          { value: 'high', label: 'High' }
        ]
      },
      { key: 'currency', label: 'Currency', kind: 'select', options: CURRENCY_OPTIONS },
      { key: 'description', label: 'Notes', kind: 'text' },
      { key: 'location', label: 'Location', kind: 'text' },
      { key: 'tags', label: 'Tags', kind: 'text', placeholder: 'Comma separated' },
      { key: 'bank_id', label: 'Paid from', kind: 'select', dynamic: 'accounts' },
      { key: 'credit_card_id', label: 'Paid by card', kind: 'select', dynamic: 'cards' }
    ]
  }
}

export const ENTITY_LIST = Object.keys(ENTITY_CONFIGS) as EntityType[]

/** The 5 money-holding groups, in display order, shown on Accounts + Dashboard. */
export const ACCOUNT_ENTITIES = [
  'banks',
  'assets',
  'credit-cards',
  'fixed-deposits',
  'investments'
] as const

export type AccountEntityName = (typeof ACCOUNT_ENTITIES)[number]

/** All 7 entity names, in the order the backend registers them. */
export const ALL_ENTITY_NAMES: EntityType[] = [
  'banks',
  'assets',
  'expenses',
  'credit-cards',
  'fixed-deposits',
  'investments',
  'incomes'
]

/** Backend table name for each entity — the sync endpoint only accepts snake_case. */
export const ENTITY_SNAKE: Record<EntityType, string> = {
  banks: 'banks',
  assets: 'assets',
  expenses: 'expenses',
  'credit-cards': 'credit_cards',
  'fixed-deposits': 'fixed_deposits',
  investments: 'investments',
  incomes: 'incomes'
}

export function isEntityType(value: unknown): value is EntityType {
  return typeof value === 'string' && Object.prototype.hasOwnProperty.call(ENTITY_CONFIGS, value)
}

export function entityConfig(entity: string): EntityConfig {
  return isEntityType(entity) ? ENTITY_CONFIGS[entity] : ENTITY_CONFIGS.banks
}

export function rowTitle(cfg: EntityConfig, row: FinanceRow): string {
  const key = cfg.displayKeys[0]
  const v = row[key]
  return v !== undefined && v !== null && String(v).length ? String(v) : cfg.singular
}

/** Primary list-row label (falls back to the title when the column is absent). */
export function rowSecondary(cfg: EntityConfig, row: FinanceRow): string {
  const key = cfg.displayKeys[1]
  const v = row[key]
  return v === undefined || v === null ? '' : String(v)
}

export function rowValue(cfg: EntityConfig, row: FinanceRow): number {
  for (const k of cfg.valueKeys) {
    if (row[k] !== undefined && row[k] !== null && row[k] !== '') {
      const n = Number(row[k])
      if (Number.isFinite(n)) return n
    }
  }
  return 0
}

/** Best available "context" line for a row: date columns first, then descriptive ones. */
export function rowSubtitle(cfg: EntityConfig, row: FinanceRow): string {
  const parts: string[] = []
  for (const key of cfg.subKeys) {
    const v = row[key]
    if (v === undefined || v === null || v === '') continue
    const text = String(v).replace(/_/g, ' ')
    if (!parts.includes(text)) parts.push(text)
  }
  return parts.slice(0, 2).join(' · ')
}

function humanize(value: unknown): string {
  if (value === undefined || value === null || value === '') return ''
  return String(value).replace(/_/g, ' ')
}

export function fieldLabelValue(field: FieldDef, row: FinanceRow): string {
  const raw = row[field.key]
  if (raw === undefined || raw === null || raw === '') return '—'
  if (field.kind === 'bool') return raw === true || raw === 'true' ? 'Yes' : 'No'
  if (field.kind === 'select') {
    const match = field.options?.find((o) => o.value === String(raw))
    return match ? match.label : humanize(raw)
  }
  return humanize(raw)
}

export interface SummaryHead {
  total: number
  count: number
}

/** Headline total + row count for a `/summary` payload, whichever shape it uses. */
export function summaryHead(entity: EntityType, summary: unknown): SummaryHead {
  if (!summary || typeof summary !== 'object') return { total: 0, count: 0 }
  const rec = summary as Record<string, unknown>
  const cfg = entityConfig(entity)
  const totalRaw = rec[cfg.summaryTotalKey]
  const total = typeof totalRaw === 'number' && Number.isFinite(totalRaw) ? totalRaw : 0
  let count = 0
  for (const key of ['count', 'active_count']) {
    const v = rec[key]
    if (typeof v === 'number' && Number.isFinite(v)) {
      count = v
      break
    }
  }
  return { total, count }
}

/** Every field of a config, used for the per-record detail sheet. */
export function detailFields(cfg: EntityConfig): FieldDef[] {
  return cfg.fields
}

/** All distinct values already stored in a column — powers the category filter chips. */
export function distinctValues(rows: FinanceRow[], key: string): string[] {
  const seen = new Set<string>()
  for (const row of rows) {
    const v = row[key]
    if (v === undefined || v === null || v === '') continue
    seen.add(String(v))
  }
  return [...seen].sort((a, b) => a.localeCompare(b))
}

export type { EntitySummaryMap }