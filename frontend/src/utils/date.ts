/**
 * Date formatting.
 *
 * Backend date columns are plain `YYYY-MM-DD` strings and RFC3339 timestamps for
 * audit columns. `new Date('2026-03-04')` is parsed as UTC midnight and then
 * rendered in local time, which shifts the day for anyone west of Greenwich — so
 * every bare `YYYY-MM-DD` is parsed by component here and never through Date.
 */

const MONTHS = [
  'Jan',
  'Feb',
  'Mar',
  'Apr',
  'May',
  'Jun',
  'Jul',
  'Aug',
  'Sep',
  'Oct',
  'Nov',
  'Dec'
]

const DATE_ONLY = /^(\d{4})-(\d{2})-(\d{2})$/
const MONTH_ONLY = /^(\d{4})-(\d{2})$/

export interface DateParts {
  y: number
  m: number
  d: number
}

function pad(n: number): string {
  return String(n).padStart(2, '0')
}

/** Splits `YYYY-MM-DD` (or a Date) into parts without any timezone conversion. */
export function parseDateParts(value: unknown): DateParts | null {
  if (value instanceof Date) {
    if (Number.isNaN(value.getTime())) return null
    return { y: value.getFullYear(), m: value.getMonth() + 1, d: value.getDate() }
  }
  if (typeof value === 'number' && Number.isFinite(value)) {
    const dt = new Date(value)
    return Number.isNaN(dt.getTime()) ? null : { y: dt.getFullYear(), m: dt.getMonth() + 1, d: dt.getDate() }
  }
  if (typeof value !== 'string') return null
  const raw = value.trim()
  if (!raw) return null
  const only = DATE_ONLY.exec(raw)
  if (only) {
    return { y: Number(only[1]), m: Number(only[2]), d: Number(only[3]) }
  }
  const stamp = raw.slice(0, 10)
  const second = DATE_ONLY.exec(stamp)
  if (second) return { y: Number(second[1]), m: Number(second[2]), d: Number(second[3]) }
  const dt = new Date(raw)
  if (Number.isNaN(dt.getTime())) return null
  return { y: dt.getFullYear(), m: dt.getMonth() + 1, d: dt.getDate() }
}

/** Days since the epoch for a date-only string, used for timezone-free arithmetic. */
function ordinal(parts: DateParts): number {
  return Date.UTC(parts.y, parts.m - 1, parts.d)
}

function todayParts(): DateParts {
  const now = new Date()
  return { y: now.getFullYear(), m: now.getMonth() + 1, d: now.getDate() }
}

function isEmpty(value: unknown): boolean {
  return value === undefined || value === null || (typeof value === 'string' && value.trim() === '')
}

/** `2026-03-04` → `4 Mar 2026`. */
export function formatDate(value: unknown): string {
  const parts = parseDateParts(value)
  if (!parts) return isEmpty(value) ? '—' : String(value)
  return `${parts.d} ${MONTHS[Math.min(11, Math.max(0, parts.m - 1))]} ${parts.y}`
}

/** `2026-03-04` → `4 Mar` (year omitted unless it differs from the current year). */
export function formatDayMonth(value: unknown): string {
  const parts = parseDateParts(value)
  if (!parts) return isEmpty(value) ? '—' : String(value)
  const base = `${parts.d} ${MONTHS[Math.min(11, Math.max(0, parts.m - 1))]}`
  return parts.y === todayParts().y ? base : `${base} ${String(parts.y).slice(2)}`
}

/** Accepts `2026-03` or any full date → `Mar 2026`. */
export function formatMonth(value: unknown): string {
  if (typeof value === 'string') {
    const only = MONTH_ONLY.exec(value.trim())
    if (only) {
      const m = Number(only[2])
      return `${MONTHS[Math.min(11, Math.max(0, m - 1))]} ${only[1]}`
    }
  }
  const parts = parseDateParts(value)
  if (!parts) return isEmpty(value) ? '—' : String(value)
  return `${MONTHS[Math.min(11, Math.max(0, parts.m - 1))]} ${parts.y}`
}

/** `Mar 2026` — the label used by the cash-flow and expense charts. */
export function formatMonthLabel(value: unknown): string {
  return formatMonth(value)
}

/** RFC3339 timestamp → `4 Mar 2026, 14:05`. Empty/invalid input yields `—`. */
export function formatDateTime(value: unknown): string {
  if (isEmpty(value)) return '—'
  if (typeof value === 'string' && DATE_ONLY.test(value.trim())) return formatDate(value)
  const dt = value instanceof Date ? value : new Date(String(value))
  if (Number.isNaN(dt.getTime())) return formatDate(value)
  return `${formatDate(dt)}, ${pad(dt.getHours())}:${pad(dt.getMinutes())}`
}

/** Whole days between two dates (positive when `to` is later), timezone-free. */
export function daysBetween(from: unknown, to: unknown = new Date()): number | null {
  const a = parseDateParts(from)
  const b = parseDateParts(to)
  if (!a || !b) return null
  return Math.round((ordinal(b) - ordinal(a)) / 86_400_000)
}

/**
 * `Today`, `Tomorrow`, `Yesterday`, `in 12 days`, `3 months ago`, `2 years ago`.
 * Used for FD maturity and transaction recency.
 */
export function relativeDays(value: unknown): string {
  const days = daysBetween(new Date(), value)
  if (days === null) return '—'
  if (days === 0) return 'Today'
  if (days === 1) return 'Tomorrow'
  if (days === -1) return 'Yesterday'
  const abs = Math.abs(days)
  const unit = (n: number, one: string, many: string): string => `${n} ${n === 1 ? one : many}`
  let body: string
  if (abs < 7) body = unit(abs, 'day', 'days')
  else if (abs < 31) body = unit(Math.round(abs / 7), 'week', 'weeks')
  else if (abs < 365) body = unit(Math.round(abs / 30), 'month', 'months')
  else body = unit(Math.round((abs / 365) * 10) / 10, 'year', 'years')
  return days > 0 ? `in ${body}` : `${body} ago`
}

/** Signed day count for sort keys and compact badges: `+12`, `−3`. */
export function dayDelta(value: unknown): number | null {
  return daysBetween(new Date(), value)
}

/** `2026-03-04` for today, in local time — the default for date inputs. */
export function todayISO(): string {
  const p = todayParts()
  return `${p.y}-${pad(p.m)}-${pad(p.d)}`
}

/** `2026-03` bucket key used to group expenses by month client-side. */
export function monthKey(value: unknown): string {
  if (typeof value === 'string') {
    const raw = value.trim()
    if (MONTH_ONLY.test(raw)) return raw
    const parts = parseDateParts(raw)
    if (parts) return `${parts.y}-${pad(parts.m)}`
    return ''
  }
  const parts = parseDateParts(value)
  return parts ? `${parts.y}-${pad(parts.m)}` : ''
}