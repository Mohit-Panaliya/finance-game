<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { IonPage, IonContent, IonHeader } from '@ionic/vue'
import SyncChip from '@/components/game/SyncChip.vue'
import GameDatePicker from '@/components/game/GameDatePicker.vue'
import GameSelect from '@/components/game/GameSelect.vue'
import type { GameSelectOption } from '@/components/game/GameSelect.vue'
import { api } from '@/api/client'
import { formatGold } from '@/entityConfig'

interface NetWorth {
  banks: number
  assets: number
  fixed_deposits: number
  investments: number
  total: number
}
interface YearlyIncome {
  salary: number
  business: number
  investment: number
  other: number
  total: number
}
interface MonthlyExpense {
  month: string
  category: string
  amount: number
}
interface RoiByType {
  type?: string
  invested?: number
  current_value?: number
  gain_loss?: number
  roi_percentage?: number
}
interface Roi {
  total_invested: number
  total_current_value: number
  total_gain_loss: number
  roi_percentage: number
  by_type: RoiByType[]
}
interface CashFlowPoint {
  month: string
  income: number
  expense: number
  net: number
}
interface TopItem {
  label: string
  amount: number
  category: string | null
}
interface InvestmentPerf {
  name?: string
  type?: string
  invested?: number
  current_value?: number
  gain_loss?: number
  roi_percentage?: number
}
interface FdMaturity {
  id: string
  bank_name?: string
  principal?: number
  current_value?: number
  maturity_date?: string
  days_left?: number
  interest_rate?: number
}
interface CreditCardUtil {
  card_name: string
  limit: number
  balance: number
  available: number
  utilization_pct: number
}
interface SavingsRate {
  total_income: number
  total_expense: number
  savings: number
  savings_rate_pct: number
}
interface AnalysisData {
  net_worth: NetWorth
  yearly_income_by_type: YearlyIncome
  monthly_expenses: MonthlyExpense[]
  roi: Roi
  cash_flow: CashFlowPoint[]
  top_5_expenses: TopItem[]
  top_5_income_sources: TopItem[]
  investment_performance: InvestmentPerf[]
  fd_maturity_timeline: FdMaturity[]
  credit_card_utilization: CreditCardUtil[]
  savings_rate: SavingsRate
}

const router = useRouter()

const tabs = [
  { key: 'overview', label: 'Overview', avatar: '🧙' },
  { key: 'income', label: 'Income', avatar: '💰' },
  { key: 'expenses', label: 'Expenses', avatar: '🛡️' },
  { key: 'investments', label: 'Invest', avatar: '🏰' },
  { key: 'cashflow', label: 'Cash Flow', avatar: '⚖️' }
] as const

const tab = ref<(typeof tabs)[number]['key']>('overview')
const data = ref<AnalysisData | null>(null)
const loading = ref(false)
const error = ref('')

const startDate = ref('')
const endDate = ref('')
const startPickerOpen = ref(false)
const endPickerOpen = ref(false)
const category = ref<string | null>(null)

const categoryOptions: GameSelectOption[] = [
  { value: '', label: 'All categories', icon: '🗄️' },
  { value: 'housing', label: 'Housing', icon: '🏠' },
  { value: 'food', label: 'Food', icon: '🍖' },
  { value: 'transport', label: 'Transport', icon: '🐎' },
  { value: 'utilities', label: 'Utilities', icon: '💡' },
  { value: 'entertainment', label: 'Entertainment', icon: '🎮' },
  { value: 'health', label: 'Health', icon: '🧪' },
  { value: 'education', label: 'Education', icon: '📜' },
  { value: 'shopping', label: 'Shopping', icon: '🛒' },
  { value: 'travel', label: 'Travel', icon: '🗺️' },
  { value: 'other', label: 'Other', icon: '📦' }
]

// entity panels shown in Overview (client-side filter)
const entityFilter = ref<string[]>(['banks', 'assets', 'fixed_deposits', 'investments'])
const entityOptions = [
  { key: 'banks', label: 'Banks', icon: '🏦' },
  { key: 'assets', label: 'Assets', icon: '💎' },
  { key: 'fixed_deposits', label: 'FDs', icon: '📜' },
  { key: 'investments', label: 'Invest', icon: '📈' }
] as const

function toggleEntity(key: string): void {
  const i = entityFilter.value.indexOf(key)
  if (i >= 0) entityFilter.value.splice(i, 1)
  else entityFilter.value.push(key)
}

async function fetchAnalysis(): Promise<void> {
  loading.value = true
  error.value = ''
  try {
    const query: Record<string, string> = {}
    if (startDate.value) query.start_date = startDate.value
    if (endDate.value) query.end_date = endDate.value
    if (category.value) query.categories = category.value
    data.value = await api.get<AnalysisData>('/game/analysis', query)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load analysis'
  } finally {
    loading.value = false
  }
}

watch([startDate, endDate, category], () => void fetchAnalysis())
onMounted(() => void fetchAnalysis())

const netWorthTotal = computed(() => data.value?.net_worth.total ?? 0)

const netWorthBars = computed(() => {
  const nw = data.value?.net_worth
  if (!nw) return []
  const all = [
    { key: 'banks', label: 'Banks', value: Math.max(0, nw.banks), icon: '🏦' },
    { key: 'assets', label: 'Assets', value: Math.max(0, nw.assets), icon: '💎' },
    { key: 'fixed_deposits', label: 'FDs', value: Math.max(0, nw.fixed_deposits), icon: '📜' },
    { key: 'investments', label: 'Investments', value: Math.max(0, nw.investments), icon: '📈' }
  ].filter((b) => entityFilter.value.includes(b.key))
  const max = Math.max(...all.map((b) => b.value), 1)
  return all.map((b) => ({ ...b, pct: Math.round((b.value / max) * 100) }))
})

const incomeBars = computed(() => {
  const y = data.value?.yearly_income_by_type
  if (!y) return []
  const all = [
    { label: 'Salary', value: Math.max(0, y.salary), icon: '⚔️' },
    { label: 'Business', value: Math.max(0, y.business), icon: '🏪' },
    { label: 'Investment', value: Math.max(0, y.investment), icon: '📈' },
    { label: 'Other', value: Math.max(0, y.other), icon: '🪙' }
  ]
  const max = Math.max(...all.map((b) => b.value), 1)
  return all.map((b) => ({ ...b, pct: Math.round((b.value / max) * 100) }))
})

const expenseByCategory = computed(() => {
  const list = data.value?.monthly_expenses ?? []
  const map = new Map<string, number>()
  for (const m of list) map.set(m.category, (map.get(m.category) ?? 0) + m.amount)
  const rows = [...map.entries()].map(([category, amount]) => ({ category, amount }))
  const max = Math.max(...rows.map((r) => r.amount), 1)
  return rows
    .sort((a, b) => b.amount - a.amount)
    .map((r) => ({ ...r, pct: Math.round((r.amount / max) * 100) }))
})

const savings = computed(() => data.value?.savings_rate)
const savingsPct = computed(() => Math.round(data.value?.savings_rate.savings_rate_pct ?? 0))
const roi = computed(() => data.value?.roi)

const cashFlow = computed(() => data.value?.cash_flow ?? [])
const maxFlow = computed(() => Math.max(...cashFlow.value.map((c) => Math.max(c.income, c.expense)), 1))

function fmt(n: number | undefined): string {
  return formatGold(Math.round(n ?? 0))
}
</script>

<template>
  <ion-page>
    <ion-header class="hud">
      <div class="hud-row">
        <button class="back-btn" type="button" @click="router.push('/')">‹</button>
        <h1 class="carved carved-gold hud-title">ANALYSIS</h1>
        <SyncChip />
      </div>
      <div class="filter-row">
        <button class="date-chip" type="button" @click="startPickerOpen = true">
          📅 {{ startDate || 'From' }}
        </button>
        <button class="date-chip" type="button" @click="endPickerOpen = true">
          📅 {{ endDate || 'To' }}
        </button>
        <div class="cat-select">
          <GameSelect
            v-model="category"
            :options="categoryOptions"
            placeholder="All categories"
          />
        </div>
        <button
          v-if="startDate || endDate || category"
          class="clear-chip"
          type="button"
          @click="startDate = ''; endDate = ''; category = null"
        >
          ✕ Clear
        </button>
      </div>
      <div class="tab-row">
        <button
          v-for="t in tabs"
          :key="t.key"
          class="tab-btn"
          :class="{ active: tab === t.key }"
          type="button"
          @click="tab = t.key"
        >
          <span class="tab-avatar">{{ t.avatar }}</span>
          <span class="tab-label carved">{{ t.label }}</span>
        </button>
      </div>
    </ion-header>

    <ion-content :fullscreen="true" class="an-content">
      <div class="an-scroll ff-hide-scrollbar">
        <div v-if="loading" class="an-state carved carved-sm">Consulting the sages…</div>
        <div v-else-if="error" class="an-state carved carved-sm an-error">{{ error }}</div>
        <div v-else-if="!data" class="an-state carved carved-sm">No data yet, commander.</div>

        <template v-else>
          <!-- OVERVIEW -->
          <template v-if="tab === 'overview'">
            <div class="panel-stone hero-panel">
              <div class="hero-avatar">🧙</div>
              <div class="hero-mid">
                <span class="carved carved-sm hero-label">NET WORTH</span>
                <span class="carved carved-gold hero-value">🪙 {{ fmt(netWorthTotal) }}</span>
              </div>
              <div class="hero-side">
                <span class="carved carved-sm">SAVINGS RATE</span>
                <span class="carved hero-pct" :class="savingsPct >= 20 ? 'good' : savingsPct >= 0 ? 'mid' : 'bad'">
                  {{ savingsPct }}%
                </span>
              </div>
            </div>

            <div class="panel-stone">
              <div class="panel-head">
                <span class="carved panel-title">Vault Breakdown</span>
                <div class="entity-chips">
                  <button
                    v-for="e in entityOptions"
                    :key="e.key"
                    class="entity-chip"
                    :class="{ on: entityFilter.includes(e.key) }"
                    type="button"
                    @click="toggleEntity(e.key)"
                  >
                    {{ e.icon }}
                  </button>
                </div>
              </div>
              <div v-for="b in netWorthBars" :key="b.key" class="bar-row">
                <span class="bar-icon">{{ b.icon }}</span>
                <span class="bar-label carved carved-sm">{{ b.label }}</span>
                <div class="bar-track">
                  <div class="bar-fill gold" :style="{ width: b.pct + '%' }" />
                </div>
                <span class="bar-value carved">🪙 {{ fmt(b.value) }}</span>
              </div>
              <div v-if="!netWorthBars.length" class="empty carved carved-sm">No vaults selected</div>
            </div>

            <div class="two-col">
              <div class="panel-stone">
                <span class="carved panel-title">ROI</span>
                <div class="stat-grid">
                  <div class="stat">
                    <span class="carved carved-sm">Invested</span>
                    <span class="carved stat-val">🪙 {{ fmt(roi?.total_invested) }}</span>
                  </div>
                  <div class="stat">
                    <span class="carved carved-sm">Current</span>
                    <span class="carved stat-val">🪙 {{ fmt(roi?.total_current_value) }}</span>
                  </div>
                  <div class="stat">
                    <span class="carved carved-sm">Gain/Loss</span>
                    <span class="carved stat-val" :class="(roi?.total_gain_loss ?? 0) >= 0 ? 'good' : 'bad'">
                      {{ (roi?.total_gain_loss ?? 0) >= 0 ? '+' : '' }}{{ fmt(roi?.total_gain_loss) }}
                    </span>
                  </div>
                  <div class="stat">
                    <span class="carved carved-sm">ROI %</span>
                    <span class="carved stat-val" :class="(roi?.roi_percentage ?? 0) >= 0 ? 'good' : 'bad'">
                      {{ (roi?.roi_percentage ?? 0).toFixed(1) }}%
                    </span>
                  </div>
                </div>
              </div>
              <div class="panel-stone">
                <span class="carved panel-title">Savings Chest</span>
                <div class="stat-grid">
                  <div class="stat">
                    <span class="carved carved-sm">Income</span>
                    <span class="carved stat-val">🪙 {{ fmt(savings?.total_income) }}</span>
                  </div>
                  <div class="stat">
                    <span class="carved carved-sm">Expense</span>
                    <span class="carved stat-val">🪙 {{ fmt(savings?.total_expense) }}</span>
                  </div>
                  <div class="stat">
                    <span class="carved carved-sm">Saved</span>
                    <span class="carved stat-val good">🪙 {{ fmt(savings?.savings) }}</span>
                  </div>
                  <div class="stat">
                    <span class="carved carved-sm">Rate</span>
                    <span class="carved stat-val" :class="savingsPct >= 20 ? 'good' : 'mid'">{{ savingsPct }}%</span>
                  </div>
                </div>
              </div>
            </div>
          </template>

          <!-- INCOME -->
          <template v-else-if="tab === 'income'">
            <div class="panel-stone hero-panel">
              <div class="hero-avatar">💰</div>
              <div class="hero-mid">
                <span class="carved carved-sm hero-label">YEARLY INCOME</span>
                <span class="carved carved-gold hero-value">🪙 {{ fmt(data.yearly_income_by_type.total) }}</span>
              </div>
            </div>
            <div class="panel-stone">
              <span class="carved panel-title">By Source Type</span>
              <div v-for="b in incomeBars" :key="b.label" class="bar-row">
                <span class="bar-icon">{{ b.icon }}</span>
                <span class="bar-label carved carved-sm">{{ b.label }}</span>
                <div class="bar-track">
                  <div class="bar-fill green" :style="{ width: b.pct + '%' }" />
                </div>
                <span class="bar-value carved">🪙 {{ fmt(b.value) }}</span>
              </div>
            </div>
            <div class="panel-stone">
              <span class="carved panel-title">Top 5 Income Sources</span>
              <div v-for="(it, i) in data.top_5_income_sources" :key="i" class="list-row">
                <span class="carved list-rank">#{{ i + 1 }}</span>
                <span class="carved list-label">{{ it.label }}</span>
                <span class="carved list-amount good">+{{ fmt(it.amount) }}</span>
              </div>
              <div v-if="!data.top_5_income_sources.length" class="empty carved carved-sm">
                No loot yet — raid for income!
              </div>
            </div>
          </template>

          <!-- EXPENSES -->
          <template v-else-if="tab === 'expenses'">
            <div class="panel-stone hero-panel">
              <div class="hero-avatar">🛡️</div>
              <div class="hero-mid">
                <span class="carved carved-sm hero-label">TOTAL EXPENSES</span>
                <span class="carved hero-value bad">🪙 {{ fmt(data.savings_rate.total_expense) }}</span>
              </div>
            </div>
            <div class="panel-stone">
              <span class="carved panel-title">By Category</span>
              <div v-for="b in expenseByCategory" :key="b.category" class="bar-row">
                <span class="bar-icon">⚔️</span>
                <span class="bar-label carved carved-sm capitalize">{{ b.category }}</span>
                <div class="bar-track">
                  <div class="bar-fill red" :style="{ width: b.pct + '%' }" />
                </div>
                <span class="bar-value carved">🪙 {{ fmt(b.amount) }}</span>
              </div>
              <div v-if="!expenseByCategory.length" class="empty carved carved-sm">Peaceful times — no expenses.</div>
            </div>
            <div class="panel-stone">
              <span class="carved panel-title">Top 5 Expense Strikes</span>
              <div v-for="(it, i) in data.top_5_expenses" :key="i" class="list-row">
                <span class="carved list-rank">#{{ i + 1 }}</span>
                <span class="carved list-label">{{ it.label }}</span>
                <span class="carved list-amount bad">-{{ fmt(it.amount) }}</span>
              </div>
              <div v-if="!data.top_5_expenses.length" class="empty carved carved-sm">Nothing spent. Suspicious…</div>
            </div>
          </template>

          <!-- INVESTMENTS -->
          <template v-else-if="tab === 'investments'">
            <div class="panel-stone hero-panel">
              <div class="hero-avatar">🏰</div>
              <div class="hero-mid">
                <span class="carved carved-sm hero-label">PORTFOLIO VALUE</span>
                <span class="carved carved-gold hero-value">🪙 {{ fmt(roi?.total_current_value) }}</span>
              </div>
              <div class="hero-side">
                <span class="carved carved-sm">GAIN/LOSS</span>
                <span class="carved hero-pct" :class="(roi?.total_gain_loss ?? 0) >= 0 ? 'good' : 'bad'">
                  {{ (roi?.total_gain_loss ?? 0) >= 0 ? '+' : '' }}{{ fmt(roi?.total_gain_loss) }}
                </span>
              </div>
            </div>

            <div class="panel-stone" v-if="data.investment_performance.length">
              <span class="carved panel-title">Investment Performance</span>
              <div v-for="(it, i) in data.investment_performance" :key="i" class="list-row">
                <span class="bar-icon">📈</span>
                <span class="carved list-label">{{ it.name ?? it.type ?? 'Investment' }}</span>
                <span class="carved list-amount" :class="(it.gain_loss ?? 0) >= 0 ? 'good' : 'bad'">
                  {{ (it.gain_loss ?? 0) >= 0 ? '+' : '' }}{{ fmt(it.gain_loss) }}
                </span>
              </div>
            </div>

            <div class="panel-stone" v-if="data.fd_maturity_timeline.length">
              <span class="carved panel-title">FD Maturity Timeline</span>
              <div v-for="fd in data.fd_maturity_timeline" :key="fd.id" class="list-row">
                <span class="bar-icon">📜</span>
                <span class="carved list-label">{{ fd.bank_name ?? 'Fixed Deposit' }}</span>
                <span class="carved list-amount">
                  🪙 {{ fmt(fd.current_value) }} · {{ fd.days_left ?? 0 }}d
                </span>
              </div>
            </div>

            <div class="panel-stone" v-if="data.credit_card_utilization.length">
              <span class="carved panel-title">Credit Card Utilization</span>
              <div v-for="cc in data.credit_card_utilization" :key="cc.card_name" class="bar-row">
                <span class="bar-icon">💳</span>
                <span class="bar-label carved carved-sm">{{ cc.card_name }}</span>
                <div class="bar-track">
                  <div
                    class="bar-fill"
                    :class="cc.utilization_pct > 70 ? 'red' : cc.utilization_pct > 40 ? 'gold' : 'green'"
                    :style="{ width: Math.min(100, Math.round(cc.utilization_pct)) + '%' }"
                  />
                </div>
                <span class="bar-value carved">{{ cc.utilization_pct.toFixed(0) }}%</span>
              </div>
            </div>

            <div
              v-if="!data.investment_performance.length && !data.fd_maturity_timeline.length && !data.credit_card_utilization.length"
              class="panel-stone"
            >
              <div class="empty carved carved-sm">No investments summoned yet.</div>
            </div>
          </template>

          <!-- CASH FLOW -->
          <template v-else>
            <div class="panel-stone hero-panel">
              <div class="hero-avatar">⚖️</div>
              <div class="hero-mid">
                <span class="carved carved-sm hero-label">NET SAVED</span>
                <span class="carved hero-value" :class="(savings?.savings ?? 0) >= 0 ? 'good' : 'bad'">
                  🪙 {{ fmt(savings?.savings) }}
                </span>
              </div>
            </div>
            <div class="panel-stone">
              <span class="carved panel-title">Monthly Flow</span>
              <div v-for="c in cashFlow" :key="c.month" class="flow-row">
                <span class="carved flow-month">{{ c.month }}</span>
                <div class="flow-bars">
                  <div class="flow-track">
                    <div class="bar-fill green" :style="{ width: (c.income / maxFlow) * 100 + '%' }" />
                  </div>
                  <div class="flow-track">
                    <div class="bar-fill red" :style="{ width: (c.expense / maxFlow) * 100 + '%' }" />
                  </div>
                </div>
                <div class="flow-nums">
                  <span class="carved carved-sm good">+{{ fmt(c.income) }}</span>
                  <span class="carved carved-sm bad">-{{ fmt(c.expense) }}</span>
                </div>
                <span class="carved flow-net" :class="c.net >= 0 ? 'good' : 'bad'">
                  {{ c.net >= 0 ? '+' : '' }}{{ fmt(c.net) }}
                </span>
              </div>
              <div v-if="!cashFlow.length" class="empty carved carved-sm">No flow recorded for this range.</div>
            </div>
          </template>
        </template>
      </div>
    </ion-content>

    <GameDatePicker
      v-if="startPickerOpen"
      :model-value="startDate"
      :open="startPickerOpen"
      title="Start Date"
      @update:model-value="startDate = $event"
      @update:open="startPickerOpen = $event"
      @confirm="startDate = $event; startPickerOpen = false"
    />
    <GameDatePicker
      v-if="endPickerOpen"
      :model-value="endDate"
      :open="endPickerOpen"
      title="End Date"
      @update:model-value="endDate = $event"
      @update:open="endPickerOpen = $event"
      @confirm="endDate = $event; endPickerOpen = false"
    />
  </ion-page>
</template>

<style scoped>
.hud {
  position: relative;
  z-index: 20;
  background: linear-gradient(180deg, rgba(26, 15, 0, 0.97), rgba(42, 26, 10, 0.9));
  border-bottom: 3px solid rgba(245, 197, 66, 0.35);
  padding-top: env(safe-area-inset-top);
}
.hud-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px 4px;
}
.back-btn {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  border: 3px solid #0a0500;
  background: linear-gradient(180deg, #ffe27a, var(--ff-gold) 55%, var(--ff-gold-dark));
  color: var(--ff-brown-deep);
  font-size: 26px;
  font-weight: 900;
  line-height: 1;
  cursor: pointer;
  box-shadow: 0 4px 0 #5c3c00;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.back-btn:active {
  transform: translateY(4px);
  box-shadow: none;
}
.hud-title {
  margin: 0;
  flex: 1;
  font-size: 20px;
  letter-spacing: 0.08em;
}
.filter-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 14px;
  flex-wrap: wrap;
}
.date-chip {
  border: 3px solid #120a00;
  border-radius: 10px;
  background: linear-gradient(180deg, #6b4a1f, #4a2c12);
  color: var(--ff-parchment);
  font-family: inherit;
  font-size: 12px;
  font-weight: 700;
  padding: 7px 12px;
  cursor: pointer;
  box-shadow: 0 3px 0 rgba(0, 0, 0, 0.5);
}
.date-chip:active {
  transform: translateY(3px);
  box-shadow: none;
}
.cat-select {
  min-width: 170px;
}
.clear-chip {
  border: 3px solid #120a00;
  border-radius: 10px;
  background: linear-gradient(180deg, #b3402e, #7a2418);
  color: #ffe2dc;
  font-family: inherit;
  font-size: 12px;
  font-weight: 800;
  padding: 7px 10px;
  cursor: pointer;
  box-shadow: 0 3px 0 rgba(0, 0, 0, 0.5);
}
.tab-row {
  display: flex;
  gap: 6px;
  padding: 6px 14px 8px;
  overflow-x: auto;
  scrollbar-width: none;
}
.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  border: 3px solid #120a00;
  border-radius: 12px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.08), rgba(0, 0, 0, 0.25));
  color: var(--ff-parchment);
  font-family: inherit;
  padding: 7px 12px;
  cursor: pointer;
  box-shadow: 0 3px 0 rgba(0, 0, 0, 0.45);
  white-space: nowrap;
}
.tab-btn.active {
  background: linear-gradient(180deg, #ffe27a, var(--ff-gold) 55%, var(--ff-gold-dark));
  color: var(--ff-brown-deep);
  box-shadow: 0 3px 0 #5c3c00, 0 0 14px rgba(245, 197, 66, 0.5);
}
.tab-btn:active {
  transform: translateY(3px);
  box-shadow: none;
}
.tab-avatar {
  font-size: 16px;
}
.tab-label {
  font-size: 13px;
}
.an-content {
  --background:
    radial-gradient(circle at 50% -10%, rgba(245, 197, 66, 0.2), transparent 50%),
    radial-gradient(circle at 50% 130%, #4a2c12, #120a00 75%);
}
.an-scroll {
  padding: 16px 14px calc(24px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.an-state {
  text-align: center;
  opacity: 0.7;
  padding: 30px 10px;
}
.an-error {
  color: #ff9c8a;
}
.hero-panel {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
}
.hero-avatar {
  font-size: 44px;
  width: 70px;
  height: 70px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: radial-gradient(circle at 35% 30%, rgba(255, 226, 122, 0.35), rgba(0, 0, 0, 0.3));
  border: 4px solid #120a00;
  box-shadow: 0 4px 0 rgba(0, 0, 0, 0.5);
  animation: bob 2.6s ease-in-out infinite;
}
.hero-mid {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.hero-label {
  opacity: 0.7;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}
.hero-value {
  font-size: 26px;
}
.hero-side {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}
.hero-pct {
  font-size: 22px;
}
.panel-stone {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}
.panel-title {
  font-size: 15px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--ff-gold);
}
.entity-chips {
  display: flex;
  gap: 6px;
}
.entity-chip {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  border: 3px solid #120a00;
  background: rgba(0, 0, 0, 0.35);
  font-size: 16px;
  cursor: pointer;
  opacity: 0.45;
  box-shadow: 0 3px 0 rgba(0, 0, 0, 0.5);
  padding: 0;
}
.entity-chip.on {
  opacity: 1;
  background: linear-gradient(180deg, #ffe27a, var(--ff-gold) 55%, var(--ff-gold-dark));
  box-shadow: 0 3px 0 #5c3c00, 0 0 10px rgba(245, 197, 66, 0.5);
}
.bar-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.bar-icon {
  font-size: 18px;
  width: 24px;
  text-align: center;
}
.bar-label {
  width: 96px;
  font-size: 12px;
  color: var(--ff-parchment);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.bar-track {
  flex: 1;
  height: 16px;
  border: 3px solid #120a00;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.45);
  overflow: hidden;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.6);
}
.bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.6s var(--ff-bounce);
}
.bar-fill.gold {
  background: linear-gradient(180deg, #ffe27a, var(--ff-gold-dark));
}
.bar-fill.green {
  background: linear-gradient(180deg, #8ee06b, #2f8a2f);
}
.bar-fill.red {
  background: linear-gradient(180deg, #ff8a6b, #a3301c);
}
.bar-value {
  min-width: 84px;
  text-align: right;
  font-size: 13px;
  color: var(--ff-parchment);
}
.two-col {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
@media (max-width: 700px) {
  .two-col {
    grid-template-columns: 1fr;
  }
}
.stat-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.stat {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 8px 10px;
  border: 3px solid #120a00;
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.3);
}
.stat-val {
  font-size: 16px;
  color: var(--ff-parchment);
}
.list-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border: 3px solid #120a00;
  border-radius: 10px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), transparent);
}
.list-rank {
  color: var(--ff-gold);
  font-size: 13px;
  width: 30px;
}
.list-label {
  flex: 1;
  min-width: 0;
  font-size: 14px;
  color: var(--ff-parchment);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.list-amount {
  font-size: 14px;
}
.flow-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border: 3px solid #120a00;
  border-radius: 10px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), transparent);
}
.flow-month {
  width: 74px;
  font-size: 13px;
  color: var(--ff-parchment);
}
.flow-bars {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.flow-track {
  height: 10px;
  border: 2px solid #120a00;
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.45);
  overflow: hidden;
}
.flow-nums {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
  min-width: 84px;
}
.flow-net {
  min-width: 78px;
  text-align: right;
  font-size: 14px;
}
.empty {
  text-align: center;
  opacity: 0.6;
  padding: 8px;
}
.good {
  color: #8ee06b;
}
.mid {
  color: var(--ff-gold);
}
.bad {
  color: #ff8a6b;
}
.capitalize {
  text-transform: capitalize;
}
</style>
