<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import {
  addOutline,
  arrowDownOutline,
  arrowUpOutline,
  cashOutline,
  checkmarkCircleOutline,
  createOutline,
  peopleOutline,
  searchOutline,
  trashOutline,
  warningOutline
} from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import type { DebtRow, DebtSummary } from '@/types'
import { formatMoney, formatSigned } from '@/utils/money'
import { formatDayMonth, todayISO } from '@/utils/date'
import AppButton from '@/components/ui/AppButton.vue'
import AppDatePicker from '@/components/ui/AppDatePicker.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppModal from '@/components/ui/AppModal.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import SyncChip from '@/components/ui/SyncChip.vue'

const finance = useFinanceStore()
const sync = useSyncStore()

/* ------------------------------------------------------------------ *
 * Static option lists
 * ------------------------------------------------------------------ */

type DebtFilter = 'all' | 'lent' | 'borrowed' | 'overdue' | 'settled'

const DIRECTION_OPTIONS = [
  { value: 'lent', label: 'They owe me' },
  { value: 'borrowed', label: 'I owe them' }
]

const KIND_OPTIONS = [
  { value: 'loan', label: 'Loan' },
  { value: 'advance', label: 'Advance' },
  { value: 'shared', label: 'Shared expense' },
  { value: 'other', label: 'Other' }
]

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

const FILTERS: Array<{ key: DebtFilter; label: string }> = [
  { key: 'all', label: 'All' },
  { key: 'lent', label: 'They owe me' },
  { key: 'borrowed', label: 'I owe them' },
  { key: 'overdue', label: 'Overdue' },
  { key: 'settled', label: 'Settled' }
]

/* ------------------------------------------------------------------ *
 * Small helpers
 * ------------------------------------------------------------------ */

function num(value: unknown): number {
  const n = Number(value)
  return Number.isFinite(n) ? n : 0
}

function round2(n: number): number {
  return Math.round((n + Number.EPSILON) * 100) / 100
}

function rowKey(row: DebtRow): string {
  return String(row.id ?? '')
}

function daysLeft(row: DebtRow): number | null {
  const n = Number(row.days_to_due)
  return Number.isFinite(n) ? n : null
}

function isOverdue(row: DebtRow): boolean {
  const days = daysLeft(row)
  return !row.is_settled && days !== null && days < 0
}

function dueSoon(row: DebtRow): boolean {
  const days = daysLeft(row)
  return !row.is_settled && days !== null && days >= 0 && days <= 7
}

function kindLabel(row: DebtRow): string {
  const hit = KIND_OPTIONS.find((o) => o.value === row.kind)
  return hit ? hit.label : String(row.kind || 'Debt').replace(/_/g, ' ')
}

function settledPct(row: DebtRow): number {
  const total = num(row.amount)
  if (total <= 0) return row.is_settled ? 100 : 0
  return Math.min(100, Math.max(0, (num(row.settled_amount) / total) * 100))
}

function dueText(row: DebtRow): string {
  if (row.is_settled) return `settled ${formatDayMonth(row.settled_date)}`
  const days = daysLeft(row)
  if (days === null) return 'no due date'
  if (days < 0) {
    const n = Math.abs(Math.round(days))
    return `${n} ${n === 1 ? 'day' : 'days'} overdue`
  }
  if (days === 0) return 'due today'
  return `due ${formatDayMonth(row.due_date)}`
}

function subtitleFor(row: DebtRow): string {
  return `${kindLabel(row)} · ${dueText(row)}`
}

function extraFor(row: DebtRow): string {
  if (row.is_settled) return `Settled ${formatDayMonth(row.settled_date)}`
  if (num(row.settled_amount) > 0) return `${Math.round(settledPct(row))}% repaid`
  return `of ${formatMoney(row.amount, row.currency, { compact: true })}`
}

function iconFor(row: DebtRow) {
  return row.direction === 'borrowed' ? arrowUpOutline : arrowDownOutline
}

/* ------------------------------------------------------------------ *
 * Derived state
 * ------------------------------------------------------------------ */

const debts = computed<DebtRow[]>(() => finance.debts)
const loading = computed(() => finance.debtsLoading)
const error = computed(() => finance.debtsError)

const filter = ref<DebtFilter>('all')
const swipeOpen = ref<string | null>(null)

/** `/debts/summary` is decorative — fall back to the row list if it failed. */
const summary = computed<DebtSummary>(() => {
  const s = finance.debtSummary
  if (s) return s
  const owedToMe = debts.value
    .filter((r) => r.direction === 'lent')
    .reduce((acc, r) => acc + num(r.outstanding), 0)
  const iOwe = debts.value
    .filter((r) => r.direction === 'borrowed')
    .reduce((acc, r) => acc + num(r.outstanding), 0)
  return {
    owed_to_me: owedToMe,
    i_owe: iOwe,
    net: owedToMe - iOwe,
    lent_count: debts.value.filter((r) => r.direction === 'lent').length,
    borrowed_count: debts.value.filter((r) => r.direction === 'borrowed').length,
    settled_count: debts.value.filter((r) => r.is_settled).length,
    overdue_count: debts.value.filter(isOverdue).length,
    total_count: debts.value.length
  }
})

const owedToMe = computed(() => num(summary.value.owed_to_me))
const iOwe = computed(() => num(summary.value.i_owe))
const net = computed(() => num(summary.value.net))
const overdueCount = computed(() => num(summary.value.overdue_count))
const settledCount = computed(() => num(summary.value.settled_count))
const totalCount = computed(() => num(summary.value.total_count))

const netText = computed(() => (net.value === 0 ? formatMoney(0) : formatSigned(net.value)))
const netTone = computed(() => (net.value > 0 ? 'text-success' : net.value < 0 ? 'text-danger' : 'text-muted'))
const netNote = computed(() =>
  net.value > 0 ? 'in your favour' : net.value < 0 ? 'you owe more' : 'all square'
)

function matches(row: DebtRow, key: DebtFilter): boolean {
  switch (key) {
    case 'lent':
      return row.direction === 'lent'
    case 'borrowed':
      return row.direction === 'borrowed'
    case 'overdue':
      return isOverdue(row)
    case 'settled':
      return row.is_settled === true
    default:
      return true
  }
}

/** Unsettled and overdue first, then soonest due, then settled at the bottom. */
function rank(row: DebtRow): number {
  if (row.is_settled) return 3
  if (isOverdue(row)) return 0
  return daysLeft(row) === null ? 2 : 1
}

const filtered = computed<DebtRow[]>(() => {
  const out = debts.value.filter((r) => matches(r, filter.value))
  out.sort((a, b) => {
    const byRank = rank(a) - rank(b)
    if (byRank !== 0) return byRank
    const da = daysLeft(a) ?? Number.MAX_SAFE_INTEGER
    const db = daysLeft(b) ?? Number.MAX_SAFE_INTEGER
    if (da !== db) return da - db
    return String(a.counterparty ?? '').localeCompare(String(b.counterparty ?? ''))
  })
  return out
})

function chipCount(key: DebtFilter): number {
  return debts.value.filter((r) => matches(r, key)).length
}

const activeFilterLabel = computed(
  () => FILTERS.find((f) => f.key === filter.value)?.label ?? 'All'
)

function setFilter(key: DebtFilter) {
  filter.value = key
  swipeOpen.value = null
}

const headCounts = computed(
  () =>
    `${debts.value.length} ${debts.value.length === 1 ? 'debt' : 'debts'} · ${num(
      summary.value.lent_count
    )} owed to you · ${num(summary.value.borrowed_count)} you owe`
)

/* ------------------------------------------------------------------ *
 * Row affordances
 * ------------------------------------------------------------------ */

function toggleSwipe(row: DebtRow) {
  const id = rowKey(row)
  swipeOpen.value = swipeOpen.value === id ? null : id
}

function closeSwipe() {
  swipeOpen.value = null
}

/* ------------------------------------------------------------------ *
 * Create / edit
 * ------------------------------------------------------------------ */

const formOpen = ref(false)
const editingId = ref<string | null>(null)
const saving = ref(false)
const formError = ref('')
const draft = ref<Record<string, string | number>>({})

const bankOptions = computed(() => [
  { value: '', label: 'Not linked' },
  ...finance
    .rows('banks')
    .filter((b) => b.id !== undefined && b.id !== null)
    .map((b) => ({ value: String(b.id), label: String(b.name ?? 'Account') }))
])

function draftValue(key: string): string | number {
  return draft.value[key] ?? ''
}

function setDraft(key: string, value: string | number) {
  draft.value = { ...draft.value, [key]: value }
}

function startCreate() {
  draft.value = {
    counterparty: '',
    direction: 'lent',
    amount: '',
    kind: 'loan',
    occurred_date: todayISO(),
    due_date: '',
    currency: 'INR',
    account_id: '',
    note: ''
  }
  editingId.value = null
  closeSwipe()
  formError.value = ''
  formOpen.value = true
}

function startEdit(row: DebtRow) {
  draft.value = {
    counterparty: String(row.counterparty ?? ''),
    direction: row.direction || 'lent',
    amount: num(row.amount) || '',
    kind: row.kind || 'loan',
    occurred_date: String(row.occurred_date ?? todayISO()),
    due_date: String(row.due_date ?? ''),
    currency: row.currency || 'INR',
    account_id: String(row.account_id ?? ''),
    note: String(row.note ?? '')
  }
  editingId.value = rowKey(row) || null
  closeSwipe()
  formError.value = ''
  formOpen.value = true
}

async function save() {
  formError.value = ''

  const counterparty = String(draft.value.counterparty ?? '').trim()
  if (!counterparty) {
    formError.value = 'Counterparty is required'
    return
  }
  const direction = String(draft.value.direction ?? '')
  if (!direction) {
    formError.value = 'Direction is required'
    return
  }
  const amount = Number(draft.value.amount ?? '')
  if (draft.value.amount === '' || !Number.isFinite(amount) || amount <= 0) {
    formError.value = 'Amount is required and must be greater than zero'
    return
  }
  const occurred = String(draft.value.occurred_date ?? '')
  if (!occurred) {
    formError.value = 'Occurred date is required'
    return
  }

  const payload: Record<string, unknown> = {
    counterparty,
    direction,
    amount,
    kind: String(draft.value.kind ?? '') || 'loan',
    currency: String(draft.value.currency ?? '') || 'INR',
    occurred_date: occurred
  }
  const dueDate = String(draft.value.due_date ?? '')
  if (dueDate) payload.due_date = dueDate
  const accountId = String(draft.value.account_id ?? '')
  if (accountId) payload.account_id = accountId
  const note = String(draft.value.note ?? '').trim()
  if (note) payload.note = note

  saving.value = true
  try {
    const id = editingId.value
    if (id) await finance.updateDebt(id, payload)
    else await finance.createDebt(payload)
    formOpen.value = false
  } catch {
    formError.value = 'Could not save this debt. Check your connection and try again.'
  } finally {
    saving.value = false
  }
}

/* ------------------------------------------------------------------ *
 * Repayments
 * ------------------------------------------------------------------ */

const settleRow = ref<DebtRow | null>(null)
const settleAmount = ref<string | number>('')
const settleError = ref('')
const settleSaving = ref(false)
const settleResult = ref<{ repaid: number; remaining: number; currency: string } | null>(null)

const settleCurrency = computed(() => settleRow.value?.currency || 'INR')
const settleOutstanding = computed(() => num(settleRow.value?.outstanding))

function openSettle(row: DebtRow) {
  closeSwipe()
  settleRow.value = row
  settleAmount.value = round2(num(row.outstanding))
  settleError.value = ''
  settleResult.value = null
}

function closeSettle() {
  settleRow.value = null
  settleAmount.value = ''
  settleError.value = ''
  settleResult.value = null
}

function settleFull() {
  settleAmount.value = round2(settleOutstanding.value)
  settleError.value = ''
}

async function confirmSettle() {
  const row = settleRow.value
  if (!row?.id) return

  const outstanding = round2(num(row.outstanding))
  const value = round2(Number(settleAmount.value))
  settleError.value = ''

  if (!Number.isFinite(value) || value <= 0) {
    settleError.value = 'Enter a repayment amount greater than zero'
    return
  }
  if (value > outstanding + 0.01) {
    settleError.value = `That is more than the ${formatMoney(outstanding, row.currency)} outstanding`
    return
  }

  settleSaving.value = true
  const full = value >= outstanding - 0.01
  try {
    await finance.settleDebt(rowKey(row), full ? undefined : value)
    settleResult.value = {
      repaid: full ? outstanding : value,
      remaining: full ? 0 : round2(outstanding - value),
      currency: settleCurrency.value
    }
  } catch {
    settleError.value = 'Could not record the repayment. Check your connection and try again.'
  } finally {
    settleSaving.value = false
  }
}

/* ------------------------------------------------------------------ *
 * Delete
 * ------------------------------------------------------------------ */

const pendingDelete = ref<DebtRow | null>(null)
const deleting = ref(false)
const deleteError = ref('')

function askDelete(row: DebtRow) {
  closeSwipe()
  deleteError.value = ''
  pendingDelete.value = row
}

async function doDelete() {
  const row = pendingDelete.value
  if (!row?.id) return
  deleting.value = true
  deleteError.value = ''
  try {
    await finance.removeDebt(rowKey(row))
    pendingDelete.value = null
  } catch {
    deleteError.value = 'Could not delete this debt. Check your connection and try again.'
  } finally {
    deleting.value = false
  }
}

/* ------------------------------------------------------------------ *
 * Load
 * ------------------------------------------------------------------ */

function reload() {
  void finance.fetchDebts()
  void finance.fetchDebtSummary()
}

onMounted(() => {
  reload()
  if (!finance.rows('banks').length) void finance.fetchList('banks')
})
</script>

<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div>
            <h1 class="page-title">Debts</h1>
            <p class="page-subtitle">{{ headCounts }}</p>
          </div>
          <SyncChip :status="sync.status" :pending="sync.queueCount" @sync="reload" />
        </header>

        <div class="debt-summary">
          <div class="debt-tile">
            <span class="debt-label">Owed to me</span>
            <span class="debt-figure text-success">{{ formatMoney(owedToMe) }}</span>
            <span class="debt-note text-muted">{{ summary.lent_count }} to collect</span>
          </div>
          <div class="debt-tile">
            <span class="debt-label">I owe</span>
            <span class="debt-figure text-danger">{{ formatMoney(iOwe) }}</span>
            <span class="debt-note text-muted">{{ summary.borrowed_count }} to repay</span>
          </div>
          <div class="debt-tile debt-tile-net">
            <span class="debt-label">Net</span>
            <span class="debt-figure" :class="netTone">{{ netText }}</span>
            <span class="debt-note text-muted">{{ netNote }}</span>
          </div>
        </div>

        <div v-if="overdueCount > 0" class="debt-alert">
          <ion-icon class="debt-alert-icon" :icon="warningOutline" />
          <span>
            <strong>{{ overdueCount }}</strong>
            {{ overdueCount === 1 ? 'debt is' : 'debts are' }} past the due date. Chase them before they roll
            further.
          </span>
        </div>

        <p class="debt-quiet text-sm text-muted">
          {{ settledCount }} settled of {{ totalCount }} tracked
        </p>

        <div v-if="error" class="form-error">{{ error }}</div>

        <div v-if="loading && !debts.length" class="loading-note">Loading debts…</div>

        <template v-else>
          <div v-if="!debts.length" class="empty">
            <ion-icon class="empty-icon" :icon="peopleOutline" />
            <p class="empty-title">No debts tracked</p>
            <p class="empty-text">
              Both sides of the book live here — money you lent that you expect back, and money you owe.
              Each debt keeps its own balance, due date and repayment history.
            </p>
            <AppButton variant="primary" size="sm" @click="startCreate">
              <ion-icon :icon="addOutline" /> Add your first debt
            </AppButton>
          </div>

          <div v-else-if="!filtered.length" class="empty">
            <ion-icon class="empty-icon" :icon="searchOutline" />
            <p class="empty-title">Nothing in this view</p>
            <p class="empty-text">No debts match “{{ activeFilterLabel }}”. Switch the filter to see the rest.</p>
            <AppButton variant="neutral" size="sm" @click="setFilter('all')">Show all debts</AppButton>
          </div>

          <section v-else class="section">
            <div class="section-head">
              <h2 class="section-title">Debt book</h2>
              <span class="section-note">{{ filtered.length }} of {{ debts.length }} shown</span>
            </div>

            <div class="chips">
              <button
                v-for="opt in FILTERS"
                :key="opt.key"
                class="chip"
                :class="{ 'chip-active': filter === opt.key }"
                type="button"
                @click="setFilter(opt.key)"
              >
                {{ opt.label }}
                <span class="chip-count">{{ chipCount(opt.key) }}</span>
              </button>
            </div>

            <div class="row-list">
              <div v-for="row in filtered" :key="rowKey(row)" class="swipe-wrap">
                <div class="swipe-actions">
                  <button
                    class="swipe-btn swipe-btn-settle"
                    type="button"
                    :disabled="row.is_settled"
                    @click="openSettle(row)"
                  >
                    <ion-icon :icon="cashOutline" /> Repay
                  </button>
                  <button class="swipe-btn swipe-btn-edit" type="button" @click="startEdit(row)">
                    <ion-icon :icon="createOutline" /> Edit
                  </button>
                  <button class="swipe-btn swipe-btn-del" type="button" @click="askDelete(row)">
                    <ion-icon :icon="trashOutline" /> Delete
                  </button>
                </div>
                <div
                  class="swipe-content"
                  :class="{ 'swipe-content-open': swipeOpen === rowKey(row) }"
                  @click="toggleSwipe(row)"
                >
                  <span class="row-icon"><ion-icon :icon="iconFor(row)" /></span>
                  <span class="row-main">
                    <span class="row-title clamp-1">{{ row.counterparty }}</span>
                    <span class="debt-badges">
                      <span class="badge" :class="row.direction === 'borrowed' ? 'badge-danger' : 'badge-success'">
                        {{ row.direction === 'borrowed' ? 'I owe them' : 'They owe me' }}
                      </span>
                      <span v-if="row.is_settled" class="badge badge-success">Settled</span>
                      <span v-else-if="isOverdue(row)" class="badge badge-danger">Overdue</span>
                      <span v-else-if="dueSoon(row)" class="badge badge-warning">Due soon</span>
                    </span>
                    <span class="row-sub clamp-1">{{ subtitleFor(row) }}</span>
                    <span class="debt-bar">
                      <span class="bar">
                        <span class="bar-fill debt-bar-fill" :style="{ width: `${settledPct(row)}%` }" />
                      </span>
                    </span>
                  </span>
                  <span class="row-value" :class="{ 'text-muted': row.is_settled }">
                    {{ formatMoney(row.outstanding, row.currency, { compact: true }) }}
                    <span class="row-extra">{{ extraFor(row) }}</span>
                  </span>
                </div>
              </div>
            </div>
          </section>
        </template>

        <AppButton variant="primary" size="md" block @click="startCreate">
          <ion-icon :icon="addOutline" /> Record a debt
        </AppButton>
      </div>
    </ion-content>

    <!-- Create / edit -->
    <AppModal
      :open="formOpen"
      :title="editingId ? 'Edit debt' : 'New debt'"
      subtitle="Both directions — money lent and money borrowed"
      @close="formOpen = false"
    >
      <div class="form-grid">
        <p v-if="formError" class="form-error">{{ formError }}</p>

        <AppInput
          :model-value="draftValue('counterparty')"
          type="text"
          label="Counterparty"
          placeholder="Who is this debt with?"
          @update:model-value="(v) => setDraft('counterparty', v)"
        />

        <AppSelect
          :options="DIRECTION_OPTIONS"
          :model-value="draftValue('direction')"
          label="Direction"
          @update:model-value="(v) => setDraft('direction', v as string)"
        />

        <div class="form-row-2">
          <AppInput
            :model-value="draftValue('amount')"
            type="number"
            label="Amount"
            :suffix="String(draftValue('currency') || 'INR')"
            placeholder="0"
            @update:model-value="(v) => setDraft('amount', v)"
          />
          <AppSelect
            :options="CURRENCY_OPTIONS"
            :model-value="draftValue('currency')"
            label="Currency"
            @update:model-value="(v) => setDraft('currency', v as string)"
          />
        </div>

        <div class="form-row-2">
          <AppSelect
            :options="KIND_OPTIONS"
            :model-value="draftValue('kind')"
            label="Kind"
            @update:model-value="(v) => setDraft('kind', v as string)"
          />
          <AppSelect
            :options="bankOptions"
            :model-value="draftValue('account_id')"
            label="Account"
            @update:model-value="(v) => setDraft('account_id', v as string)"
          />
        </div>

        <div class="form-row-2">
          <AppDatePicker
            :model-value="draftValue('occurred_date') ? String(draftValue('occurred_date')) : null"
            label="Occurred on"
            @update:model-value="(v) => setDraft('occurred_date', v ?? '')"
          />
          <AppDatePicker
            :model-value="draftValue('due_date') ? String(draftValue('due_date')) : null"
            label="Due on"
            placeholder="No due date"
            @update:model-value="(v) => setDraft('due_date', v ?? '')"
          />
        </div>

        <AppInput
          :model-value="draftValue('note')"
          type="text"
          label="Note"
          placeholder="Optional — what it was for"
          @update:model-value="(v) => setDraft('note', v)"
        />
      </div>
      <template #footer>
        <AppButton variant="neutral" size="md" @click="formOpen = false">Cancel</AppButton>
        <AppButton variant="primary" size="md" :disabled="saving" @click="save">
          {{ saving ? 'Saving…' : 'Save' }}
        </AppButton>
      </template>
    </AppModal>

    <!-- Repayment -->
    <AppModal
      :open="settleRow !== null"
      title="Record repayment"
      :subtitle="settleRow ? String(settleRow.counterparty ?? '') : ''"
      :sheet="false"
      @close="closeSettle"
    >
      <div v-if="settleResult" class="settle-done">
        <ion-icon class="settle-done-icon" :icon="checkmarkCircleOutline" />
        <p class="settle-done-title">Repayment recorded</p>
        <div class="meta-list">
          <div class="meta-row">
            <span class="meta-key">Repaid</span>
            <span class="meta-val">{{ formatMoney(settleResult.repaid, settleResult.currency) }}</span>
          </div>
          <div class="meta-row">
            <span class="meta-key">Remaining</span>
            <span class="meta-val" :class="settleResult.remaining > 0 ? '' : 'text-success'">
              {{
                settleResult.remaining > 0
                  ? formatMoney(settleResult.remaining, settleResult.currency)
                  : 'Settled in full'
              }}
            </span>
          </div>
        </div>
      </div>

      <div v-else class="form-grid">
        <p v-if="settleError" class="form-error">{{ settleError }}</p>

        <div class="settle-outstanding">
          <span class="settle-outstanding-label">Outstanding</span>
          <span class="settle-outstanding-value">{{ formatMoney(settleOutstanding, settleCurrency) }}</span>
        </div>

        <AppInput
          :model-value="settleAmount"
          type="number"
          label="Repayment amount"
          :suffix="settleCurrency"
          :hint="`Up to ${formatMoney(settleOutstanding, settleCurrency)}`"
          @update:model-value="(v) => (settleAmount = v)"
        />

        <div class="settle-quick">
          <AppButton variant="ghost" size="sm" @click="settleFull">Settle in full</AppButton>
        </div>
      </div>

      <template #footer>
        <AppButton v-if="settleResult" variant="primary" size="md" block @click="closeSettle">Done</AppButton>
        <template v-else>
          <AppButton variant="neutral" size="md" @click="closeSettle">Cancel</AppButton>
          <AppButton variant="success" size="md" :disabled="settleSaving" @click="confirmSettle">
            {{ settleSaving ? 'Recording…' : 'Record' }}
          </AppButton>
        </template>
      </template>
    </AppModal>

    <!-- Delete confirmation -->
    <AppModal
      :open="pendingDelete !== null"
      title="Delete debt"
      :subtitle="pendingDelete ? String(pendingDelete.counterparty ?? '') : ''"
      :sheet="false"
      @close="pendingDelete = null"
    >
      <p v-if="deleteError" class="form-error">{{ deleteError }}</p>
      <p class="confirm-text">
        This removes the debt and its repayment history permanently. Delete it only if it was recorded by
        mistake.
      </p>
      <template #footer>
        <AppButton variant="neutral" size="md" @click="pendingDelete = null">Keep</AppButton>
        <AppButton variant="danger" size="md" :disabled="deleting" @click="doDelete">
          {{ deleting ? 'Deleting…' : 'Delete' }}
        </AppButton>
      </template>
    </AppModal>
  </ion-page>
</template>

<style scoped>
/* ---- summary strip ---- */
.debt-summary {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--density-gap);
}
.debt-tile {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: var(--density-tile-pad);
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}
.debt-tile-net {
  grid-column: 1 / -1;
}
.debt-label {
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.debt-figure {
  font-size: 1.12rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.debt-note {
  font-size: 0.74rem;
}
@media (min-width: 620px) {
  .debt-summary {
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
  }
  .debt-tile-net {
    grid-column: auto;
  }
  .debt-figure {
    font-size: 1.25rem;
  }
}

/* ---- overdue banner ---- */
.debt-alert {
  display: flex;
  align-items: flex-start;
  gap: var(--density-gap);
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  border-radius: var(--radius-md);
  border: 1px solid var(--warning-soft);
  background: var(--warning-soft);
  color: var(--warning);
  font-size: 0.82rem;
  font-weight: 550;
  line-height: 1.45;
}
.debt-alert-icon {
  flex-shrink: 0;
  font-size: 17px;
  margin-top: 1px;
}
.debt-quiet {
  margin: -4px 0 0;
}

/* ---- chips ---- */
.chip-count {
  margin-left: 6px;
  font-size: 0.7rem;
  font-variant-numeric: tabular-nums;
  opacity: 0.65;
}

/* ---- list rows ---- */
.debt-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  margin: 3px 0 1px;
}
.debt-bar {
  display: block;
  margin-top: 5px;
}
.debt-bar-fill {
  background: var(--success);
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
.swipe-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.swipe-btn-settle {
  background: var(--success);
  color: var(--on-accent);
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
  transform: translateX(-222px);
}

/* ---- notes ---- */
.loading-note {
  padding: var(--density-card-pad);
  text-align: center;
  color: var(--text-muted);
  font-size: 0.88rem;
}
.confirm-text {
  margin: 0;
  font-size: 0.9rem;
  color: var(--text-muted);
  line-height: 1.5;
}

/* ---- repayment sheet ---- */
.settle-outstanding {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--density-gap);
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}
.settle-outstanding-label {
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.settle-outstanding-value {
  font-size: 1.1rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}
.settle-quick {
  display: flex;
  justify-content: flex-start;
}
.settle-done {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--density-gap);
}
.settle-done-icon {
  font-size: 34px;
  color: var(--success);
}
.settle-done-title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}
.settle-done .meta-list {
  width: 100%;
}
</style>
