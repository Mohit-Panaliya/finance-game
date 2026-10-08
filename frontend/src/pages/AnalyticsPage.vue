<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import { RouterLink, useRouter } from 'vue-router'
import { analyticsOutline, linkOutline, refreshOutline, trendingUpOutline } from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import { useCountUp } from '@/composables/useCountUp'
import { ENTITY_SNAKE } from '@/entityConfig'
import type { EntityType } from '@/types'
import { BarChart, CalendarHeatmap, DonutChart, LineChart, ProgressRing } from '@/components/charts'
import type { BarDatum, DonutSlice, HeatmapDay, LineSeries } from '@/components/charts'
import { formatCompact, formatMoney, formatNumber, formatPercent } from '@/utils/money'
import { daysBetween, formatDate, formatMonth, relativeDays, todayISO } from '@/utils/date'
import AppButton from '@/components/ui/AppButton.vue'
import SyncChip from '@/components/ui/SyncChip.vue'

const finance = useFinanceStore()
const sync = useSyncStore()
const router = useRouter()

const FILTER_TYPES: EntityType[] = ['banks', 'assets', 'credit-cards', 'fixed-deposits', 'investments', 'incomes', 'expenses']
const selectedTypes = ref<EntityType[]>([])

const analysis = computed(() => finance.analysis)

/* ------------------------------------------------------------------ *
 * helpers
 * ------------------------------------------------------------------ */

const MONTH_SHORT = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']

function num(value: unknown): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : 0
}

/** `fixed_deposits` → `fixed deposits`. */
function prettify(value: unknown): string {
  return String(value ?? '').replace(/_/g, ' ').trim()
}

/** Axis labels want `Mar`, not `Mar 2026`. */
function shortMonth(value: unknown): string {
  const raw = String(value ?? '').trim()
  const m = /^(\d{4})-(\d{2})/.exec(raw)
  if (m) {
    const i = Number(m[2]) - 1
    return MONTH_SHORT[Math.min(11, Math.max(0, i))] ?? ''
  }
  return prettify(value)
}

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

/* ------------------------------------------------------------------ *
 * 1 · net worth
 * ------------------------------------------------------------------ */

/** Only the net-worth components whose entity type is selected are counted. */
const filteredNetWorth = computed(() => {
  const nw = analysis.value?.net_worth
  if (!nw) return 0
  if (!selectedTypes.value.length) return num(nw.total)
  const map: Array<[EntityType, number]> = [
    ['banks', num(nw.banks)],
    ['assets', num(nw.assets)],
    ['fixed-deposits', num(nw.fixed_deposits)],
    ['investments', num(nw.investments)]
  ]
  return map
    .filter(([t]) => selectedTypes.value.includes(t))
    .reduce((acc, [, v]) => acc + v, 0)
})

const netWorthTotal = useCountUp(computed(() => filteredNetWorth.value))

const netWorthSlices = computed<DonutSlice[]>(() => {
  const nw = analysis.value?.net_worth
  if (!nw) return []
  const all: DonutSlice[] = [
    { label: 'Banks', value: num(nw.banks) },
    { label: 'Assets', value: num(nw.assets) },
    { label: 'Deposits', value: num(nw.fixed_deposits) },
    { label: 'Investments', value: num(nw.investments) }
  ]
  if (!selectedTypes.value.length) return all
  const keep = new Set(selectedTypes.value)
  return all.filter((s, i) => {
    const type: EntityType[] = ['banks', 'assets', 'fixed-deposits', 'investments']
    return keep.has(type[i] as EntityType)
  })
})

const incomeSlices = computed<DonutSlice[]>(() => {
  const y = analysis.value?.yearly_income_by_type
  if (!y) return []
  return [
    { label: 'Salary', value: num(y.salary) },
    { label: 'Business', value: num(y.business) },
    { label: 'Investment', value: num(y.investment) },
    { label: 'Other', value: num(y.other) }
  ]
})

const savings = computed(() => analysis.value?.savings_rate ?? null)
const savingsPct = computed(() => num(savings.value?.savings_rate_pct))
const savingsRingValue = computed(() => Math.min(100, Math.max(0, savingsPct.value)))
const savingsTone = computed(() => {
  if (savingsPct.value >= 25) return 'var(--success)'
  if (savingsPct.value >= 0) return 'var(--warning)'
  return 'var(--danger)'
})

/* ------------------------------------------------------------------ *
 * 2 · calendar window
 * ------------------------------------------------------------------ */

type RangeKey = 'month' | '3m' | '6m' | 'year' | 'all'
type CalMode = 'total' | 'expense'

const RANGE_PRESETS: Array<{ key: RangeKey; label: string }> = [
  { key: 'month', label: 'This month' },
  { key: '3m', label: 'Last 3 months' },
  { key: '6m', label: 'Last 6 months' },
  { key: 'year', label: 'This year' },
  { key: 'all', label: 'All' }
]

const range = ref<RangeKey>('6m')
const calMode = ref<CalMode>('total')

/** First `YYYY-MM-DD` covered by a preset; `null` means "everything". */
function rangeStart(key: RangeKey): string | null {
  if (key === 'all') return null
  const now = new Date()
  const y = now.getFullYear()
  const m = now.getMonth() + 1
  if (key === 'month') return `${y}-${pad2(m)}-01`
  if (key === 'year') return `${y}-01-01`
  let back = y
  let bm = m - (key === '3m' ? 3 : 6)
  while (bm <= 0) {
    bm += 12
    back -= 1
  }
  return `${back}-${pad2(bm)}-01`
}

const calRows = computed(() => {
  const start = rangeStart(range.value)
  const rows = (analysis.value?.calendar ?? []).filter((d) => {
    if (!d?.date) return false
    if (start && d.date < start) return false
    return true
  })
  return rows.slice().sort((a, b) => a.date.localeCompare(b.date))
})

const calDays = computed<HeatmapDay[]>(() =>
  calRows.value.map((d) => ({
    date: d.date,
    value: calMode.value === 'expense' ? num(d.expense) : num(d.income) + num(d.expense)
  }))
)

const heatWeeks = computed(() => {
  if (range.value === 'all') return 53
  const start = rangeStart(range.value)
  if (!start) return 53
  const span = daysBetween(start, todayISO()) ?? 0
  return Math.min(53, Math.max(4, Math.ceil(Math.abs(span) / 7) + 1))
})

const calTotals = computed(() => {
  let income = 0
  let expense = 0
  let net = 0
  let activeDays = 0
  let peak = 0
  for (const d of calRows.value) {
    const i = num(d.income)
    const e = num(d.expense)
    income += i
    expense += e
    net += num(d.net) || i - e
    const busy = i + e
    if (busy > 0) activeDays += 1
    if (busy > peak) peak = busy
  }
  return { income, expense, net, activeDays, peak }
})

/* ------------------------------------------------------------------ *
 * 3 · cash flow
 * ------------------------------------------------------------------ */

const cashFlow = computed(() => analysis.value?.cash_flow ?? [])

const cfLabels = computed(() => cashFlow.value.map((p) => shortMonth(p.month)))

const cfSeries = computed<LineSeries[]>(() => [
  { name: 'Income', color: 'var(--success)', values: cashFlow.value.map((p) => num(p.income)) },
  { name: 'Expenses', color: 'var(--danger)', values: cashFlow.value.map((p) => num(p.expense)) }
])

const cfRecent = computed(() => cashFlow.value.slice(-6).reverse())

/* ------------------------------------------------------------------ *
 * 4 · spending
 * ------------------------------------------------------------------ */

const spendMonths = computed<BarDatum[]>(() => {
  const map = new Map<string, number>()
  for (const r of analysis.value?.monthly_expenses ?? []) {
    const month = String(r?.month ?? '').trim()
    if (!month) continue
    map.set(month, (map.get(month) ?? 0) + num(r.amount))
  }
  return [...map.entries()]
    .sort((a, b) => a[0].localeCompare(b[0]))
    .slice(-6)
    .map(([month, value]) => ({ label: shortMonth(month), value }))
})

const categoryRows = computed(() => {
  const rows = (analysis.value?.expense_categories ?? []).filter((c) => !!c?.category)
  return rows.slice().sort((a, b) => num(b.total) - num(a.total))
})

const categoryBars = computed<BarDatum[]>(() =>
  categoryRows.value.slice(0, 8).map((c) => ({ label: prettify(c.category), value: num(c.total) }))
)

const categoryTotal = computed(() => categoryRows.value.reduce((acc, c) => acc + num(c.total), 0))

/* ------------------------------------------------------------------ *
 * 5 · account attribution
 * ------------------------------------------------------------------ */

const UNLINKED_NAME = 'Unlinked'

const attribution = computed(() =>
  (analysis.value?.account_attribution ?? [])
    .filter((r) => !!r)
    .map((r) => ({
      account_id: r.account_id ?? null,
      name: prettify(r.account_name) || UNLINKED_NAME,
      income: num(r.income),
      expense: num(r.expense),
      net: num(r.net) || num(r.income) - num(r.expense),
      count: num(r.count),
      unlinked: !r.account_id
    }))
)

const linkedAccounts = computed(() =>
  attribution.value
    .filter((a) => !a.unlinked)
    .sort((a, b) => Math.max(b.income, b.expense) - Math.max(a.income, a.expense))
    .slice(0, 6)
)

const linkedIncomeBars = computed<BarDatum[]>(() =>
  linkedAccounts.value.map((a) => ({ label: a.name, value: a.income, color: 'var(--success)' }))
)

const linkedExpenseBars = computed<BarDatum[]>(() =>
  linkedAccounts.value.map((a) => ({ label: a.name, value: a.expense, color: 'var(--danger)' }))
)

const unlinked = computed(() => {
  const rows = attribution.value.filter((a) => a.unlinked)
  return rows.reduce(
    (acc, r) => ({
      income: acc.income + r.income,
      expense: acc.expense + r.expense,
      net: acc.net + r.net,
      count: acc.count + r.count
    }),
    { income: 0, expense: 0, net: 0, count: 0 }
  )
})

const unlinkedShare = computed(() => {
  const total = attribution.value.reduce((acc, a) => acc + a.income + a.expense, 0)
  if (total <= 0) return 0
  return ((unlinked.value.income + unlinked.value.expense) / total) * 100
})

/* ------------------------------------------------------------------ *
 * 6 · investments, cards, debts, deposits
 * ------------------------------------------------------------------ */

const roiRows = computed(() => (analysis.value?.roi.by_type ?? []).filter((r) => !!r?.investment_type))

const roiBars = computed<BarDatum[]>(() =>
  roiRows.value
    .slice()
    .sort((a, b) => num(b.gain_loss) - num(a.gain_loss))
    .map((r) => ({
      label: prettify(r.investment_type),
      value: num(r.gain_loss),
      color: num(r.gain_loss) >= 0 ? 'var(--success)' : 'var(--danger)'
    }))
)

const cards = computed(() =>
  (analysis.value?.credit_card_utilization ?? [])
    .slice()
    .sort((a, b) => num(b.utilization_pct) - num(a.utilization_pct))
)

function utilTone(pct: number): string {
  if (pct >= 75) return 'var(--danger)'
  if (pct >= 40) return 'var(--warning)'
  return 'var(--success)'
}

function utilBadge(pct: number): string {
  if (pct >= 75) return 'badge-danger'
  if (pct >= 40) return 'badge-warning'
  return 'badge-success'
}

const debts = computed(() => analysis.value?.debts ?? null)

const hasDebtsRoute = computed(() => router.getRoutes().some((r) => r.path === '/debts'))

const fdTimeline = computed(() =>
  (analysis.value?.fd_maturity_timeline ?? []).slice().sort((a, b) => num(a.days_to_maturity) - num(b.days_to_maturity))
)

/* ------------------------------------------------------------------ *
 * actions
 * ------------------------------------------------------------------ */

function toggleType(t: EntityType) {
  selectedTypes.value = selectedTypes.value.includes(t)
    ? selectedTypes.value.filter((x) => x !== t)
    : [...selectedTypes.value, t]
}

async function reload() {
  await finance.fetchAnalysisFiltered(selectedTypes.value)
}

watch(selectedTypes, () => {
  void reload()
})

onMounted(() => {
  void finance.fetchAnalysis()
})
</script>

<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div>
            <h1 class="page-title">Insights</h1>
            <p class="page-subtitle">Computed from every account, deposit, investment and transaction</p>
          </div>
          <SyncChip :status="sync.status" :pending="sync.queueCount" @sync="reload()" />
        </header>

        <div class="chips ff-hide-scrollbar">
          <button class="chip" :class="{ 'chip-active': selectedTypes.length === 0 }" type="button" @click="selectedTypes = []">
            All
          </button>
          <button
            v-for="t in FILTER_TYPES"
            :key="t"
            class="chip"
            :class="{ 'chip-active': selectedTypes.includes(t) }"
            type="button"
            @click="toggleType(t)"
          >
            {{ ENTITY_SNAKE[t].replace(/_/g, ' ') }}
          </button>
        </div>

        <div v-if="finance.analysisError" class="form-error">{{ finance.analysisError }}</div>

        <div v-if="finance.analysisLoading && !analysis" class="loading-note">Crunching your numbers…</div>

        <template v-else-if="analysis">
          <!-- ============================ 1 · overview ============================ -->
          <div class="group">
            <h2 class="group-title">Overview</h2>

            <section class="card-hero">
              <p class="card-label">Net worth</p>
              <p class="card-value">{{ formatMoney(netWorthTotal) }}</p>
              <p class="card-foot">
                {{ selectedTypes.length ? 'Filtered to ' + selectedTypes.length + ' entity type(s)' : 'All account groups' }}
              </p>
            </section>

            <div class="pair">
              <section class="block">
                <div class="section-head">
                  <h3 class="section-title">Where it sits</h3>
                  <span class="section-note">of {{ formatCompact(filteredNetWorth) }}</span>
                </div>
                <div class="card">
                  <DonutChart
                    :slices="netWorthSlices"
                    :center-value="formatCompact(filteredNetWorth)"
                    :center-label="selectedTypes.length ? 'filtered' : 'net worth'"
                    empty-text="No balances to break down yet."
                  />
                </div>
              </section>

              <section class="block">
                <div class="section-head">
                  <h3 class="section-title">Yearly income by type</h3>
                  <span class="section-note">{{ formatCompact(analysis.yearly_income_by_type.total) }}</span>
                </div>
                <div class="card">
                  <DonutChart
                    :slices="incomeSlices"
                    :center-value="formatCompact(analysis.yearly_income_by_type.total)"
                    center-label="this year"
                    empty-text="No income recorded this year."
                  />
                </div>
              </section>
            </div>

            <section class="block">
              <div class="section-head">
                <h3 class="section-title">Savings rate</h3>
                <span class="section-note" :class="savingsPct >= 0 ? 'text-success' : 'text-danger'">
                  {{ formatMoney(savings?.savings ?? 0) }} kept
                </span>
              </div>
              <div class="card ring-row">
                <ProgressRing
                  :value="savingsRingValue"
                  :size="118"
                  :thickness="11"
                  :color="savingsTone"
                  :label="formatPercent(savingsPct)"
                  sublabel="of income"
                />
                <div class="meta-list grow">
                  <div class="meta-row">
                    <span class="meta-key">Total income</span>
                    <span class="meta-val">{{ formatMoney(savings?.total_income ?? 0) }}</span>
                  </div>
                  <div class="meta-row">
                    <span class="meta-key">Total expense</span>
                    <span class="meta-val">{{ formatMoney(savings?.total_expense ?? 0) }}</span>
                  </div>
                  <div class="meta-row">
                    <span class="meta-key">Saved</span>
                    <span class="meta-val" :class="savingsPct >= 0 ? 'text-success' : 'text-danger'">
                      {{ formatMoney(savings?.savings ?? 0) }}
                    </span>
                  </div>
                </div>
              </div>
              <p class="text-sm text-muted cal-note">
                The ring is clamped to 0–100%. The real figure is
                <strong :class="savingsPct >= 0 ? 'text-success' : 'text-danger'">{{ formatPercent(savingsPct) }}</strong>.
              </p>
            </section>
          </div>

          <!-- ============================ 2 · activity ============================ -->
          <div class="group">
            <h2 class="group-title">Activity</h2>

            <section class="block">
              <div class="section-head">
                <h3 class="section-title">Daily activity</h3>
                <span class="section-note">{{ calTotals.activeDays }} active days</span>
              </div>

              <div class="chips ff-hide-scrollbar">
                <button
                  v-for="p in RANGE_PRESETS"
                  :key="p.key"
                  class="chip"
                  :class="{ 'chip-active': range === p.key }"
                  type="button"
                  @click="range = p.key"
                >
                  {{ p.label }}
                </button>
              </div>

              <div class="chips ff-hide-scrollbar">
                <button
                  class="chip"
                  :class="{ 'chip-active': calMode === 'total' }"
                  type="button"
                  @click="calMode = 'total'"
                >
                  Earn + spend
                </button>
                <button
                  class="chip"
                  :class="{ 'chip-active': calMode === 'expense' }"
                  type="button"
                  @click="calMode = 'expense'"
                >
                  Spend only
                </button>
              </div>

              <div class="card">
                <CalendarHeatmap
                  :days="calDays"
                  :weeks="heatWeeks"
                  :format-value="(n: number) => formatCompact(n)"
                  empty-text="No transactions in this window. Try a wider range."
                />
                <div class="cal-stats">
                  <div class="cal-stat">
                    <span class="cal-stat-label">In</span>
                    <span class="cal-stat-value text-success">{{ formatCompact(calTotals.income) }}</span>
                  </div>
                  <div class="cal-stat">
                    <span class="cal-stat-label">Out</span>
                    <span class="cal-stat-value text-danger">{{ formatCompact(calTotals.expense) }}</span>
                  </div>
                  <div class="cal-stat">
                    <span class="cal-stat-label">Net</span>
                    <span class="cal-stat-value" :class="calTotals.net >= 0 ? 'text-success' : 'text-danger'">
                      {{ formatMoney(calTotals.net) }}
                    </span>
                  </div>
                </div>
                <p class="text-sm text-muted cal-note">
                  <template v-if="calMode === 'total'">
                    Darker squares are busier days — total money earned <em>and</em> spent. Peak
                    {{ formatCompact(calTotals.peak) }} in one day.
                  </template>
                  <template v-else>
                    Darker squares are bigger spending days. Peak
                    {{ formatCompact(calTotals.peak) }} out in one day.
                  </template>
                </p>
              </div>
            </section>

            <section class="block">
              <div class="section-head">
                <h3 class="section-title">Cash flow</h3>
                <div class="legend">
                  <span class="legend-item"><span class="legend-dot" style="background: var(--success)" /> Income</span>
                  <span class="legend-item"><span class="legend-dot" style="background: var(--danger)" /> Expenses</span>
                </div>
              </div>
              <div class="card">
                <LineChart
                  v-if="cashFlow.length"
                  :series="cfSeries"
                  :labels="cfLabels"
                  :height="230"
                  :format-value="(n: number) => formatCompact(n)"
                />
                <p v-else class="empty-text">No cash-flow history yet. Add income and expense transactions first.</p>

                <div v-if="cfRecent.length" class="cf-table">
                  <div class="cf-row cf-head">
                    <span>Month</span>
                    <span>In</span>
                    <span>Out</span>
                    <span>Net</span>
                  </div>
                  <div v-for="p in cfRecent" :key="p.month" class="cf-row">
                    <span class="cf-label">{{ formatMonth(p.month) }}</span>
                    <span class="cf-num">{{ formatCompact(p.income) }}</span>
                    <span class="cf-num">{{ formatCompact(p.expense) }}</span>
                    <span class="cf-num" :class="p.net >= 0 ? 'text-success' : 'text-danger'">
                      {{ formatMoney(p.net) }}
                    </span>
                  </div>
                </div>
              </div>
            </section>
          </div>

          <!-- ============================ 3 · spending ============================ -->
          <div class="group">
            <h2 class="group-title">Spending</h2>

            <div class="pair">
              <section class="block">
                <div class="section-head">
                  <h3 class="section-title">Monthly spend</h3>
                  <span class="section-note">last {{ spendMonths.length }} months</span>
                </div>
                <div class="card">
                  <BarChart
                    v-if="spendMonths.length"
                    :data="spendMonths"
                    :height="210"
                    :format-value="(n: number) => formatCompact(n)"
                  />
                  <p v-else class="empty-text">No monthly expense history yet.</p>
                </div>
              </section>

              <section class="block">
                <div class="section-head">
                  <h3 class="section-title">Spend by category</h3>
                  <span class="section-note">{{ formatCompact(categoryTotal) }}</span>
                </div>
                <div class="card">
                  <BarChart
                    v-if="categoryBars.length"
                    :data="categoryBars"
                    :height="210"
                    :format-value="(n: number) => formatCompact(n)"
                  />
                  <p v-else class="empty-text">No expense categories yet.</p>
                </div>
              </section>
            </div>

            <div class="pair">
              <section class="block">
                <div class="section-head">
                  <h3 class="section-title">Top expenses</h3>
                </div>
                <div class="row-list">
                  <div v-for="(t, i) in analysis.top_5_expenses" :key="`te-${i}`" class="row-item">
                    <span class="row-main">
                      <span class="row-title clamp-1">{{ t.label }}</span>
                      <span class="row-sub">{{ t.category ? prettify(t.category) : 'Uncategorised' }}</span>
                    </span>
                    <span class="row-value text-danger">{{ formatMoney(t.amount) }}</span>
                  </div>
                  <p v-if="!analysis.top_5_expenses.length" class="empty-text">No expenses recorded.</p>
                </div>
              </section>

              <section class="block">
                <div class="section-head">
                  <h3 class="section-title">Top income sources</h3>
                </div>
                <div class="row-list">
                  <div v-for="(t, i) in analysis.top_5_income_sources" :key="`ti-${i}`" class="row-item">
                    <span class="row-main">
                      <span class="row-title clamp-1">{{ t.label }}</span>
                      <span class="row-sub">{{ t.category ? prettify(t.category) : '—' }}</span>
                    </span>
                    <span class="row-value text-success">{{ formatMoney(t.amount) }}</span>
                  </div>
                  <p v-if="!analysis.top_5_income_sources.length" class="empty-text">No income recorded.</p>
                </div>
              </section>
            </div>
          </div>

          <!-- ========================= 4 · account attribution ========================= -->
          <div class="group">
            <h2 class="group-title">Which account moved the money</h2>

            <div v-if="unlinked.count > 0" class="notice notice-warning unlinked">
              <span class="unlinked-head">
                <ion-icon :icon="linkOutline" />
                {{ formatNumber(unlinked.count) }} transactions carry no bank account
              </span>
              <span class="unlinked-body">
                {{ formatMoney(unlinked.income) }} in and {{ formatMoney(unlinked.expense) }} out
                ({{ formatPercent(unlinkedShare) }} of all activity) is unattributed. Edit those rows and pick a bank
                account so per-account reporting balances.
              </span>
              <RouterLink class="unlinked-cta" to="/transactions">Fix in transactions</RouterLink>
            </div>

            <template v-if="linkedAccounts.length">
              <div class="pair">
                <section class="block">
                  <div class="section-head">
                    <h3 class="section-title">Income per account</h3>
                  </div>
                  <div class="card">
                    <BarChart
                      :data="linkedIncomeBars"
                      :height="200"
                      :format-value="(n: number) => formatCompact(n)"
                    />
                  </div>
                </section>

                <section class="block">
                  <div class="section-head">
                    <h3 class="section-title">Spend per account</h3>
                  </div>
                  <div class="card">
                    <BarChart
                      :data="linkedExpenseBars"
                      :height="200"
                      :format-value="(n: number) => formatCompact(n)"
                    />
                  </div>
                </section>
              </div>

              <section class="block">
                <div class="section-head">
                  <h3 class="section-title">Net per account</h3>
                </div>
                <div class="card">
                  <div class="meta-list">
                    <div v-for="a in linkedAccounts" :key="a.name" class="meta-row">
                      <span class="meta-key clamp-1">{{ a.name }}</span>
                      <span class="meta-val" :class="a.net >= 0 ? 'text-success' : 'text-danger'">
                        {{ formatMoney(a.net) }}
                      </span>
                    </div>
                  </div>
                </div>
              </section>
            </template>

            <p v-else-if="!unlinked.count" class="empty-text">
              No per-account attribution yet — transactions need a bank account to show up here.
            </p>
          </div>

          <!-- ============================ 5 · positions ============================ -->
          <div class="group">
            <h2 class="group-title">Positions</h2>

            <section class="block">
              <div class="section-head">
                <h3 class="section-title">Investment return</h3>
                <span class="badge" :class="analysis.roi.roi_percentage >= 0 ? 'badge-success' : 'badge-danger'">
                  {{ formatPercent(analysis.roi.roi_percentage) }}
                </span>
              </div>
              <div class="card">
                <div class="stat-grid">
                  <div class="stat-tile">
                    <span class="stat-tile-label">Invested</span>
                    <span class="stat-tile-value">{{ formatCompact(analysis.roi.total_invested) }}</span>
                  </div>
                  <div class="stat-tile">
                    <span class="stat-tile-label">Current value</span>
                    <span class="stat-tile-value">{{ formatCompact(analysis.roi.total_current_value) }}</span>
                  </div>
                  <div class="stat-tile">
                    <span class="stat-tile-label">Gain / loss</span>
                    <span class="stat-tile-value" :class="analysis.roi.total_gain_loss >= 0 ? 'text-success' : 'text-danger'">
                      {{ formatMoney(analysis.roi.total_gain_loss) }}
                    </span>
                  </div>
                  <div class="stat-tile">
                    <span class="stat-tile-label">Types</span>
                    <span class="stat-tile-value">{{ formatNumber(roiRows.length) }}</span>
                  </div>
                </div>
                <div class="chart-split">
                  <BarChart
                    v-if="roiBars.length"
                    :data="roiBars"
                    :height="200"
                    :format-value="(n: number) => formatCompact(n)"
                  />
                  <p v-else class="empty-text">No investments to evaluate yet.</p>
                </div>
              </div>
            </section>

            <section v-if="cards.length" class="block">
              <div class="section-head">
                <h3 class="section-title">Card utilisation</h3>
              </div>
              <div class="card">
                <div v-for="c in cards" :key="c.card_name" class="card-row">
                  <ProgressRing
                    :value="num(c.utilization_pct)"
                    :size="88"
                    :thickness="10"
                    :color="utilTone(num(c.utilization_pct))"
                    :label="formatPercent(c.utilization_pct, 0)"
                    sublabel="used"
                  />
                  <div class="grow">
                    <div class="card-row-head">
                      <span class="row-title clamp-1">{{ c.card_name }}</span>
                      <span class="badge" :class="utilBadge(num(c.utilization_pct))">
                        {{ formatPercent(c.utilization_pct, 0) }}
                      </span>
                    </div>
                    <p class="text-sm text-muted">
                      {{ formatMoney(c.balance) }} used · {{ formatMoney(c.available) }} free of
                      {{ formatMoney(c.limit) }}
                    </p>
                  </div>
                </div>
              </div>
            </section>

            <section v-if="debts && (debts.total_count > 0 || debts.overdue_count > 0)" class="block">
              <div class="section-head">
                <h3 class="section-title">Debts</h3>
                <span class="section-note">{{ formatNumber(debts.total_count) }} entries</span>
              </div>
              <div class="debt-grid">
                <div class="card">
                  <p class="card-label">Owed to me</p>
                  <p class="card-value-sm text-success">{{ formatMoney(debts.owed_to_me) }}</p>
                  <p class="card-foot">{{ formatNumber(debts.lent_count) }} lent</p>
                </div>
                <div class="card">
                  <p class="card-label">I owe</p>
                  <p class="card-value-sm text-danger">{{ formatMoney(debts.i_owe) }}</p>
                  <p class="card-foot">{{ formatNumber(debts.borrowed_count) }} borrowed</p>
                </div>
                <div class="card">
                  <p class="card-label">Net</p>
                  <p class="card-value-sm" :class="debts.net >= 0 ? 'text-success' : 'text-danger'">
                    {{ formatMoney(debts.net) }}
                  </p>
                  <p class="card-foot">{{ formatNumber(debts.settled_count) }} settled</p>
                </div>
              </div>
              <p v-if="debts.overdue_count > 0" class="notice notice-warning">
                {{ formatNumber(debts.overdue_count) }} overdue — settle those before the next cycle.
              </p>
              <RouterLink v-if="hasDebtsRoute" class="section-link" to="/debts">Open the debt book</RouterLink>
              <a v-else class="section-link" href="/debts">Open the debt book</a>
            </section>

            <section v-if="fdTimeline.length" class="block">
              <div class="section-head">
                <h3 class="section-title">Deposit maturities</h3>
                <span class="section-note">{{ formatNumber(fdTimeline.length) }} deposits</span>
              </div>
              <div class="row-list">
                <div v-for="fd in fdTimeline" :key="fd.id" class="row-item">
                  <span class="row-main">
                    <span class="row-title clamp-1">{{ fd.name }}</span>
                    <span class="row-sub">
                      {{ formatDate(fd.maturity_date) }} · {{ num(fd.interest_rate) }}% p.a. · principal
                      {{ formatMoney(fd.principal) }}
                    </span>
                  </span>
                  <span class="row-value">
                    {{ formatMoney(fd.current_value || fd.principal) }}
                    <span class="row-extra">{{ relativeDays(fd.maturity_date) }}</span>
                  </span>
                </div>
              </div>
            </section>

            <p v-if="!cards.length && !roiBars.length && !fdTimeline.length && !debts?.total_count" class="empty-text">
              No investments, cards, deposits or debts recorded yet.
            </p>
          </div>

          <AppButton variant="neutral" size="md" block :disabled="finance.analysisLoading" @click="reload">
            <ion-icon :icon="refreshOutline" /> Recalculate
          </AppButton>
        </template>

        <div v-else class="empty">
          <ion-icon class="empty-icon" :icon="analyticsOutline" />
          <p class="empty-title">Nothing to analyse yet</p>
          <p class="empty-text">
            <ion-icon :icon="trendingUpOutline" /> Add accounts and transactions, then recalculate.
          </p>
        </div>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
/* ---- grouping so the page does not read as 15 equal-weight blocks ---- */
.group {
  display: flex;
  flex-direction: column;
  gap: var(--density-section-gap);
  margin-top: calc(var(--density-section-gap) * 2);
  animation: an-group-in 0.26s ease both;
}

.block {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
  min-width: 0;
}

/* Two charts side by side once there is room; one column on a phone. */
.pair {
  display: grid;
  gap: var(--density-gap);
  grid-template-columns: minmax(0, 1fr);
}

@media (min-width: 760px) {
  .pair {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

.chart-split {
  margin-top: var(--density-section-gap);
  padding-top: var(--density-gap);
  border-top: 1px solid var(--border);
}

.grow {
  min-width: 0;
  flex: 1;
}

/* ---- overview ---- */
.ring-row {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  flex-wrap: wrap;
}

.ring-row .meta-list {
  margin: 0;
  min-width: 190px;
}

/* ---- calendar ---- */
.cal-stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--density-gap);
  margin-top: var(--density-section-gap);
  padding-top: var(--density-gap);
  border-top: 1px solid var(--border);
}

.cal-stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.cal-stat-label {
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-faint);
}

.cal-stat-value {
  font-size: 0.95rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cal-note {
  margin: var(--density-gap) 0 0;
  max-width: 62ch;
}

.cal-note strong {
  font-weight: 700;
}

/* ---- cash-flow table ---- */
.cf-table {
  margin-top: var(--density-section-gap);
  padding-top: var(--density-gap);
  border-top: 1px solid var(--border);
}

.cf-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto auto;
  gap: var(--density-gap);
  align-items: baseline;
  padding: var(--density-meta-pad);
  border-bottom: 1px solid var(--border);
  font-size: 0.82rem;
}

.cf-row:last-child {
  border-bottom: none;
}

.cf-head {
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--text-faint);
}

.cf-label {
  font-weight: 650;
  color: var(--text);
}

.cf-num {
  text-align: right;
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
}

/* ---- unlinked callout ---- */
.unlinked {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
  align-items: flex-start;
}

.unlinked-head {
  display: inline-flex;
  align-items: center;
  gap: var(--density-gap);
  font-weight: 700;
}

.unlinked-body {
  font-weight: 500;
  line-height: 1.45;
}

.unlinked-cta,
.section-link {
  display: inline-block;
  margin-top: 4px;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  border-radius: 999px;
  border: 1px solid currentColor;
  font-size: 0.78rem;
  font-weight: 700;
  color: var(--warning);
  text-decoration: none;
}

.section-link {
  align-self: flex-start;
  margin-top: 0;
  color: var(--accent-ink);
  border-color: var(--accent-soft);
  background: var(--accent-soft);
}

/* ---- card utilisation rows ---- */
.card-row {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  padding: var(--density-meta-pad);
  border-bottom: 1px solid var(--border);
}

.card-row:first-child {
  padding-top: 0;
}

.card-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.card-row-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--density-gap);
  margin-bottom: 2px;
}

.card-row p {
  margin: 0;
}

/* ---- debts ---- */
.debt-grid {
  display: grid;
  gap: var(--density-gap);
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
}

.debt-grid .card-value-sm {
  font-variant-numeric: tabular-nums;
}

.row-list .empty-text {
  padding: var(--density-card-pad);
}

/* ---- restrained entrance, disabled when the OS asks for less motion ---- */
@keyframes an-group-in {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .group {
    animation: none;
  }
}
</style>