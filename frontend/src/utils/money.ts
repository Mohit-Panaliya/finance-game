/**
 * Money formatting.
 *
 * Every backend model carries a `currency` column (INR by default). These helpers
 * honour it via Intl.NumberFormat and cache formatters per currency+options pair.
 */

export const DEFAULT_CURRENCY = 'INR'

const formatterCache = new Map<string, Intl.NumberFormat | null>()

function toCurrency(input?: string | null): string {
  const code = String(input ?? '').trim().toUpperCase()
  if (!code) return DEFAULT_CURRENCY
  return /^[A-Z]{3}$/.test(code) ? code : DEFAULT_CURRENCY
}

function formatter(currency: string, options: Intl.NumberFormatOptions): Intl.NumberFormat | null {
  const key = `${currency}|${JSON.stringify(options)}`
  const cached = formatterCache.get(key)
  if (cached !== undefined) return cached
  let made: Intl.NumberFormat | null
  try {
    made = new Intl.NumberFormat(undefined, { ...options, currency, style: 'currency' })
  } catch {
    made = null
  }
  formatterCache.set(key, made)
  return made
}

function toNumber(value: unknown): number {
  if (typeof value === 'number') return Number.isFinite(value) ? value : 0
  if (typeof value === 'string' && value.trim() !== '') {
    const n = Number(value)
    return Number.isFinite(n) ? n : 0
  }
  return 0
}

/** Reads the `currency` column off a finance row, defaulting to INR. */
export function rowCurrency(row: unknown): string {
  if (!row || typeof row !== 'object') return DEFAULT_CURRENCY
  const value = (row as Record<string, unknown>).currency
  return toCurrency(typeof value === 'string' ? value : undefined)
}

/**
 * Full precision currency string, e.g. `₹1,23,456.78`.
 * `compact: true` shortens large values (`₹1.2L`) for dense cards.
 */
export function formatMoney(
  value: unknown,
  currency?: string | null,
  options: { compact?: boolean; decimals?: number } = {}
): string {
  const n = toNumber(value)
  const cur = toCurrency(currency)
  const compact = options.compact === true
  const decimals = options.decimals ?? (compact ? 1 : Math.abs(n) < 100 && !Number.isInteger(n) ? 2 : 0)

  const intl = formatter(cur, {
    notation: compact ? 'compact' : 'standard',
    maximumFractionDigits: decimals,
    minimumFractionDigits: compact ? 0 : Math.min(decimals, 2)
  })
  if (intl) {
    try {
      return intl.format(n)
    } catch {
      /* fall through to manual */
    }
  }
  const abs = Math.abs(n)
  const sign = n < 0 ? '-' : ''
  if (compact && abs >= 1000) {
    const units: Array<[number, string]> = [
      [1e7, 'Cr'],
      [1e5, 'L'],
      [1e3, 'K']
    ]
    for (const [size, suffix] of units) {
      if (abs >= size) {
        const scaled = abs / size
        return `${sign}${cur} ${scaled >= 100 ? scaled.toFixed(0) : scaled.toFixed(1)}${suffix}`
      }
    }
  }
  return `${sign}${cur} ${abs.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2
  })}`
}

/** Compact variant used inside cards and bar rows. */
export function formatCompact(value: unknown, currency?: string | null): string {
  return formatMoney(value, currency, { compact: true })
}

/** Always carries an explicit `+` / `−` sign — used for income vs expense rows. */
export function formatSigned(value: unknown, currency?: string | null, compact = false): string {
  const n = toNumber(value)
  const abs = formatMoney(Math.abs(n), currency, { compact })
  return n < 0 ? `−${abs}` : `+${abs}`
}

/** `1234.5` → `1,234.5`; used for percentages and plain counts. */
export function formatNumber(value: unknown, decimals = 0): string {
  const n = toNumber(value)
  return n.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: Math.max(0, decimals)
  })
}

/** Signed percentage, e.g. `+4.2%`. */
export function formatPercent(value: unknown, decimals = 1): string {
  const n = toNumber(value)
  const body = `${Math.abs(n).toFixed(decimals)}%`
  if (n > 0) return `+${body}`
  if (n < 0) return `−${body}`
  return body
}