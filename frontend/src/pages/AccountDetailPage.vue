<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import {
  addOutline,
  arrowBackOutline,
  chevronDownOutline,
  chevronForwardOutline,
  chevronUpOutline,
  createOutline,
  searchOutline,
  trashOutline,
  walletOutline
} from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import {
  ENTITY_SNAKE,
  detailFields,
  entityConfig,
  isEntityType,
  rowSubtitle,
  rowTitle,
  rowValue
} from '@/entityConfig'
import type { FieldDef } from '@/entityConfig'
import type { EntityType, FinanceRow, StatementEntry } from '@/types'
import { api, ApiError } from '@/api/client'
import { formatMoney, formatSigned, rowCurrency } from '@/utils/money'
import { formatDate, formatDateTime, todayISO } from '@/utils/date'
import { uuid } from '@/services/offlineQueue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDatePicker from '@/components/ui/AppDatePicker.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppModal from '@/components/ui/AppModal.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import SyncChip from '@/components/ui/SyncChip.vue'

const route = useRoute()
const router = useRouter()
const finance = useFinanceStore()
const sync = useSyncStore()

/** The column that carries "when" for each entity — used for list subtitles and the sheet header. */
const DATE_COLUMN: Record<EntityType, string> = {
  banks: 'created_at',
  assets: 'purchase_date',
  'fixed-deposits': 'start_date',
  investments: 'purchase_date',
  'credit-cards': 'created_at',
  incomes: 'income_date',
  expenses: 'expense_date'
}

const rawId = route.params.id as string | undefined

const entity = computed<EntityType>(() => {
  const raw = route.params.entity
  const value = Array.isArray(raw) ? raw[0] : raw
  return isEntityType(value) ? value : 'banks'
})

const isNewRoute = computed(() => route.name === 'account-new')
const cfg = computed(() => entityConfig(entity.value))
const list = computed(() => finance.lists[entity.value])
const fields = computed<FieldDef[]>(() => detailFields(cfg.value))

const search = ref('')
const sortKey = ref('')
const sortDir = ref<'asc' | 'desc'>('desc')
const swipeOpen = ref<string | null>(null)

const detail = ref<FinanceRow | null>(null)
const formOpen = ref(false)
const editingId = ref<string | null>(null)
// /accounts/:entity/:id opens straight into the edit form. Entries arriving from the
// Activity page are namespaced as `i-<id>` / `e-<id>`.
const routeEditId = (() => {
  if (!rawId) return null
  const prefix = rawId.slice(0, 2)
  return prefix === 'i-' || prefix === 'e-' ? rawId.slice(2) : rawId
})()

if (routeEditId) {
  editingId.value = routeEditId
  formOpen.value = true
}

// The form opens immediately, but its draft cannot be filled until the list for this
// entity has loaded, so seed it from a watcher. Setting editingId alone would show an
// empty form, and saving that would blank the record.
let routeDraftPending = routeEditId !== null
watch(
  () => list.value.items,
  (rows) => {
    if (!routeDraftPending || routeEditId === null) return
    const row = rows.find((r) => rowId(r) === routeEditId)
    if (!row) return
    loadDraft(row)
    routeDraftPending = false
  },
  { immediate: true },
)
const pendingDelete = ref<FinanceRow | null>(null)
const saving = ref(false)
const formError = ref('')
const draft = ref<Record<string, string | number | null>>({})

const dateColumn = computed(() => DATE_COLUMN[entity.value])

/* ---------------- draft helpers ---------------- */

function seedDraft() {
  const next: Record<string, string | number | null> = {}
  for (const f of fields.value) {
    if (f.kind === 'date') next[f.key] = todayISO()
    else if (f.kind === 'bool') next[f.key] = f.key === 'is_active' || f.key === 'is_liquid' ? 'true' : 'false'
    else if (f.kind === 'select') next[f.key] = f.options?.[0]?.value ?? ''
    else next[f.key] = ''
  }
  draft.value = next
}

function loadDraft(row: FinanceRow) {
  const next: Record<string, string | number | null> = {}
  for (const f of fields.value) {
    const raw = row[f.key]
    if (f.kind === 'bool') next[f.key] = raw === true || raw === 'true' ? 'true' : 'false'
    else if (f.kind === 'money' || f.kind === 'number')
      next[f.key] = raw === undefined || raw === null ? '' : Number(raw)
    else next[f.key] = raw === undefined || raw === null ? '' : String(raw)
  }
  draft.value = next
}

function setDraft(key: string, value: string | number | null) {
  draft.value = { ...draft.value, [key]: value }
}

/** Human name of a bank or card a row points at, or null when it no longer exists. */
function labelFor(entity: 'banks' | 'credit-cards', id: string): string | null {
  const hit = finance.rows(entity).find((r) => String(r.id) === id)
  const name = hit ? String(hit.name ?? '').trim() : ''
  return name || null
}

const accountOptions = computed(() => [
  { value: '', label: 'Not linked' },
  ...finance.rows('banks').map((b) => ({ value: String(b.id), label: String(b.name ?? '') }))
])

const cardOptions = computed(() => [
  { value: '', label: 'Not linked' },
  ...finance.rows('credit-cards').map((c) => ({ value: String(c.id), label: String(c.name ?? '') }))
])

/** Static lists come from the config; `dynamic` selects are filled from live rows. */
function optionsFor(field: FieldDef) {
  if (field.dynamic === 'accounts') return accountOptions.value
  if (field.dynamic === 'cards') return cardOptions.value
  return field.options ?? []
}

/** Income and expense rows can point at the account they moved through. */
const needsAccounts = computed(() => fields.value.some((f) => f.dynamic === 'accounts'))
const needsCards = computed(() => fields.value.some((f) => f.dynamic === 'cards'))

/** Fetches the link targets once, and only when the current form actually needs them. */
async function ensureLinkLists(): Promise<void> {
  if (needsAccounts.value && !finance.lists.banks.items.length) {
    await finance.fetchList('banks')
  }
  if (needsCards.value && !finance.lists['credit-cards'].items.length) {
    await finance.fetchList('credit-cards')
  }
}

/** The account a row was booked against, so the link is visible in the list. */
function linkedAccountLabel(row: FinanceRow): string {
  if (row.bank_id) {
    const name = labelFor('banks', String(row.bank_id))
    if (name) return name
  }
  if (row.credit_card_id) {
    const name = labelFor('credit-cards', String(row.credit_card_id))
    if (name) return name
  }
  return ''
}

function draftValue(field: FieldDef): string | number | null {
  return draft.value[field.key] ?? ''
}

/* ---------------- list ---------------- */

const sortedRows = computed<FinanceRow[]>(() => {
  const rows = [...list.value.items]
  const term = search.value.trim().toLowerCase()
  const out = term
    ? rows.filter((r) => fields.value.some((f) => String(r[f.key] ?? '').toLowerCase().includes(term)))
    : rows
  const key = sortKey.value || cfg.value.displayKeys[1]
  if (!key) return out
  const numeric = fields.value.find((f) => f.key === key)?.kind === 'money' || fields.value.find((f) => f.key === key)?.kind === 'number'
  out.sort((a, b) => {
    const av = a[key]
    const bv = b[key]
    let cmp: number
    if (numeric) cmp = (Number(av) || 0) - (Number(bv) || 0)
    else cmp = String(av ?? '').localeCompare(String(bv ?? ''))
    return sortDir.value === 'asc' ? cmp : -cmp
  })
  return out
})

const sortOptions = computed(() =>
  fields.value
    .filter((f) => ['money', 'number', 'text', 'date'].includes(f.kind))
    .map((f) => ({ value: f.key, label: f.label }))
)

const sortSelectValue = computed(() => sortKey.value)

function applySort(value: string | number) {
  sortKey.value = String(value)
  sortDir.value = 'desc'
}

function rowId(row: FinanceRow): string {
  return String(row.id ?? '')
}

function subtitleFor(row: FinanceRow): string {
  const parts: string[] = []
  const sub = rowSubtitle(cfg.value, row)
  if (sub) parts.push(sub)
  const linked = linkedAccountLabel(row)
  if (linked) parts.push(linked)
  if (!parts.length) {
    const dc = DATE_COLUMN[entity.value]
    return dc ? formatDate(row[dc]) : ''
  }
  return parts.join(' · ')
}

function toggleSwipe(row: FinanceRow) {
  const id = rowId(row)
  swipeOpen.value = swipeOpen.value === id ? null : id
}

function closeSwipe() {
  swipeOpen.value = null
}

function showDetail(row: FinanceRow) {
  closeSwipe()
  detail.value = row
}

/* ---------------- detail sheet ---------------- */

function fieldValue(row: FinanceRow, f: FieldDef): string {
  const raw = row[f.key]
  if (raw === undefined || raw === null || raw === '') return '—'
  if (f.kind === 'bool') return raw === true || raw === 'true' ? 'Yes' : 'No'
  if (f.kind === 'select') {
    const hit = f.options?.find((o) => o.value === String(raw))
    if (hit) return hit.label
    if (f.dynamic === 'accounts') return labelFor('banks', String(raw)) ?? 'Not linked'
    if (f.dynamic === 'cards') return labelFor('credit-cards', String(raw)) ?? 'Not linked'
    return String(raw).replace(/_/g, ' ')
  }
  if (f.kind === 'money' || f.kind === 'number') {
    const n = Number(raw)
    return Number.isFinite(n) ? formatMoney(n, rowCurrency(row)) : '—'
  }
  if (f.kind === 'date') return formatDate(raw)
  return String(raw)
}

/** Server-side columns that have no config entry (audit fields) are listed too. */
const detailExtra = computed(() => {
  const row = detail.value
  if (!row) return []
  const known = new Set(fields.value.map((f) => f.key))
  return Object.keys(row)
    .filter((k) => !known.has(k) && k !== 'user_id' && k !== 'id' && !k.startsWith('game_'))
    .map((k) => ({ key: k, value: row[k] }))
})

const detailDate = computed(() => {
  const row = detail.value
  if (!row) return ''
  return formatDate(row[DATE_COLUMN[entity.value]])
})

/* ---------------- statement trail (banks + credit cards) ---------------- */

/** Only accounts have a derived statement; every other entity has nothing to walk. */
const isStatementAccount = computed(() => entity.value === 'banks' || entity.value === 'credit-cards')

const statement = ref<StatementEntry[]>([])
const statementLoading = ref(false)
const statementError = ref('')
/** Guards against a slow response landing after the modal has been switched or closed. */
const statementFor = ref<string | null>(null)

const statementCurrency = computed(() => (detail.value ? rowCurrency(detail.value) : null))

watch(
  () => (detail.value ? String(detail.value.id ?? '') : ''),
  async (id) => {
    statement.value = []
    statementError.value = ''
    statementFor.value = id || null
    if (!id || !isStatementAccount.value) {
      statementLoading.value = false
      return
    }
    statementLoading.value = true
    try {
      const rows = await api.get<StatementEntry[]>(`/${entity.value}/${id}/statement`)
      if (statementFor.value !== id) return
      statement.value = Array.isArray(rows) ? rows : []
    } catch (e) {
      if (statementFor.value !== id) return
      statementError.value = e instanceof ApiError ? e.message : 'Statement unavailable'
    } finally {
      if (statementFor.value === id) statementLoading.value = false
    }
  },
  { immediate: true },
)

/** Each entry links to the record it came from (`/accounts/incomes/{id}`, …). */
function statementHref(entry: StatementEntry): string {
  const plural = `${entry.entry_type}s`
  return isEntityType(plural) ? `/accounts/${plural}/${entry.id}` : ''
}

/* ---------------- create / edit / delete ---------------- */

function startCreate() {
  seedDraft()
  editingId.value = null
  detail.value = null
  closeSwipe()
  formError.value = ''
  formOpen.value = true
}

function startEdit(row: FinanceRow) {
  loadDraft(row)
  editingId.value = rowId(row) || null
  detail.value = null
  closeSwipe()
  formError.value = ''
  formOpen.value = true
}

function coerce(field: FieldDef, raw: string | number | null): unknown {
  if (raw === '' || raw === null || raw === undefined) return null
  if (field.kind === 'money' || field.kind === 'number') {
    const n = Number(raw)
    return Number.isFinite(n) ? n : null
  }
  if (field.kind === 'bool') return String(raw) === 'true'
  if (field.kind === 'date') return String(raw)
  return typeof raw === 'string' ? raw.trim() : raw
}

async function save() {
  formError.value = ''
  const payload: Record<string, unknown> = {}
  for (const f of fields.value) {
    const value = coerce(f, draft.value[f.key] ?? null)
    if (f.required && (value === null || value === '')) {
      formError.value = `${f.label} is required`
      return
    }
    if (value !== null) payload[f.key] = value
  }

  saving.value = true
  const id = editingId.value
  try {
    if (id) {
      await finance.update(entity.value, id, payload)
    } else {
      await finance.create(entity.value, payload)
    }
    formOpen.value = false
    if (isNewRoute.value) await router.replace(`/accounts/${entity.value}`)
  } catch {
    // Offline (or the server rejected the write): queue it and show it locally.
    formError.value = 'You appear to be offline. The change is queued and will sync automatically.'
    if (id) {
      finance.patchLocal(entity.value, id, payload)
      await sync.queue(ENTITY_SNAKE[entity.value], 'update', { id, ...payload }, id)
    } else {
      const localId = uuid()
      finance.stageLocal(entity.value, { ...payload, id: localId } as FinanceRow)
      await sync.queue(ENTITY_SNAKE[entity.value], 'create', { id: localId, ...payload }, localId)
    }
    formOpen.value = false
    if (isNewRoute.value) await router.replace(`/accounts/${entity.value}`)
  } finally {
    saving.value = false
  }
}

function askDelete(row: FinanceRow) {
  detail.value = null
  closeSwipe()
  pendingDelete.value = row
}

async function doDelete() {
  const row = pendingDelete.value
  pendingDelete.value = null
  if (!row?.id) return
  const id = rowId(row)
  try {
    await finance.remove(entity.value, id)
  } catch {
    finance.dropLocal(entity.value, id)
    await sync.queue(ENTITY_SNAKE[entity.value], 'delete', { id }, id)
  }
}

watch(entity, () => {
  detail.value = null
  formOpen.value = false
  pendingDelete.value = null
  closeSwipe()
  void finance.fetchList(entity.value, search.value)
  void finance.fetchSummary(entity.value)
  void ensureLinkLists()
})

onMounted(() => {
  void finance.fetchList(entity.value, search.value)
  void finance.fetchSummary(entity.value)
  void ensureLinkLists()
  if (isNewRoute.value) startCreate()
})
</script>

<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div class="head-left">
            <button class="back-btn" type="button" aria-label="Back" @click="router.back()">
              <ion-icon :icon="arrowBackOutline" />
            </button>
            <div>
              <h1 class="page-title">{{ cfg.label }}</h1>
              <p class="page-subtitle">
                {{ sortedRows.length }} shown · {{ list.total }} total
              </p>
            </div>
          </div>
          <SyncChip :status="sync.status" :pending="sync.queueCount" @sync="finance.refresh()" />
        </header>

        <div class="toolbar">
          <div class="search">
            <ion-icon class="search-icon" :icon="searchOutline" />
            <input
              v-model="search"
              class="search-input"
              type="search"
              :placeholder="`Search ${cfg.label.toLowerCase()}`"
            />
          </div>
          <AppSelect
            class="sort-select"
            :options="[{ value: '', label: 'Default' }, ...sortOptions]"
            :model-value="sortSelectValue"
            @update:model-value="applySort"
          />
          <button
            class="sort-dir"
            type="button"
            :aria-label="`Sort ${sortDir === 'asc' ? 'ascending' : 'descending'}`"
            @click="sortDir = sortDir === 'asc' ? 'desc' : 'asc'"
          >
            <ion-icon :icon="sortDir === 'asc' ? chevronUpOutline : chevronDownOutline" />
          </button>
        </div>

        <div v-if="list.error" class="form-error">{{ list.error }}</div>

        <div v-if="list.loading && !list.items.length" class="loading-note">Loading {{ cfg.label.toLowerCase() }}…</div>

        <div v-else-if="!sortedRows.length" class="empty">
          <ion-icon class="empty-icon" :icon="walletOutline" />
          <p class="empty-title">No {{ cfg.label.toLowerCase() }} yet</p>
          <p class="empty-text">
            {{
              search
                ? 'No records match your search.'
                : `Add your first ${cfg.singular.toLowerCase()} to get started.`
            }}
          </p>
        </div>

        <div v-else class="row-list">
          <div v-for="row in sortedRows" :key="rowId(row)" class="swipe-wrap">
            <div class="swipe-actions">
              <button class="swipe-btn swipe-btn-edit" type="button" @click="startEdit(row)">
                <ion-icon :icon="createOutline" /> Edit
              </button>
              <button class="swipe-btn swipe-btn-del" type="button" @click="askDelete(row)">
                <ion-icon :icon="trashOutline" /> Delete
              </button>
            </div>
            <div
              class="swipe-content"
              :class="{ 'swipe-content-open': swipeOpen === rowId(row) }"
              @click="toggleSwipe(row)"
            >
              <span class="row-icon"><ion-icon :icon="cfg.icon" /></span>
              <span class="row-main">
                <span class="row-title clamp-1">{{ rowTitle(cfg, row) }}</span>
                <span class="row-sub clamp-1">{{ subtitleFor(row) }}</span>
              </span>
              <span class="row-value">{{ formatMoney(rowValue(cfg, row), rowCurrency(row), { compact: true }) }}</span>
              <button class="row-open" type="button" aria-label="Open details" @click.stop="showDetail(row)">
                <ion-icon :icon="chevronForwardOutline" />
              </button>
            </div>
          </div>
        </div>

        <AppButton variant="primary" size="md" block @click="startCreate">
          <ion-icon :icon="addOutline" /> New {{ cfg.singular.toLowerCase() }}
        </AppButton>
      </div>
    </ion-content>

    <!-- Per-record detail: every configured field plus remaining server columns -->
    <AppModal
      :open="detail !== null"
      :title="detail ? rowTitle(cfg, detail) : ''"
      :subtitle="detail ? `${cfg.singular} · ${detailDate}` : ''"
      @close="detail = null"
    >
      <template v-if="detail">
        <!-- Derived statement: every row that moved this account, newest first -->
        <section v-if="isStatementAccount" class="stmt">
          <p class="card-label">Statement</p>
          <p v-if="statementLoading" class="text-sm text-muted">Loading statement…</p>
          <p v-else-if="statementError" class="form-error">{{ statementError }}</p>
          <p v-else-if="!statement.length" class="text-sm text-muted">
            No movements recorded for this account yet.
          </p>
          <div v-else class="row-list stmt-list">
            <router-link
              v-for="e in statement"
              :key="`${e.entry_type}-${e.id}`"
              class="row-item stmt-row"
              :to="statementHref(e)"
              @click="detail = null"
            >
              <span class="row-main">
                <span class="row-title clamp-1">{{ e.title || e.entry_type }}</span>
                <span class="row-sub clamp-1">{{ formatDate(e.occurred_on) }} · {{ e.entry_type }}</span>
              </span>
              <span class="row-value">
                <span :class="e.signed_amount < 0 ? 'text-danger' : 'text-success'">
                  {{ formatSigned(e.signed_amount, statementCurrency) }}
                </span>
                <span class="row-extra">{{ formatMoney(e.balance_after, statementCurrency) }}</span>
              </span>
            </router-link>
          </div>
        </section>

        <div class="meta-list">
          <div v-for="f in fields" :key="f.key" class="meta-row">
            <span class="meta-key">{{ f.label }}</span>
            <span class="meta-val">{{ fieldValue(detail, f) }}</span>
          </div>
          <div v-for="extra in detailExtra" :key="extra.key" class="meta-row">
            <span class="meta-key">{{ extra.key.replace(/_/g, ' ') }}</span>
            <span class="meta-val">
              {{ extra.key.endsWith('_at') ? formatDateTime(extra.value) : extra.key === 'tags' && Array.isArray(extra.value) ? extra.value.join(', ') : extra.value }}
            </span>
          </div>
        </div>
      </template>
      <template #footer>
        <AppButton variant="neutral" size="md" @click="detail && startEdit(detail)">Edit</AppButton>
        <AppButton variant="danger" size="md" @click="detail && askDelete(detail)">Delete</AppButton>
      </template>
    </AppModal>

    <!-- Create / edit form -->
    <AppModal
      :open="formOpen"
      :title="editingId ? `Edit ${cfg.singular.toLowerCase()}` : `New ${cfg.singular.toLowerCase()}`"
      :subtitle="cfg.label"
      @close="formOpen = false"
    >
      <div class="form-grid">
        <p v-if="formError" class="form-error">{{ formError }}</p>

        <AppDatePicker
          v-for="f in fields.filter((x) => x.kind === 'date')"
          :key="f.key"
          :model-value="draftValue(f) === '' ? null : String(draftValue(f))"
          :label="f.label"
          @update:model-value="(v) => setDraft(f.key, v)"
        />

        <AppSelect
          v-for="f in fields.filter((x) => x.kind === 'select' || x.kind === 'bool')"
          :key="f.key"
          :options="optionsFor(f)"
          :model-value="draftValue(f)"
          :label="f.label"
          @update:model-value="(v) => setDraft(f.key, v as string)"
        />

        <AppInput
          v-for="f in fields.filter((x) => x.kind !== 'date' && x.kind !== 'select' && x.kind !== 'bool')"
          :key="f.key"
          :model-value="draftValue(f) as string | number"
          :type="f.kind === 'money' || f.kind === 'number' ? 'number' : 'text'"
          :label="f.label"
          :suffix="f.suffix"
          :hint="f.hint"
          :placeholder="f.placeholder"
          @update:model-value="(v) => setDraft(f.key, v)"
        />
      </div>
      <template #footer>
        <AppButton variant="neutral" size="md" @click="formOpen = false">Cancel</AppButton>
        <AppButton variant="primary" size="md" :disabled="saving" @click="save">
          {{ saving ? 'Saving…' : 'Save' }}
        </AppButton>
      </template>
    </AppModal>

    <!-- Delete confirmation -->
    <AppModal
      :open="pendingDelete !== null"
      title="Delete record"
      :subtitle="pendingDelete ? rowTitle(cfg, pendingDelete) : ''"
      :sheet="false"
      @close="pendingDelete = null"
    >
      <p class="confirm-text">
        This removes the record permanently. If you are offline the change is queued and pushed on reconnect.
      </p>
      <template #footer>
        <AppButton variant="neutral" size="md" @click="pendingDelete = null">Keep</AppButton>
        <AppButton variant="danger" size="md" @click="doDelete">Delete</AppButton>
      </template>
    </AppModal>
  </ion-page>
</template>

<style scoped>
.head-left {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  min-width: 0;
}
.back-btn {
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 50%;
  color: var(--text);
  font-size: 18px;
  cursor: pointer;
}
.toolbar {
  display: flex;
  gap: var(--density-gap);
  align-items: center;
}
.search {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  min-height: 42px;
  padding: 0 var(--density-row-pad-x);
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}
.search-icon {
  font-size: 16px;
  color: var(--text-faint);
  flex-shrink: 0;
}
.search-input {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text);
  font-family: var(--font-body);
  font-size: 0.95rem;
}
.search-input::placeholder {
  color: var(--text-faint);
}
.sort-select {
  width: 128px;
  flex-shrink: 0;
}
.sort-dir {
  width: 42px;
  height: 42px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  font-size: 17px;
  cursor: pointer;
}
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
.row-open {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-faint);
  font-size: 16px;
  cursor: pointer;
  padding: 0;
}

/* swipe-to-reveal actions */
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
  font-family: var(--font-body);
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

/* statement trail (banks + cards) */
.stmt {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
}
.stmt-row .row-title,
.stmt-row .row-value {
  font-variant-numeric: tabular-nums;
}
</style>
