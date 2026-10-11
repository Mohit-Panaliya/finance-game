<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div class="page-head-text">
            <h1 class="page-title">Buy List</h1>
            <p class="page-subtitle">Track items you plan to purchase</p>
          </div>
          <HeaderActions
            :actions="headerActions"
          />
        </header>

        <section class="section">
          <div class="chips ff-hide-scrollbar">
            <button
              v-for="opt in FILTERS"
              :key="opt.key"
              class="chip"
              type="button"
              :class="{ 'chip-active': filter === opt.key }"
              @click="setFilter(opt.key)"
            >
              {{ opt.label }}
            </button>
          </div>
        </section>

        <section class="section">
          <div v-if="items.length" class="row-list">
            <div v-for="item in items" :key="item.id" class="row-item">
              <div class="row-main">
                <div class="row-title-row">
                  <span class="row-title clamp-1">{{ item.title }}</span>
                  <span v-if="statusBadge(item)" class="badge" :class="statusBadge(item)?.cls">
                    {{ statusBadge(item)?.label }}
                  </span>
                </div>
                <div class="row-sub clamp-1">
                  {{ item.asset_type }} · {{ item.shop || 'No shop' }} · {{ item.priority }} priority
                </div>
                <div v-if="item.converted_at" class="row-sub clamp-1">
                  Converted {{ item.converted_at }}
                  <template v-if="item.converted_asset_id">
                    ·
                    <a :href="assetLink(item.converted_asset_id)">View asset</a>
                  </template>
                </div>
              </div>
              <div class="row-value">
                <template v-if="item.status === 'purchased' && item.actual_cost != null">
                  {{ formatMoney(item.actual_cost) }}
                  <span class="row-extra">est {{ formatMoney(item.estimated_cost) }}</span>
                </template>
                <template v-else>
                  {{ formatMoney(item.estimated_cost) }}
                </template>
                <span v-if="depr[item.id]" class="row-extra">
                  Book {{ formatMoney(depr[item.id].book_value_now) }}
                </span>
              </div>
              <div class="row-actions">
                <app-button v-if="item.status !== 'purchased'" size="sm" @click="openConvert(item)">
                  Convert
                </app-button>
                <app-button size="sm" variant="ghost" @click="openEdit(item)">Edit</app-button>
                <app-button
                  v-if="item.status === 'purchased'"
                  size="sm"
                  variant="ghost"
                  @click="buyAgain(item)"
                  >Buy again</app-button
                >
              </div>
            </div>
          </div>

          <div v-else-if="allItems.length" class="empty">
            <ion-icon class="empty-icon" :icon="searchOutline" />
            <p class="empty-title">Nothing in this view</p>
            <p class="empty-text">No items match the “{{ activeFilterLabel }}” filter.</p>
            <app-button variant="neutral" size="sm" @click="setFilter('all')">Show all items</app-button>
          </div>

          <div v-else class="empty">
            <ion-icon class="empty-icon" :icon="cartOutline" />
            <p class="empty-title">Nothing on the buy list</p>
            <p class="empty-text">
              Add what you plan to buy — with an estimate and a shop — then convert it to an asset once
              you have paid.
            </p>
            <app-button variant="primary" size="sm" @click="openCreate">New item</app-button>
          </div>
        </section>

        <app-button variant="primary" size="md" block @click="openCreate">
          <ion-icon :icon="addOutline" /> New item
        </app-button>
      </div>

      <app-modal
        :open="editorOpen"
        :title="editingId ? 'Edit buy item' : 'New buy item'"
        :sheet="true"
        @close="closeEditor"
      >
        <div class="form-grid">
          <app-input v-model="form.title" label="Title" placeholder="Title" />
          <app-input v-model="form.description" label="Description" placeholder="Description" />
          <app-input v-model="form.asset_type" label="Asset type" placeholder="Asset type" />
          <app-input v-model.number="form.estimated_cost" type="number" label="Estimated cost" placeholder="Estimated cost" />
          <app-select v-model="form.priority" label="Priority" :options="priorityOptions" />
          <app-input v-model="form.target_date" label="Target date (YYYY-MM-DD)" placeholder="YYYY-MM-DD" />
          <app-input v-model="form.shop" label="Shop" placeholder="Shop" />
          <app-input v-model="form.url" label="URL" placeholder="URL" />
          <app-select v-model="form.bank_id" label="Bank" :options="bankOptions" placeholder="Select bank" />
        </div>
        <template #footer>
          <app-button v-if="editingId" variant="danger" size="md" @click="askDelete">Delete</app-button>
          <app-button variant="primary" size="md" @click="save">Save</app-button>
        </template>
      </app-modal>

      <app-modal :open="convertOpen" :title="'Convert to asset'" :sheet="true" @close="closeConvert">
        <div class="form-grid">
          <app-input v-model.number="cform.actual_cost" type="number" label="Actual cost" placeholder="Actual cost" />
          <app-input v-model="cform.purchase_date" label="Purchase date (YYYY-MM-DD)" placeholder="YYYY-MM-DD" />
          <app-select v-model="cform.bank_id" label="Bank" :options="bankOptions" placeholder="Select bank" />
          <app-input v-model="cform.asset_name" label="Asset name" placeholder="Asset name" />
          <app-input v-model="cform.category" label="Category" placeholder="Category" />
          <app-input v-model="cform.description" label="Description" placeholder="Description" />
          <app-input v-model="cform.location" label="Location" placeholder="Location" />
          <app-select v-model="cform.depreciation_method" label="Depreciation method" :options="deprOptions" />
          <app-input v-model.number="cform.useful_life_months" type="number" label="Useful life months" placeholder="Useful life months" :min="1" />
          <app-input v-model.number="cform.salvage_value" type="number" label="Salvage value" placeholder="Salvage value" />
          <div v-if="convertError" class="form-error">{{ convertError }}</div>
        </div>
        <template #footer>
          <app-button variant="neutral" size="md" @click="closeConvert">Cancel</app-button>
          <app-button variant="primary" size="md" @click="doConvert">Convert</app-button>
        </template>
      </app-modal>

      <app-modal
        :open="pendingDelete !== null"
        title="Delete item"
        :subtitle="pendingDelete ? pendingDelete.title : ''"
        :sheet="false"
        @close="pendingDelete = null"
      >
        <p class="confirm-text">
          This removes the item from your buy list. Anything already converted to an asset stays where it
          is.
        </p>
        <template #footer>
          <app-button variant="neutral" size="md" @click="pendingDelete = null">Keep</app-button>
          <app-button variant="danger" size="md" @click="doDelete">Delete</app-button>
        </template>
      </app-modal>
    </ion-content>
  </ion-page>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { IonPage, IonContent, IonIcon } from '@ionic/vue'
import { addOutline, cartOutline, searchOutline } from 'ionicons/icons'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppModal from '@/components/ui/AppModal.vue'
import HeaderActions from '@/components/ui/HeaderActions.vue'
import type { HeaderAction } from '@/components/ui/HeaderActions.vue'
import { request, ApiError } from '@/api/client'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import { formatMoney } from '@/utils/money'

const sync = useSyncStore()

/** Header action mirrors the empty-state CTA and the bottom "New item" button. */
const headerActions: HeaderAction[] = [
  { id: 'new-item', icon: addOutline, label: 'New item', action: openCreate }
]

/** Sync then reload the local list so newly added items appear immediately. */
async function onSync() {
  await sync.syncAll()
  await load()
}

const FILTERS = [
  { key: 'all', label: 'All' },
  { key: 'planned', label: 'Pending' },
  { key: 'purchased', label: 'Purchased' }
] as const

type BuyFilter = (typeof FILTERS)[number]['key']

const filter = ref<BuyFilter>('all')
const allItems = ref<any[]>([])
const banks = ref<any[]>([])
const editorOpen = ref(false)
const editingId = ref<string | null>(null)
const pendingDelete = ref<{ id: string; title: string } | null>(null)
const form = ref<any>({
  title: '',
  description: '',
  asset_type: 'Other',
  estimated_cost: 0,
  priority: 'medium',
  target_date: '',
  shop: '',
  url: '',
  bank_id: null
})
const convertOpen = ref(false)
const convertingId = ref<string | null>(null)
const cform = ref<any>({
  actual_cost: 0,
  purchase_date: '',
  bank_id: null,
  asset_name: '',
  category: '',
  description: '',
  location: '',
  depreciation_method: 'straight_line',
  useful_life_months: 60,
  salvage_value: 0
})
const convertError = ref('')
const depr = ref<Record<string, any>>({})

const priorityOptions = [
  { value: 'high', label: 'High' },
  { value: 'medium', label: 'Medium' },
  { value: 'low', label: 'Low' }
]
const deprOptions = [
  { value: 'straight_line', label: 'Straight line' },
  { value: 'declining_balance', label: 'Declining balance' },
  { value: 'double_declining', label: 'Double declining' },
  { value: 'units_of_production', label: 'Units of production' }
]
const bankOptions = computed(() => banks.value.map((b: any) => ({ value: b.id, label: b.name })))

const items = computed<any[]>(() => {
  if (filter.value === 'planned') return allItems.value.filter((x: any) => x.status === 'planned')
  if (filter.value === 'purchased') return allItems.value.filter((x: any) => x.status === 'purchased')
  return allItems.value
})

const activeFilterLabel = computed(
  () => FILTERS.find((f) => f.key === filter.value)?.label ?? 'All'
)

function setFilter(f: BuyFilter) {
  filter.value = f
}

/** Status only earns a badge when the current filter does not already say it. */
function statusBadge(item: any): { label: string; cls: string } | null {
  if (item.status === 'purchased') {
    if (filter.value === 'purchased') return null
    return { label: 'Purchased', cls: 'badge-success' }
  }
  if (item.status === 'dropped') return { label: 'Dropped', cls: '' }
  return null
}

async function load() {
  const res: any = await request('/api/buy-list', { method: 'GET' })
  // The endpoint answers `{ items, planned_count, planned_total }`; older builds
  // answered a bare array. Normalise so filtering never sees a wrapper object.
  allItems.value = Array.isArray(res) ? res : res?.items || res?.data || []
  for (const it of allItems.value) {
    if (it.converted_asset_id) {
      try {
        const d = await request(`/api/assets/${it.converted_asset_id}/depreciation`, { method: 'GET' })
        depr.value[it.id] = d
      } catch (e) {
        /* ignore */
      }
    }
  }
}

async function loadBanks() {
  const res: any = await request('/api/banks', { method: 'GET' })
  // Paged envelope { data, ... } (or a bare array from older builds).
  const rows = Array.isArray(res) ? res : res?.data ?? []
  banks.value = Array.isArray(rows) ? rows : []
}

function openCreate() {
  editingId.value = null
  form.value = {
    title: '',
    description: '',
    asset_type: 'Other',
    estimated_cost: 0,
    priority: 'medium',
    target_date: '',
    shop: '',
    url: '',
    bank_id: null
  }
  editorOpen.value = true
}

function openEdit(item: any) {
  editingId.value = item.id
  form.value = { ...item, bank_id: item.bank_id || null }
  editorOpen.value = true
}

function closeEditor() {
  editorOpen.value = false
}

async function save() {
  const payload = { ...form.value }
  if (editingId.value) {
    await request(`/api/buy-list/${editingId.value}`, { method: 'PUT', body: payload })
  } else {
    await request('/api/buy-list', { method: 'POST', body: payload })
  }
  closeEditor()
  await load()
}

function askDelete() {
  if (!editingId.value) return
  pendingDelete.value = { id: editingId.value, title: form.value.title || '(Untitled)' }
}

async function doDelete() {
  const target = pendingDelete.value
  if (!target) return
  await request(`/api/buy-list/${target.id}`, { method: 'DELETE' })
  pendingDelete.value = null
  closeEditor()
  await load()
}

async function buyAgain(item: any) {
  const payload = {
    title: item.title,
    description: item.description ?? '',
    asset_type: item.asset_type,
    estimated_cost: item.estimated_cost,
    priority: item.priority ?? 'medium',
    target_date: null as string | null,
    url: item.url ?? '',
    shop: item.shop ?? '',
    status: 'planned',
    bank_id: item.bank_id ?? null
  }
  await request('/api/buy-list', { method: 'POST', body: payload })
  await load()
}

function openConvert(item: any) {
  convertingId.value = item.id
  cform.value = {
    actual_cost: item.estimated_cost,
    purchase_date: new Date().toISOString().slice(0, 10),
    bank_id: item.bank_id || null,
    asset_name: item.title,
    category: '',
    description: item.description || '',
    location: '',
    depreciation_method: 'straight_line',
    useful_life_months: 60,
    salvage_value: 0
  }
  convertError.value = ''
  convertOpen.value = true
}

function closeConvert() {
  convertOpen.value = false
}

async function doConvert() {
  if (!convertingId.value) return
  convertError.value = ''
  const method = cform.value.depreciation_method
  const needsLife = method === 'straight_line' || method === 'declining_balance' || method === 'double_declining'
  if (needsLife && (!cform.value.useful_life_months || cform.value.useful_life_months < 1)) {
    convertError.value = 'Useful life months must be at least 1'
    return
  }
  try {
    await request(`/api/buy-list/${convertingId.value}/convert`, { method: 'POST', body: cform.value })
    const finance = useFinanceStore()
    await finance.fetchList('banks')
    await finance.fetchList('credit-cards')
    closeConvert()
    await load()
  } catch (e) {
    convertError.value = e instanceof ApiError ? e.message : String(e)
  }
}

function assetLink(id: string) {
  return `/accounts/assets/${id}`
}

onMounted(async () => {
  await loadBanks()
  await load()
})
</script>

<style scoped>
.row-title-row {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  min-width: 0;
}
.row-title-row .badge {
  flex-shrink: 0;
}

.row-actions {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  flex-shrink: 0;
}

.row-sub a {
  color: var(--accent-ink);
}

/* Money and identifiers inside the sub-lines keep the same tabular
   treatment as the `.row-value` column. */
.row-sub {
  font-variant-numeric: tabular-nums;
}

/* ---- delete confirmation ---- */
.confirm-text {
  margin: 0;
  font-size: calc(0.9rem * var(--density-font-scale));
  line-height: var(--density-leading);
  color: var(--text-muted);
}
</style>
