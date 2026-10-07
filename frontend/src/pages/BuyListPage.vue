<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div class="page-head-text">
            <h1 class="page-title">Buy List</h1>
            <p class="page-subtitle">Track items you plan to purchase</p>
          </div>
          <sync-chip />
        </header>

        <section class="section">
          <div class="row-list">
            <div class="row-item" style="padding: var(--density-card-pad)">
              <div class="row-main" style="display: flex; justify-content: space-between; align-items: center; gap: var(--density-gap); flex-wrap: wrap">
                <div class="chips">
                  <button class="chip" :class="{ 'chip-active': filter === 'all' }" @click="setFilter('all')">All</button>
                  <button class="chip" :class="{ 'chip-active': filter === 'planned' }" @click="setFilter('planned')">Pending</button>
                  <button class="chip" :class="{ 'chip-active': filter === 'purchased' }" @click="setFilter('purchased')">Purchased</button>
                </div>
                <app-button @click="openCreate">New item</app-button>
              </div>
            </div>
          </div>
        </section>

        <section class="section">
          <div class="row-list">
            <div v-for="item in items" :key="item.id" class="row-item">
              <div class="row-main">
                <div class="row-title">
                  {{ item.title }}
                  <span class="badge">{{ item.asset_type }}</span>
                  <span class="badge">{{ item.priority }}</span>
                </div>
                <div class="row-sub">
                  Est: {{ formatMoney(item.estimated_cost) }} | Shop: {{ item.shop || '-' }} | Status: {{ item.status }}
                </div>
                <div class="row-sub" v-if="item.converted_at">
                  Converted at: {{ item.converted_at }} | Asset:
                  <a v-if="item.converted_asset_id" :href="assetLink(item.converted_asset_id)">{{ item.converted_asset_id }}</a>
                </div>
                <div class="row-sub" v-if="depr[item.id]">
                  Depreciation: {{ depr[item.id].method }} | Book now: {{ formatMoney(depr[item.id].book_value_now) }}
                </div>
              </div>
              <div class="row-extra" style="display: flex; gap: var(--density-gap); align-items: center; flex-wrap: wrap">
                <app-button v-if="item.status !== 'purchased'" size="sm" @click="openConvert(item)">Convert</app-button>
                <app-button size="sm" variant="ghost" @click="openEdit(item)">Edit</app-button>
                <app-button size="sm" variant="danger" @click="remove(item)">Delete</app-button>
              </div>
            </div>
            <div v-if="items.length === 0" class="empty">
              <div class="empty-title">No items</div>
              <div class="empty-text">Add items to your buy list.</div>
            </div>
          </div>
        </section>
      </div>

      <app-modal :open="editorOpen" :title="editingId ? 'Edit buy item' : 'New buy item'" :sheet="true" @close="closeEditor">
        <div class="form-grid" style="padding: var(--density-card-pad)">
          <app-input v-model="form.title" label="Title" placeholder="Title" />
          <app-input v-model="form.description" label="Description" placeholder="Description" />
          <app-input v-model="form.asset_type" label="Asset type" placeholder="Asset type" />
          <app-input v-model.number="form.estimated_cost" type="number" label="Estimated cost" placeholder="Estimated cost" />
          <app-select v-model="form.priority" label="Priority" :options="priorityOptions" />
          <app-input v-model="form.target_date" label="Target date (YYYY-MM-DD)" placeholder="YYYY-MM-DD" />
          <app-input v-model="form.shop" label="Shop" placeholder="Shop" />
          <app-input v-model="form.url" label="URL" placeholder="URL" />
          <app-select v-model="form.bank_id" label="Bank" :options="bankOptions" placeholder="Select bank" />
          <div class="form-actions">
            <app-button @click="save">Save</app-button>
          </div>
        </div>
      </app-modal>

      <app-modal :open="convertOpen" :title="'Convert to asset'" :sheet="true" @close="closeConvert">
        <div class="form-grid" style="padding: var(--density-card-pad)">
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
          <div class="form-actions">
            <app-button @click="doConvert">Convert</app-button>
          </div>
        </div>
      </app-modal>
    </ion-content>
  </ion-page>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { IonPage, IonContent } from '@ionic/vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppModal from '@/components/ui/AppModal.vue'
import SyncChip from '@/components/ui/SyncChip.vue'
import { request, ApiError } from '@/api/client'
import { useFinanceStore } from '@/stores/financeStore'
import { formatMoney } from '@/utils/money'

const filter = ref<'all' | 'planned' | 'purchased'>('all')
const items = ref<any[]>([])
const banks = ref<any[]>([])
const editorOpen = ref(false)
const editingId = ref<string | null>(null)
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

function setFilter(f: any) {
  filter.value = f
  load()
}

async function load() {
  const res: any = await request('/api/buy-list', { method: 'GET' })
  let list = res || []
  if (filter.value === 'planned') list = list.filter((x: any) => x.status === 'planned')
  if (filter.value === 'purchased') list = list.filter((x: any) => x.status === 'purchased')
  items.value = list
  for (const it of items.value) {
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
  banks.value = res || []
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

async function remove(item: any) {
  await request(`/api/buy-list/${item.id}`, { method: 'DELETE' })
  await load()
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
</style>
