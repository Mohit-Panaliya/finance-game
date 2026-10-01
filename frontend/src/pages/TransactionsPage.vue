<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import { addOutline, pricetagOutline, statsChartOutline } from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import type { EntityType, FinanceRow } from '@/types'
import { formatMoney, formatSigned, rowCurrency } from '@/utils/money'
import { formatDate, formatMonth, monthKey } from '@/utils/date'
import AppButton from '@/components/ui/AppButton.vue'
import SyncChip from '@/components/ui/SyncChip.vue'

type Kind = EntityType | 'all'
type RangeKey = 'this-month' | 'last-month' | '3m' | 'ytd' | 'all'

const router = useRouter()
const finance = useFinanceStore()
const sync = useSyncStore()

const kind = ref<Kind>('all')
const range = ref<RangeKey>('this-month')
const category = ref('all')

interface Entry {
  id: string
  entity: 'incomes' | 'expenses'
  title: string
  subtitle: string
  amount: number
  currency: string
  date: string
  income: boolean
}

function monthShift(delta: number): string {
  const now = new Date()
  return new Date(now.getFullYear(), now.getMonth() + delta, 1).toISOString().slice(0, 7)
}

const thisMonth = new Date().toISOString().slice(0, 7)
const lastMonth = monthShift(-1)

function rangeBounds(key: RangeKey): { from: string; to: string } | null {
  const now = new Date()
  const ytd = `${now.getFullYear()}-01-01`
  const today = now.toISOString().slice(0, 10)
  switch (key) {
    case 'this-month':
      return { from: `${thisMonth}-01`, to: today }
    case 'last-month': {
      const start = `${lastMonth}-01`
      const endDate = new Date(now.getFullYear(), now.getMonth(), 0)
      return { from: start, to: endDate.toISOString().slice(0, 10) }
    }
    case '3m': {
      const from = new Date(now.getFullYear(), now.getMonth() - 2, 1)
      return { from: from.toISOString().slice(0, 10), to: today }
    }
    case 'ytd':
      return { from: ytd, to: today }
    default:
      return null
  }
}

const RANGES: Array<{ value: RangeKey; label: string }> = [
  { value: 'this-month', label: 'This month' },
  { value: 'last-month', label: 'Last month' },
  { value: '3m', label: '3 months' },
  { value: 'ytd', label: 'Year to date' },
  { value: 'all', label: 'All time' }
]

const KINDS: Array<{ value: Kind; label: string }> = [
  { value: 'all', label: 'All' },
  { value: 'incomes', label: 'Income' },
  { value: 'expenses', label: 'Expenses' }
]

const allEntries = computed<Entry[]>(() => {
  const income: Entry[] = finance.rows('incomes').map((r: FinanceRow) => ({
    id: `i-${String(r.id ?? '')}`,
    entity: 'incomes',
    title: String(r.title ?? 'Income'),
    subtitle: [String(r.source ?? ''), String(r.income_type ?? '').replace(/_/g, ' ')]
      .filter(Boolean)
      .join(' · '),
    amount: Number(r.amount) || 0,
    currency: rowCurrency(r),
    date: String(r.income_date ?? ''),
    income: true
  }))
  const expense: Entry[] = finance.rows('expenses').map((r: FinanceRow) => ({
    id: `e-${String(r.id ?? '')}`,
    entity: 'expenses',
    title: String(r.title ?? 'Expense'),
    subtitle: [String(r.category ?? '').replace(/_/g, ' '), String(r.payment_method ?? '').replace(/_/g, ' ')]
      .filter(Boolean)
      .join(' · '),
    amount: Number(r.amount) || 0,
    currency: rowCurrency(r),
    date: String(r.expense_date ?? ''),
    income: false
  }))
  return [...income, ...expense]
})

const categories = computed(() => {
  const set = new Set<string>()
  for (const e of allEntries.value) {
    const c = e.subtitle.split(' · ')[0]
    if (c) set.add(c)
  }
  return [...set].sort((a, b) => a.localeCompare(b))
})

const filtered = computed<Entry[]>(() => {
  const bounds = rangeBounds(range.value)
  return allEntries.value
    .filter((e) => (kind.value === 'all' ? true : e.entity === kind.value))
    .filter((e) => (category.value === 'all' ? true : e.subtitle.split(' · ')[0] === category.value))
    .filter((e) => (bounds ? e.date >= bounds.from && e.date <= bounds.to : true))
    .sort((a, b) => b.date.localeCompare(a.date))
})

const totalIncome = computed(() =>
  filtered.value.filter((e) => e.income).reduce((acc, e) => acc + e.amount, 0)
)
const totalExpense = computed(() =>
  filtered.value.filter((e) => !e.income).reduce((acc, e) => acc + e.amount, 0)
)
const net = computed(() => totalIncome.value - totalExpense.value)

interface Bucket {
  key: string
  label: string
  entries: Entry[]
  income: number
  expense: number
}

const grouped = computed<Bucket[]>(() => {
  const buckets = new Map<string, Entry[]>()
  for (const e of filtered.value) {
    const key = monthKey(e.date) || 'unknown'
    const list = buckets.get(key)
    if (list) list.push(e)
    else buckets.set(key, [e])
  }
  return [...buckets.entries()]
    .sort((a, b) => b[0].localeCompare(a[0]))
    .map(([key, entries]) => ({
      key,
      label: formatMonth(key),
      entries,
      income: entries.filter((e) => e.income).reduce((acc, e) => acc + e.amount, 0),
      expense: entries.filter((e) => !e.income).reduce((acc, e) => acc + e.amount, 0)
    }))
})

function addIncome() {
  router.push('/accounts/incomes/new')
}

function addExpense() {
  router.push('/accounts/expenses/new')
}

watch(
  () => [range.value, kind.value, category.value],
  () => {
    /* filtering is client-side over the cached lists — no refetch needed */
  }
)

onMounted(() => {
  void finance.fetchList('incomes')
  void finance.fetchList('expenses')
})
</script>

<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div>
            <h1 class="page-title">Activity</h1>
            <p class="page-subtitle">{{ filtered.length }} entries</p>
          </div>
          <SyncChip :status="sync.status" :pending="sync.queueCount" @sync="finance.refresh()" />
        </header>

        <section class="card">
          <div class="tally">
            <div class="tally-cell">
              <span class="card-label">Income</span>
              <span class="tally-value text-success">{{ formatMoney(totalIncome) }}</span>
            </div>
            <div class="tally-cell">
              <span class="card-label">Expenses</span>
              <span class="tally-value text-danger">{{ formatMoney(totalExpense) }}</span>
            </div>
            <div class="tally-cell">
              <span class="card-label">Net</span>
              <span class="tally-value" :class="net >= 0 ? 'text-success' : 'text-danger'">
                {{ formatSigned(net) }}
              </span>
            </div>
          </div>
        </section>

        <div class="chips ff-hide-scrollbar">
          <button
            v-for="r in RANGES"
            :key="r.value"
            class="chip"
            :class="{ 'chip-active': range === r.value }"
            type="button"
            @click="range = r.value"
          >
            {{ r.label }}
          </button>
        </div>

        <div class="chips ff-hide-scrollbar">
          <button
            v-for="k in KINDS"
            :key="k.value"
            class="chip"
            :class="{ 'chip-active': kind === k.value }"
            type="button"
            @click="kind = k.value"
          >
            {{ k.label }}
          </button>
          <button
            v-for="c in categories"
            :key="c"
            class="chip"
            :class="{ 'chip-active': category === c }"
            type="button"
            @click="category = category === c ? 'all' : c"
          >
            {{ c }}
          </button>
        </div>

        <div v-if="!filtered.length" class="empty">
          <ion-icon class="empty-icon" :icon="pricetagOutline" />
          <p class="empty-title">No activity in this range</p>
          <p class="empty-text">Widen the date range or record a new income or expense entry.</p>
        </div>

        <section v-for="bucket in grouped" :key="bucket.key" class="section">
          <div class="section-head">
            <h2 class="section-title">{{ bucket.label }}</h2>
            <span class="section-note">
              <span class="text-success">+{{ formatMoney(bucket.income, 'INR', { compact: true }) }}</span>
              ·
              <span class="text-danger">−{{ formatMoney(bucket.expense, 'INR', { compact: true }) }}</span>
            </span>
          </div>
          <div class="row-list">
            <button
              v-for="e in bucket.entries"
              :key="e.id"
              class="row-item"
              type="button"
              @click="router.push(`/accounts/${e.entity}/${e.id.replace(/^[ie]-/, '')}`)"
            >
              <span class="row-icon" :class="e.income ? 'text-success' : 'text-danger'">
                <ion-icon :icon="e.income ? statsChartOutline : pricetagOutline" />
              </span>
              <span class="row-main">
                <span class="row-title clamp-1">{{ e.title }}</span>
                <span class="row-sub clamp-1">{{ e.subtitle || '—' }}</span>
              </span>
              <span class="row-value">
                <span :class="e.income ? 'text-success' : 'text-danger'">
                  {{ e.income ? '+' : '−' }}{{ formatMoney(e.amount, e.currency, { compact: true }) }}
                </span>
                <span class="row-extra">{{ formatDate(e.date) }}</span>
              </span>
            </button>
          </div>
        </section>

        <div class="fab-row">
          <AppButton variant="success" size="md" @click="addIncome">
            <ion-icon :icon="addOutline" /> Income
          </AppButton>
          <AppButton variant="neutral" size="md" @click="addExpense">
            <ion-icon :icon="addOutline" /> Expense
          </AppButton>
        </div>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.tally {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}
.tally-cell {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.tally-value {
  font-size: 1.02rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}
.fab-row {
  display: flex;
  gap: 8px;
}
.fab-row > * {
  flex: 1;
}
</style>