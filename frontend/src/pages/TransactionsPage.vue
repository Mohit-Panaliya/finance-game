<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import {
  addOutline,
  createOutline,
  pricetagOutline,
  statsChartOutline,
  trashOutline
} from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import type { EntityType, FinanceRow } from '@/types'
import { formatMoney, formatSigned, rowCurrency } from '@/utils/money'
import { formatDate, formatMonth, monthKey } from '@/utils/date'
import AppButton from '@/components/ui/AppButton.vue'
import SyncChip from '@/components/ui/SyncChip.vue'
import AppModal from '@/components/ui/AppModal.vue'

type Kind = EntityType | 'all'
type RangeKey = 'this-month' | 'last-month' | '3m' | 'ytd' | 'all'

const router = useRouter()
const finance = useFinanceStore()
const sync = useSyncStore()

const kind = ref<Kind>('all')
/** Id of the Activity row currently swiped open, so only one opens at a time. */
const openRow = ref<string | null>(null)
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
  /** The underlying list row, kept so edit/delete can address the real id. */
  row: FinanceRow
  /**
   * Bank/card this entry moved, when one is linked. It jumps straight to the
   * account's full statement so the trail is one tap away from the entry.
   */
  linked?: { label: string; to: string }
}

/** id → "Statement" link for every bank/card the user tracks. */
const statementRef = computed<Record<string, string>>(() => {
  const out: Record<string, string> = {}
  for (const b of finance.rows('banks')) out[String(b.id)] = `/accounts/banks/${String(b.id)}`
  for (const c of finance.rows('credit-cards')) out[String(c.id)] = `/accounts/credit-cards/${String(c.id)}`
  return out
})

const accountNames = computed<Record<string, string>>(() => {
  const out: Record<string, string> = {}
  for (const b of finance.rows('banks')) out[String(b.id)] = String(b.name ?? 'Bank')
  for (const c of finance.rows('credit-cards')) out[String(c.id)] = String(c.name ?? 'Card')
  return out
})

/** Text label for a linked account, e.g. "HDFC Salary". */
function linkedAccount(of: Entry): Entry['linked'] {
  const key = of.entity === 'expenses' ? String(of.row.credit_card_id || of.row.bank_id || '') : String(of.row.bank_id || '')
  const label = key ? accountNames.value[key] : ''
  const to = key ? statementRef.value[key] : ''
  return key && label && to ? { label, to } : undefined
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
    income: true,
    row: r
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
    income: false,
    row: r
  }))
  return [...income, ...expense].map((e) => ({ ...e, linked: linkedAccount(e) }))
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

/** Activity namespaces row ids as `i-` / `e-`; the REST API does not. */
function recordId(e: Entry): string {
  return e.id.replace(/^[ie]-/, '')
}

function editEntry(e: Entry) {
  router.push(`/accounts/${e.entity}/${recordId(e)}`)
}

const pendingDelete = ref<Entry | null>(null)
const deleting = ref(false)
const deleteError = ref('')

function askDelete(e: Entry) {
  deleteError.value = ''
  pendingDelete.value = e
}

function cancelDelete() {
  if (deleting.value) return
  pendingDelete.value = null
}

async function confirmDelete() {
  const entry = pendingDelete.value
  if (!entry || deleting.value) return
  deleting.value = true
  deleteError.value = ''
  try {
    await finance.remove(entry.entity, recordId(entry))
    openRow.value = null
    pendingDelete.value = null
  } catch (err) {
    deleteError.value = err instanceof Error ? err.message : String(err)
  } finally {
    deleting.value = false
  }
}

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
  // The subtitle links name the linked bank/card; their lists must be loaded too.
  void finance.fetchList('banks')
  void finance.fetchList('credit-cards')
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
            <div v-for="e in bucket.entries" :key="e.id" class="swipe-wrap">
              <div class="swipe-actions">
                <button class="swipe-btn swipe-btn-edit" type="button" @click="editEntry(e)">
                  <ion-icon :icon="createOutline" /> Edit
                </button>
                <button class="swipe-btn swipe-btn-del" type="button" @click="askDelete(e)">
                  <ion-icon :icon="trashOutline" /> Delete
                </button>
              </div>
              <div
                class="swipe-content"
                :class="{ 'swipe-content-open': openRow === e.id }"
                @click="openRow = openRow === e.id ? null : e.id"
              >
                <span class="row-icon" :class="e.income ? 'text-success' : 'text-danger'">
                  <ion-icon :icon="e.income ? statsChartOutline : pricetagOutline" />
                </span>
                <span class="row-main">
                  <span class="row-title clamp-1">{{ e.title }}</span>
                  <span class="row-sub clamp-1">
                  <template v-if="e.subtitle || e.linked">
                    {{ e.subtitle }}<template v-if="e.subtitle && e.linked"> · </template>
                    <router-link v-if="e.linked" class="sub-link" :to="e.linked.to" @click.stop>
                      {{ e.linked.label }}
                    </router-link>
                  </template>
                  <template v-else>—</template>
                </span>
                </span>
                <span class="row-value">
                  <span :class="e.income ? 'text-success' : 'text-danger'">
                    {{ e.income ? '+' : '−' }}{{ formatMoney(e.amount, e.currency, { compact: true }) }}
                  </span>
                  <span class="row-extra">{{ formatDate(e.date) }}</span>
                </span>
              </div>
            </div>
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

    <AppModal
      :open="pendingDelete !== null"
      title="Delete entry"
      :subtitle="pendingDelete ? pendingDelete.title : ''"
      :sheet="false"
      @close="cancelDelete"
    >
      <p class="confirm-text">
        This removes the entry permanently. If a bank account or card is linked, its balance is
        adjusted back by the amount. If you are offline the change is queued and pushed on reconnect.
      </p>
      <p v-if="deleteError" class="form-error">{{ deleteError }}</p>
      <template #footer>
        <AppButton variant="neutral" size="md" :disabled="deleting" @click="cancelDelete">
          Cancel
        </AppButton>
        <AppButton variant="danger" size="md" :disabled="deleting" @click="confirmDelete">
          {{ deleting ? 'Deleting…' : 'Delete' }}
        </AppButton>
      </template>
    </AppModal>
  </ion-page>
</template>

<style scoped>
.tally {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--density-gap);
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
.swipe-wrap {
  position: relative;
  overflow: hidden;
  border-bottom: 1px solid var(--border);
}
.swipe-wrap:last-child {
  border-bottom: none;
}
.swipe-actions {
  position: absolute;
  inset: 0 0 0 auto;
  display: flex;
}
.swipe-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  width: 74px;
  border: none;
  font-size: 0.7rem;
  font-weight: 600;
  cursor: pointer;
}
.swipe-btn ion-icon {
  font-size: 19px;
}
.swipe-btn-edit {
  background: var(--accent);
  color: var(--on-accent);
}
.swipe-btn-del {
  background: var(--danger);
  color: var(--on-accent);
}
.swipe-content {
  position: relative;
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  width: 100%;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  background: var(--surface);
  transition: transform 0.22s ease;
  cursor: pointer;
}
.swipe-content-open {
  transform: translateX(-148px);
}
.confirm-text {
  margin: 0;
  font-size: 0.9rem;
  color: var(--text-muted);
  line-height: 1.5;
}
.fab-row {
  display: flex;
  gap: var(--density-gap);
}
.fab-row > * {
  flex: 1;
}
.sub-link {
  color: var(--accent);
  text-decoration: underline;
  text-underline-offset: 2px;
}
</style>