<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import {
  chevronForwardOutline,
  pricetagOutline,
  statsChartOutline,
  walletOutline
} from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import { useAuthStore } from '@/stores/authStore'
import { ACCOUNT_ENTITIES, entityConfig, summaryHead } from '@/entityConfig'
import type { AccountEntityName, EntityConfig } from '@/entityConfig'
import type { EntityType, FinanceRow } from '@/types'
import { formatMoney, formatPercent, rowCurrency } from '@/utils/money'
import { formatDate, monthKey } from '@/utils/date'
import SyncChip from '@/components/ui/SyncChip.vue'

const router = useRouter()
const finance = useFinanceStore()
const sync = useSyncStore()
const auth = useAuthStore()

const greeting = computed(() => {
  const h = new Date().getHours()
  if (h < 12) return 'Good morning'
  if (h < 18) return 'Good afternoon'
  return 'Good evening'
})

const displayName = computed(() => auth.user?.name || auth.user?.username || auth.user?.email || '')

const netWorth = computed(() => finance.overview?.net_worth ?? 0)
const roi = computed(() => finance.overview?.roi ?? 0)
const invested = computed(() => finance.overview?.invested ?? 0)

interface GroupCard {
  key: AccountEntityName
  cfg: EntityConfig
  total: number
  count: number
}

const groups = computed<GroupCard[]>(() =>
  ACCOUNT_ENTITIES.map((key) => {
    const cfg = entityConfig(key)
    const head = summaryHead(key, finance.summaries[key])
    return { key, cfg, total: head.total, count: head.count }
  })
)

const totalLiquid = computed(() =>
  groups.value.filter((g) => g.key !== 'credit-cards' && g.count > 0).reduce((acc, g) => acc + g.total, 0)
)

const populatedGroups = computed(() => groups.value.filter((g) => g.count > 0).length)

const totalCardBalance = computed(() =>
  groups.value.filter((g) => g.key === 'credit-cards').reduce((acc: number, g) => acc + g.total, 0)
)

const thisMonth = new Date().toISOString().slice(0, 7)

const monthIncome = computed(() =>
  finance
    .rows('incomes')
    .filter((r: FinanceRow) => monthKey(r.income_date) === thisMonth)
    .reduce((acc: number, r: FinanceRow) => acc + (Number(r.amount) || 0), 0)
)

const monthExpense = computed(() =>
  finance
    .rows('expenses')
    .filter((r: FinanceRow) => monthKey(r.expense_date) === thisMonth)
    .reduce((acc: number, r: FinanceRow) => acc + (Number(r.amount) || 0), 0)
)

const monthSaved = computed(() => monthIncome.value - monthExpense.value)

interface RecentRow {
  id: string
  entity: EntityType
  title: string
  amount: number
  currency: string
  date: string
  income: boolean
}

const recent = computed<RecentRow[]>(() => {
  const income: RecentRow[] = finance.rows('incomes').map((r: FinanceRow) => ({
    id: String(r.id ?? ''),
    entity: 'incomes' as EntityType,
    title: String(r.title ?? 'Income'),
    amount: Number(r.amount) || 0,
    currency: rowCurrency(r),
    date: String(r.income_date ?? ''),
    income: true
  }))
  const expense: RecentRow[] = finance.rows('expenses').map((r: FinanceRow) => ({
    id: String(r.id ?? ''),
    entity: 'expenses' as EntityType,
    title: String(r.title ?? 'Expense'),
    amount: Number(r.amount) || 0,
    currency: rowCurrency(r),
    date: String(r.expense_date ?? ''),
    income: false
  }))
  return [...income, ...expense]
    .sort((a, b) => b.date.localeCompare(a.date))
    .slice(0, 6)
})

const hasAnyData = computed(() =>
  ACCOUNT_ENTITIES.some((key) => finance.count(key) > 0) || finance.count('incomes') > 0
)

const loading = computed(() => finance.overviewLoading || finance.lists.banks.loading)

function money(value: number, currency?: string): string {
  return formatMoney(value, currency ?? 'INR', { compact: true })
}

onMounted(async () => {
  await Promise.allSettled([finance.fetchOverview(), finance.fetchAll()])
})
</script>

<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div>
            <p class="page-subtitle">{{ greeting }}<template v-if="displayName">, {{ displayName }}</template></p>
            <h1 class="page-title">Dashboard</h1>
          </div>
          <SyncChip :status="sync.status" :pending="sync.queueCount" @sync="sync.syncAll()" />
        </header>

        <section class="card-hero">
          <p class="card-label">Total net worth</p>
          <p class="card-value">{{ formatMoney(netWorth) }}</p>
          <div class="hero-meta">
            <span class="badge" :class="roi >= 0 ? 'badge-success' : 'badge-danger'">
              {{ formatPercent(roi) }} ROI
            </span>
            <span class="text-sm text-muted">{{ formatMoney(invested) }} invested</span>
          </div>
        </section>

        <div class="stat-grid">
          <div class="stat-tile">
            <span class="stat-tile-label">Income this month</span>
            <span class="stat-tile-value text-success">{{ money(monthIncome) }}</span>
          </div>
          <div class="stat-tile">
            <span class="stat-tile-label">Spent this month</span>
            <span class="stat-tile-value text-danger">{{ money(monthExpense) }}</span>
          </div>
          <div class="stat-tile">
            <span class="stat-tile-label">Saved this month</span>
            <span class="stat-tile-value" :class="monthSaved >= 0 ? 'text-success' : 'text-danger'">
              {{ money(monthSaved) }}
            </span>
          </div>
          <div class="stat-tile">
            <span class="stat-tile-label">Card balance</span>
            <span class="stat-tile-value">{{ money(totalCardBalance) }}</span>
          </div>
        </div>

        <div v-if="!hasAnyData && !loading" class="empty">
          <ion-icon class="empty-icon" :icon="walletOutline" />
          <p class="empty-title">Nothing tracked yet</p>
          <p class="empty-text">Add a bank account or record an income entry to see your net worth here.</p>
          <router-link class="empty-cta" to="/accounts">Open accounts</router-link>
        </div>

        <section v-else class="section">
          <div class="section-head">
            <h2 class="section-title">Accounts</h2>
            <span class="section-note">
              {{ formatMoney(totalLiquid) }} across {{ populatedGroups }} {{ populatedGroups === 1 ? 'group' : 'groups' }}
            </span>
          </div>
          <div class="row-list">
            <button
              v-for="g in groups"
              :key="g.key"
              class="row-item"
              type="button"
              @click="router.push(`/accounts/${g.key}`)"
            >
              <span class="row-icon"><ion-icon :icon="g.cfg.icon" /></span>
              <span class="row-main">
                <span class="row-title">{{ g.cfg.label }}</span>
                <span class="row-sub">
                  {{ g.count }} {{ g.count === 1 ? g.cfg.singular.toLowerCase() : 'records' }}
                </span>
              </span>
              <span class="row-value">{{ money(g.total) }}</span>
              <ion-icon class="text-faint" :icon="chevronForwardOutline" />
            </button>
          </div>
        </section>

        <section v-if="recent.length" class="section">
          <div class="section-head">
            <h2 class="section-title">Recent activity</h2>
            <button class="section-note link-btn" type="button" @click="router.push('/transactions')">
              View all
            </button>
          </div>
          <div class="row-list">
            <button
              v-for="r in recent"
              :key="`${r.entity}-${r.id}`"
              class="row-item"
              type="button"
              @click="router.push('/transactions')"
            >
              <span class="row-icon" :class="r.income ? 'text-success' : 'text-danger'">
                <ion-icon :icon="r.income ? statsChartOutline : pricetagOutline" />
              </span>
              <span class="row-main">
                <span class="row-title clamp-1">{{ r.title }}</span>
                <span class="row-sub">{{ r.income ? 'Income' : 'Expense' }} · {{ formatDate(r.date) }}</span>
              </span>
              <span class="row-value" :class="r.income ? 'text-success' : 'text-danger'">
                {{ r.income ? '+' : '−' }}{{ formatMoney(r.amount, r.currency, { compact: true }) }}
              </span>
            </button>
          </div>
        </section>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.hero-meta {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  margin-top: var(--density-gap);
  flex-wrap: wrap;
}

.empty-cta {
  margin-top: 6px;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  border-radius: var(--radius-sm);
  background: var(--accent);
  color: var(--on-accent);
  font-size: 0.85rem;
  font-weight: 600;
  text-decoration: none;
}

.link-btn {
  background: none;
  border: none;
  padding: 0;
  color: var(--accent);
  font-family: var(--font-body);
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
}
</style>