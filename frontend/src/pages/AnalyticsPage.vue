<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import { analyticsOutline, refreshOutline, trendingUpOutline } from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import { useCountUp } from '@/composables/useCountUp'
import { ENTITY_SNAKE } from '@/entityConfig'
import type { EntityType } from '@/types'
import { formatCompact, formatMoney, formatNumber, formatPercent } from '@/utils/money'
import { formatDate, formatMonth, relativeDays } from '@/utils/date'
import AppButton from '@/components/ui/AppButton.vue'
import SyncChip from '@/components/ui/SyncChip.vue'

const finance = useFinanceStore()
const sync = useSyncStore()

const FILTER_TYPES: EntityType[] = ['banks', 'assets', 'credit-cards', 'fixed-deposits', 'investments', 'incomes', 'expenses']
const selectedTypes = ref<EntityType[]>([])

const analysis = computed(() => finance.analysis)

const netWorthBars = computed(() => {
  const nw = analysis.value?.net_worth
  if (!nw) return []
  return [
    { label: 'Banks', value: nw.banks },
    { label: 'Assets', value: nw.assets },
    { label: 'Deposits', value: nw.fixed_deposits },
    { label: 'Investments', value: nw.investments }
  ]
})

const incomeBars = computed(() => {
  const y = analysis.value?.yearly_income_by_type
  if (!y) return []
  return [
    { label: 'Salary', value: y.salary },
    { label: 'Business', value: y.business },
    { label: 'Investment', value: y.investment },
    { label: 'Other', value: y.other }
  ]
})

/** Only the net-worth components whose entity type is selected are counted. */
const filteredNetWorth = computed(() => {
  const nw = analysis.value?.net_worth
  if (!nw) return 0
  if (!selectedTypes.value.length) return nw.total
  const map: Array<[EntityType, number]> = [
    ['banks', nw.banks],
    ['assets', nw.assets],
    ['fixed-deposits', nw.fixed_deposits],
    ['investments', nw.investments]
  ]
  return map
    .filter(([t]) => selectedTypes.value.includes(t))
    .reduce((acc, [, v]) => acc + v, 0)
})

const netWorthTotal = useCountUp(computed(() => filteredNetWorth.value))
const savingsPct = computed(() => analysis.value?.savings_rate.savings_rate_pct ?? 0)

const savingsBarPct = computed(() => {
  const p = savingsPct.value
  if (!Number.isFinite(p)) return 0
  return Math.max(0, Math.min(100, p))
})

function max(values: number[]): number {
  return values.reduce((a, b) => (b > a ? b : a), 0)
}

const nwMax = computed(() => max(netWorthBars.value.map((b) => b.value)))
const incomeMax = computed(() => max(incomeBars.value.map((b) => b.value)))

function widthPct(value: number, top: number): string {
  if (!top || !Number.isFinite(value)) return '0%'
  return `${Math.max(0, Math.min(100, (value / top) * 100))}%`
}

const expenseCategoryMax = computed(() =>
  max((analysis.value?.top_5_expenses ?? []).map((t) => t.amount))
)
const incomeSourceMax = computed(() =>
  max((analysis.value?.top_5_income_sources ?? []).map((t) => t.amount))
)

/* ---- cash-flow chart ---- */

const CHART_W = 320
const CHART_H = 140
const PAD_BOTTOM = 22
const chartPoints = computed(() => analysis.value?.cash_flow ?? [])

const chartMax = computed(() => Math.max(1, max(chartPoints.value.flatMap((p) => [p.income, p.expense]))))

interface Bar {
  x: number
  w: number
  h: number
  value: number
}

function barsFor(key: 'income' | 'expense'): Bar[] {
  const pts = chartPoints.value
  if (!pts.length) return []
  const slot = CHART_W / pts.length
  const w = Math.max(2, slot / 2 - 1.5)
  return pts.map((p, i) => {
    const value = key === 'income' ? p.income : p.expense
    const h = (value / chartMax.value) * (CHART_H - PAD_BOTTOM)
    return { x: i * slot + slot / 2 - w - 1, w, h, value }
  })
}

const incomeBarsChart = computed(() => barsFor('income'))
const expenseBarsChart = computed(() => barsFor('expense'))

const chartGridLines = computed(() => {
  const out: Array<{ y: number; label: string }> = []
  for (let i = 0; i <= 2; i++) {
    const frac = i / 2
    out.push({ y: frac * (CHART_H - PAD_BOTTOM), label: formatCompact(chartMax.value * (1 - frac)) })
  }
  return out
})

const chartLabels = computed(() => {
  const pts = chartPoints.value
  const every = Math.max(1, Math.ceil(pts.length / 6))
  return pts
    .map((p, i) => ({ i, label: i % every === 0 ? formatMonth(p.month) : '' }))
    .filter((x) => x.label)
})

function labelX(index: number): number {
  const pts = chartPoints.value
  const slot = CHART_W / Math.max(1, pts.length)
  return index * slot + slot / 2
}

/* ---- monthly expenses by category ---- */

const monthlyExpenseMonths = computed(() => {
  const rows = analysis.value?.monthly_expenses ?? []
  const set = new Map<string, number>()
  for (const r of rows) set.set(r.month, (set.get(r.month) ?? 0) + r.amount)
  return [...set.entries()].sort((a, b) => a[0].localeCompare(b[0])).slice(-12)
})

const monthlyExpenseMax = computed(() => Math.max(1, max(monthlyExpenseMonths.value.map(([, v]) => v))))

const expenseTypeTotals = computed(() => {
  const rows = analysis.value?.monthly_expenses ?? []
  const map = new Map<string, number>()
  for (const r of rows) map.set(r.category, (map.get(r.category) ?? 0) + r.amount)
  return [...map.entries()].sort((a, b) => b[1] - a[1]).slice(0, 8)
})

/* ---- ROI ---- */

const roiRows = computed(() => {
  const roi = analysis.value?.roi
  if (!roi) return []
  return [...roi.by_type].sort((a, b) => b.gain_loss - a.gain_loss)
})

const roiMaxInvested = computed(() => Math.max(1, max(roiRows.value.map((r) => r.invested))))

/* ---- FD + cards ---- */

const fdTimeline = computed(() =>
  [...(analysis.value?.fd_maturity_timeline ?? [])].sort((a, b) => a.days_to_maturity - b.days_to_maturity)
)

const cards = computed(() => [...(analysis.value?.credit_card_utilization ?? [])].sort((a, b) => b.utilization_pct - a.utilization_pct))

function utilTone(pct: number): string {
  if (pct >= 75) return 'badge-danger'
  if (pct >= 40) return 'badge-warning'
  return 'badge-success'
}

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
          <!-- Net worth -->
          <section class="card-hero">
            <p class="card-label">Net worth</p>
            <p class="card-value">{{ formatMoney(netWorthTotal) }}</p>
            <p class="card-foot">
              {{ selectedTypes.length ? 'Filtered to ' + selectedTypes.length + ' entity type(s)' : 'All account groups' }}
            </p>
          </section>

          <section class="section">
            <div class="section-head">
              <h2 class="section-title">Where it sits</h2>
              <span class="section-note">of {{ formatMoney(analysis.net_worth.total) }}</span>
            </div>
            <div class="card">
              <div v-for="b in netWorthBars" :key="b.label" class="bar-track">
                <div class="bar-head">
                  <span class="bar-name">{{ b.label }}</span>
                  <span class="bar-amount">{{ formatCompact(b.value) }}</span>
                </div>
                <div class="bar">
                  <div class="bar-fill" :style="{ width: widthPct(b.value, nwMax) }" />
                </div>
              </div>
            </div>
          </section>

          <!-- Savings rate -->
          <section class="section">
            <h2 class="section-title">Savings rate</h2>
            <div class="card">
              <div class="savings-head">
                <span class="savings-value" :class="savingsPct >= 0 ? 'text-success' : 'text-danger'">
                  {{ formatPercent(savingsPct) }}
                </span>
                <span class="text-sm text-muted">
                  {{ formatMoney(analysis.savings_rate.savings) }} saved
                </span>
              </div>
              <div class="bar" style="margin-top: 10px">
                <div
                  class="bar-fill"
                  :style="{ width: `${savingsBarPct}%`, background: savingsPct >= 0 ? 'var(--success)' : 'var(--danger)' }"
                />
              </div>
              <div class="savings-foot">
                <span>In {{ formatMoney(analysis.savings_rate.total_income) }}</span>
                <span>Out {{ formatMoney(analysis.savings_rate.total_expense) }}</span>
              </div>
            </div>
          </section>

          <!-- Income mix -->
          <section class="section">
            <div class="section-head">
              <h2 class="section-title">Yearly income by type</h2>
              <span class="section-note">{{ formatMoney(analysis.yearly_income_by_type.total) }}</span>
            </div>
            <div class="card">
              <div v-for="b in incomeBars" :key="b.label" class="bar-track">
                <div class="bar-head">
                  <span class="bar-name">{{ b.label }}</span>
                  <span class="bar-amount">{{ formatCompact(b.value) }}</span>
                </div>
                <div class="bar">
                  <div class="bar-fill" :style="{ width: widthPct(b.value, incomeMax), background: 'var(--success)' }" />
                </div>
              </div>
            </div>
          </section>

          <!-- Cash flow -->
          <section class="section">
            <div class="section-head">
              <h2 class="section-title">Cash flow</h2>
              <div class="legend" style="margin: 0">
                <span class="legend-item"><span class="legend-dot" style="background: var(--success)" /> Income</span>
                <span class="legend-item"><span class="legend-dot" style="background: var(--danger)" /> Expense</span>
              </div>
            </div>
            <div class="card">
              <div v-if="!chartPoints.length" class="empty-text">No cash-flow data yet.</div>
              <svg v-else class="chart" :viewBox="`0 0 ${CHART_W} ${CHART_H}`" role="img" aria-label="Monthly income versus expenses">
                <g v-for="(g, i) in chartGridLines" :key="i">
                  <line class="chart-grid" :x1="0" :y1="g.y" :x2="CHART_W" :y2="g.y" />
                  <text class="chart-axis-text" :x="2" :y="g.y - 3">{{ g.label }}</text>
                </g>
                <rect v-for="(b, i) in incomeBarsChart" :key="`in-${i}`" :x="b.x" :y="CHART_H - PAD_BOTTOM - b.h" :width="b.w" :height="b.h" fill="var(--success)" rx="1.5" />
                <rect v-for="(b, i) in expenseBarsChart" :key="`ex-${i}`" :x="b.x + b.w + 2" :y="CHART_H - PAD_BOTTOM - b.h" :width="b.w" :height="b.h" fill="var(--danger)" rx="1.5" />
                <text v-for="l in chartLabels" :key="`lb-${l.i}`" class="chart-axis-text" :x="labelX(l.i)" :y="CHART_H - 8" text-anchor="middle">
                  {{ l.label }}
                </text>
              </svg>
              <div v-if="chartPoints.length" class="cf-list">
                <div v-for="p in chartPoints.slice(-6).reverse()" :key="p.month" class="meta-row">
                  <span class="meta-key">{{ formatMonth(p.month) }}</span>
                  <span class="meta-val" :class="p.net >= 0 ? 'text-success' : 'text-danger'">
                    {{ formatMoney(p.net) }}
                  </span>
                </div>
              </div>
            </div>
          </section>

          <!-- Top lists -->
          <section class="section">
            <h2 class="section-title">Top expenses</h2>
            <div class="row-list">
              <div v-for="(t, i) in analysis.top_5_expenses" :key="`te-${i}`" class="top-row">
                <span class="top-rank">{{ i + 1 }}</span>
                <span class="row-main">
                  <span class="row-title clamp-1">{{ t.label }}</span>
                  <span class="row-sub">{{ t.category ? t.category.replace(/_/g, ' ') : 'Uncategorised' }}</span>
                </span>
                <span class="top-amount">
                  {{ formatMoney(t.amount) }}
                  <span class="bar" style="margin-top: 4px">
                    <span class="bar-fill" :style="{ width: widthPct(t.amount, expenseCategoryMax), background: 'var(--danger)' }" />
                  </span>
                </span>
              </div>
              <div v-if="!analysis.top_5_expenses.length" class="empty-text">No expenses recorded.</div>
            </div>
          </section>

          <section class="section">
            <h2 class="section-title">Top income sources</h2>
            <div class="row-list">
              <div v-for="(t, i) in analysis.top_5_income_sources" :key="`ti-${i}`" class="top-row">
                <span class="top-rank">{{ i + 1 }}</span>
                <span class="row-main">
                  <span class="row-title clamp-1">{{ t.label }}</span>
                  <span class="row-sub">{{ t.category ? t.category.replace(/_/g, ' ') : '—' }}</span>
                </span>
                <span class="top-amount">
                  {{ formatMoney(t.amount) }}
                  <span class="bar" style="margin-top: 4px">
                    <span class="bar-fill" :style="{ width: widthPct(t.amount, incomeSourceMax), background: 'var(--success)' }" />
                  </span>
                </span>
              </div>
              <div v-if="!analysis.top_5_income_sources.length" class="empty-text">No income recorded.</div>
            </div>
          </section>

          <!-- Monthly expenses -->
          <section class="section">
            <div class="section-head">
              <h2 class="section-title">Monthly spend</h2>
              <span class="section-note">last {{ monthlyExpenseMonths.length }} months</span>
            </div>
            <div class="card">
              <div v-for="[month, total] in monthlyExpenseMonths" :key="month" class="bar-track">
                <div class="bar-head">
                  <span class="bar-name">{{ formatMonth(month) }}</span>
                  <span class="bar-amount">{{ formatCompact(total) }}</span>
                </div>
                <div class="bar">
                  <div class="bar-fill" :style="{ width: widthPct(total, monthlyExpenseMax), background: 'var(--warning)' }" />
                </div>
              </div>
              <div v-if="!monthlyExpenseMonths.length" class="empty-text">No expense history yet.</div>
            </div>
          </section>

          <section v-if="expenseTypeTotals.length" class="section">
            <h2 class="section-title">Spend by category</h2>
            <div class="card">
              <div v-for="[cat, total] in expenseTypeTotals" :key="cat" class="meta-row">
                <span class="meta-key">{{ cat.replace(/_/g, ' ') }}</span>
                <span class="meta-val">{{ formatMoney(total) }}</span>
              </div>
            </div>
          </section>

          <!-- ROI -->
          <section class="section">
            <div class="section-head">
              <h2 class="section-title">Return on investment</h2>
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
                    {{ formatCompact(analysis.roi.total_gain_loss) }}
                  </span>
                </div>
                <div class="stat-tile">
                  <span class="stat-tile-label">Positions</span>
                  <span class="stat-tile-value">{{ formatNumber(roiRows.length) }}</span>
                </div>
              </div>
              <div v-if="roiRows.length" class="roi-list">
                <div v-for="r in roiRows" :key="r.investment_type" class="bar-track">
                  <div class="bar-head">
                    <span class="bar-name">{{ r.investment_type.replace(/_/g, ' ') }}</span>
                    <span class="bar-amount" :class="r.gain_loss >= 0 ? 'text-success' : 'text-danger'">
                      {{ formatPercent(r.roi_pct) }}
                    </span>
                  </div>
                  <div class="bar">
                    <div
                      class="bar-fill"
                      :style="{
                        width: widthPct(r.invested, roiMaxInvested),
                        background: r.gain_loss >= 0 ? 'var(--success)' : 'var(--danger)'
                      }"
                    />
                  </div>
                </div>
              </div>
              <div v-else class="empty-text">No investments to evaluate.</div>
            </div>
          </section>

          <!-- FD timeline -->
          <section v-if="fdTimeline.length" class="section">
            <h2 class="section-title">Deposit maturities</h2>
            <div class="row-list">
              <div v-for="fd in fdTimeline" :key="fd.id" class="top-row">
                <span class="row-main">
                  <span class="row-title clamp-1">{{ fd.name }}</span>
                  <span class="row-sub">{{ formatDate(fd.maturity_date) }} · {{ fd.interest_rate }}% p.a.</span>
                </span>
                <span class="top-amount">
                  {{ formatCompact(fd.current_value || fd.principal) }}
                  <span class="row-extra">{{ relativeDays(fd.maturity_date) }}</span>
                </span>
              </div>
            </div>
          </section>

          <!-- Credit cards -->
          <section v-if="cards.length" class="section">
            <h2 class="section-title">Card utilisation</h2>
            <div class="card">
              <div v-for="c in cards" :key="c.card_name" class="bar-track">
                <div class="bar-head">
                  <span class="bar-name">{{ c.card_name }}</span>
                  <span class="badge" :class="utilTone(c.utilization_pct)">{{ formatPercent(c.utilization_pct, 0) }}</span>
                </div>
                <div class="bar">
                  <div class="bar-fill" :style="{ width: widthPct(c.utilization_pct, 100) }" />
                </div>
                <span class="row-extra">
                  {{ formatMoney(c.balance) }} used · {{ formatMoney(c.available) }} available of {{ formatMoney(c.limit) }}
                </span>
              </div>
            </div>
          </section>

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
.loading-note {
  padding: 26px;
  text-align: center;
  color: var(--text-muted);
  font-size: 0.88rem;
}
.savings-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}
.savings-value {
  font-size: 1.6rem;
  font-weight: 700;
}
.savings-foot {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
  font-size: 0.8rem;
  color: var(--text-muted);
}
.cf-list {
  margin-top: 12px;
  border-top: 1px solid var(--border);
  padding-top: 4px;
}
.top-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 13px;
  border-bottom: 1px solid var(--border);
}
.top-row:last-child {
  border-bottom: none;
}
.top-rank {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--surface-2);
  border: 1px solid var(--border);
  font-size: 0.72rem;
  font-weight: 700;
  color: var(--text-muted);
}
.top-amount {
  flex-shrink: 0;
  min-width: 96px;
  text-align: right;
  font-size: 0.88rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}
.roi-list {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
</style>