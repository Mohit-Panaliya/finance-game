import type { EntityType, FinanceRow } from '@/types'

export type FieldKind = 'text' | 'number' | 'date' | 'select' | 'money'

export interface FieldDef {
  key: string
  label: string
  kind: FieldKind
  options?: Array<{ value: string; label: string; icon?: string }>
  suffix?: string
  placeholder?: string
}

export interface EntityConfig {
  entity: EntityType
  label: string
  icon: string
  accent: string
  valueKeys: string[]
  displayKeys: [string, string]
  fields: FieldDef[]
}

export const ENTITY_CONFIGS: Record<EntityType, EntityConfig> = {
  banks: {
    entity: 'banks',
    label: 'Banks',
    icon: '🏦',
    accent: '#f5c542',
    valueKeys: ['balance', 'amount'],
    displayKeys: ['name', 'balance'],
    fields: [
      { key: 'name', label: 'Bank Name', kind: 'text', placeholder: 'e.g. Ironhold Bank' },
      { key: 'account_number', label: 'Account Number', kind: 'text', placeholder: '•••• 4242' },
      { key: 'ifsc', label: 'IFSC / SWIFT', kind: 'text', placeholder: 'IRON0001' },
      { key: 'balance', label: 'Balance', kind: 'money', suffix: '🪙' },
      {
        key: 'account_type',
        label: 'Account Type',
        kind: 'select',
        options: [
          { value: 'savings', label: 'Savings', icon: '🪙' },
          { value: 'checking', label: 'Checking', icon: '⚔️' },
          { value: 'business', label: 'Business', icon: '🏰' }
        ]
      }
    ]
  },
  assets: {
    entity: 'assets',
    label: 'Assets',
    icon: '🏰',
    accent: '#3498db',
    valueKeys: ['value', 'amount'],
    displayKeys: ['name', 'value'],
    fields: [
      { key: 'name', label: 'Asset Name', kind: 'text', placeholder: 'e.g. Mountain Keep' },
      {
        key: 'type',
        label: 'Asset Type',
        kind: 'select',
        options: [
          { value: 'property', label: 'Property', icon: '🏰' },
          { value: 'vehicle', label: 'Vehicle', icon: '🛞' },
          { value: 'gold', label: 'Gold / Metal', icon: '🪙' },
          { value: 'collectible', label: 'Collectible', icon: '🏺' },
          { value: 'other', label: 'Other', icon: '📦' }
        ]
      },
      { key: 'value', label: 'Current Value', kind: 'money', suffix: '🪙' },
      { key: 'purchase_date', label: 'Purchase Date', kind: 'date' },
      { key: 'purchase_price', label: 'Purchase Price', kind: 'money', suffix: '🪙' }
    ]
  },
  expenses: {
    entity: 'expenses',
    label: 'Expenses',
    icon: '⚔️',
    accent: '#e74c3c',
    valueKeys: ['amount'],
    displayKeys: ['title', 'amount'],
    fields: [
      { key: 'title', label: 'Expense Name', kind: 'text', placeholder: 'e.g. Army Rations' },
      { key: 'amount', label: 'Amount', kind: 'money', suffix: '🪙' },
      {
        key: 'category',
        label: 'Category',
        kind: 'select',
        options: [
          { value: 'food', label: 'Food', icon: '🍖' },
          { value: 'housing', label: 'Housing', icon: '🏠' },
          { value: 'transport', label: 'Transport', icon: '🛞' },
          { value: 'health', label: 'Health', icon: '❤️' },
          { value: 'entertainment', label: 'Fun', icon: '🎪' },
          { value: 'bills', label: 'Bills', icon: '📜' }
        ]
      },
      {
        key: 'frequency',
        label: 'Frequency',
        kind: 'select',
        options: [
          { value: 'once', label: 'One-time', icon: '🎯' },
          { value: 'monthly', label: 'Monthly', icon: '🌙' },
          { value: 'weekly', label: 'Weekly', icon: '📅' },
          { value: 'yearly', label: 'Yearly', icon: '🗓️' }
        ]
      },
      { key: 'date', label: 'Date', kind: 'date' }
    ]
  },
  'credit-cards': {
    entity: 'credit-cards',
    label: 'Cards',
    icon: '🧱',
    accent: '#9b59b6',
    valueKeys: ['outstanding', 'limit'],
    displayKeys: ['issuer', 'outstanding'],
    fields: [
      { key: 'issuer', label: 'Card Issuer', kind: 'text', placeholder: 'e.g. Dragspur Card' },
      { key: 'last4', label: 'Last 4 Digits', kind: 'text', placeholder: '4242' },
      { key: 'limit', label: 'Credit Limit', kind: 'money', suffix: '🪙' },
      { key: 'outstanding', label: 'Outstanding', kind: 'money', suffix: '🪙' },
      { key: 'due_date', label: 'Due Date', kind: 'date' },
      {
        key: 'network',
        label: 'Network',
        kind: 'select',
        options: [
          { value: 'visa', label: 'Visa', icon: '💠' },
          { value: 'mastercard', label: 'Mastercard', icon: '🔴' },
          { value: 'amex', label: 'Amex', icon: '🔶' }
        ]
      }
    ]
  },
  'fixed-deposits': {
    entity: 'fixed-deposits',
    label: 'Vaults (FDs)',
    icon: '🗄️',
    accent: '#4caf50',
    valueKeys: ['principal', 'amount'],
    displayKeys: ['bank', 'principal'],
    fields: [
      { key: 'bank', label: 'Bank / Holder', kind: 'text', placeholder: 'e.g. Ironhold Bank' },
      { key: 'principal', label: 'Principal', kind: 'money', suffix: '🪙' },
      { key: 'rate', label: 'Interest Rate', kind: 'number', suffix: '%', placeholder: '6.5' },
      { key: 'start_date', label: 'Start Date', kind: 'date' },
      { key: 'maturity_date', label: 'Maturity Date', kind: 'date' }
    ]
  },
  investments: {
    entity: 'investments',
    label: 'Investments',
    icon: '🔮',
    accent: '#d55cff',
    valueKeys: ['amount', 'value'],
    displayKeys: ['name', 'amount'],
    fields: [
      { key: 'name', label: 'Investment Name', kind: 'text', placeholder: 'e.g. Dragon Index Fund' },
      {
        key: 'type',
        label: 'Type',
        kind: 'select',
        options: [
          { value: 'stocks', label: 'Stocks', icon: '📈' },
          { value: 'crypto', label: 'Crypto', icon: '🪙' },
          { value: 'mutual-fund', label: 'Mutual Fund', icon: '🧺' },
          { value: 'bonds', label: 'Bonds', icon: '📜' },
          { value: 'real-estate', label: 'Real Estate', icon: '🏙️' }
        ]
      },
      { key: 'amount', label: 'Invested', kind: 'money', suffix: '🪙' },
      { key: 'returns', label: 'Current Returns', kind: 'money', suffix: '🪙' },
      { key: 'purchase_date', label: 'Purchase Date', kind: 'date' }
    ]
  },
  incomes: {
    entity: 'incomes',
    label: 'Income',
    icon: '🧪',
    accent: '#8be9fd',
    valueKeys: ['amount'],
    displayKeys: ['source', 'amount'],
    fields: [
      { key: 'source', label: 'Source', kind: 'text', placeholder: 'e.g. Quest Rewards' },
      { key: 'amount', label: 'Amount', kind: 'money', suffix: '🪙' },
      {
        key: 'frequency',
        label: 'Frequency',
        kind: 'select',
        options: [
          { value: 'weekly', label: 'Weekly', icon: '📅' },
          { value: 'monthly', label: 'Monthly', icon: '🌙' },
          { value: 'yearly', label: 'Yearly', icon: '🗓️' }
        ]
      },
      { key: 'date', label: 'Received Date', kind: 'date' },
      {
        key: 'category',
        label: 'Category',
        kind: 'select',
        options: [
          { value: 'salary', label: 'Salary', icon: '⚒️' },
          { value: 'business', label: 'Business', icon: '🏰' },
          { value: 'freelance', label: 'Freelance', icon: '🗡️' },
          { value: 'passive', label: 'Passive', icon: '♾️' }
        ]
      }
    ]
  }
}

export const ENTITY_LIST = Object.keys(ENTITY_CONFIGS) as EntityType[]

export function entityConfig(entity: string): EntityConfig {
  return ENTITY_CONFIGS[entity as EntityType] ?? ENTITY_CONFIGS.banks
}

export function rowTitle(cfg: EntityConfig, row: FinanceRow): string {
  const key = cfg.displayKeys[0]
  const v = row[key]
  return v !== undefined && v !== null && String(v).length ? String(v) : cfg.label
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

export function formatGold(n: number): string {
  if (Math.abs(n) >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`
  if (Math.abs(n) >= 10_000) return `${(n / 1000).toFixed(1)}K`
  return Math.round(n).toLocaleString()
}
