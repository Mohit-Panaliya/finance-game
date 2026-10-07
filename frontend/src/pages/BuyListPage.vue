<template>
  <ion-page>
    <ion-header :translucent="true">
      <ion-toolbar>
        <ion-title>Buy List</ion-title>
      </ion-toolbar>
    </ion-header>
    <ion-content :fullscreen="true">
      <div class="content-container">
        <div class="toolbar">
          <div class="filters">
            <ion-chip :color="filter==='all'?'primary':''" @click="setFilter('all')">All</ion-chip>
            <ion-chip :color="filter==='planned'?'primary':''" @click="setFilter('planned')">Pending</ion-chip>
            <ion-chip :color="filter==='purchased'?'primary':''" @click="setFilter('purchased')">Purchased</ion-chip>
          </div>
          <app-button @click="openCreate">New item</app-button>
        </div>

        <ion-list>
          <ion-item v-for="item in items" :key="item.id">
            <ion-label>
              <h2>{{ item.title }} <ion-badge>{{ item.asset_type }}</ion-badge> <ion-badge color="medium">{{ item.priority }}</ion-badge></h2>
              <p>Est: {{ item.estimated_cost }} | Shop: {{ item.shop || '-' }} | Status: {{ item.status }}</p>
              <p v-if="item.converted_at">Converted at: {{ item.converted_at }} | Asset: <a v-if="item.converted_asset_id" :href="assetLink(item.converted_asset_id)">{{ item.converted_asset_id }}</a></p>
              <div v-if="depr[item.id]" class="depr">
                <small>Depreciation: {{ depr[item.id].method }} | Book now: {{ depr[item.id].book_value_now }}</small>
              </div>
            </ion-label>
            <div slot="end" class="row-actions">
              <app-button v-if="item.status !== 'purchased'" @click="openConvert(item)">Convert</app-button>
              <app-button fill="clear" @click="openEdit(item)">Edit</app-button>
              <app-button fill="clear" color="danger" @click="remove(item)">Delete</app-button>
            </div>
          </ion-item>
        </ion-list>
      </div>

      <ion-modal :is-open="editorOpen" @didDismiss="closeEditor">
        <ion-header>
          <ion-toolbar>
            <ion-title>{{ editingId ? 'Edit' : 'New' }} buy item</ion-title>
            <ion-buttons slot="end"><app-button @click="save">Save</app-button></ion-buttons>
          </ion-toolbar>
        </ion-header>
        <ion-content class="modal-content">
          <ion-item><ion-input v-model="form.title" placeholder="Title" /></ion-item>
          <ion-item><ion-input v-model="form.description" placeholder="Description" /></ion-item>
          <ion-item><ion-input v-model="form.asset_type" placeholder="Asset type" /></ion-item>
          <ion-item><ion-input v-model.number="form.estimated_cost" type="number" placeholder="Estimated cost" /></ion-item>
          <ion-item>
            <ion-select v-model="form.priority" interface="popover" placeholder="Priority">
              <ion-select-option value="high">High</ion-select-option>
              <ion-select-option value="medium">Medium</ion-select-option>
              <ion-select-option value="low">Low</ion-select-option>
            </ion-select>
          </ion-item>
          <ion-item><ion-input v-model="form.target_date" placeholder="Target date (YYYY-MM-DD)" /></ion-item>
          <ion-item><ion-input v-model="form.shop" placeholder="Shop" /></ion-item>
          <ion-item><ion-input v-model="form.url" placeholder="URL" /></ion-item>
          <ion-item>
            <ion-select v-model="form.bank_id" interface="popover" placeholder="Bank">
              <ion-select-option v-for="b in banks" :key="b.id" :value="b.id">{{ b.name }}</ion-select-option>
            </ion-select>
          </ion-item>
        </ion-content>
      </ion-modal>

      <ion-modal :is-open="convertOpen" @didDismiss="closeConvert">
        <ion-header>
          <ion-toolbar>
            <ion-title>Convert to asset</ion-title>
            <ion-buttons slot="end"><app-button @click="doConvert">Convert</app-button></ion-buttons>
          </ion-toolbar>
        </ion-header>
        <ion-content class="modal-content">
          <ion-item><ion-input v-model.number="cform.actual_cost" type="number" placeholder="Actual cost" /></ion-item>
          <ion-item><ion-input v-model="cform.purchase_date" placeholder="Purchase date (YYYY-MM-DD)" /></ion-item>
          <ion-item>
            <ion-select v-model="cform.bank_id" interface="popover" placeholder="Bank">
              <ion-select-option v-for="b in banks" :key="b.id" :value="b.id">{{ b.name }}</ion-select-option>
            </ion-select>
          </ion-item>
          <ion-item><ion-input v-model="cform.asset_name" placeholder="Asset name" /></ion-item>
          <ion-item><ion-input v-model="cform.category" placeholder="Category" /></ion-item>
          <ion-item><ion-input v-model="cform.description" placeholder="Description" /></ion-item>
          <ion-item><ion-input v-model="cform.location" placeholder="Location" /></ion-item>
          <ion-item>
            <ion-select v-model="cform.depreciation_method" interface="popover" placeholder="Depreciation method">
              <ion-select-option value="straight_line">Straight line</ion-select-option>
              <ion-select-option value="declining_balance">Declining balance</ion-select-option>
              <ion-select-option value="double_declining">Double declining</ion-select-option>
              <ion-select-option value="units_of_production">Units of production</ion-select-option>
            </ion-select>
          </ion-item>
          <ion-item><ion-input v-model.number="cform.useful_life_months" type="number" placeholder="Useful life months" /></ion-item>
          <ion-item><ion-input v-model.number="cform.salvage_value" type="number" placeholder="Salvage value" /></ion-item>
        </ion-content>
      </ion-modal>
    </ion-content>
  </ion-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  IonPage, IonHeader, IonToolbar, IonTitle, IonContent, IonChip, IonList, IonItem, IonLabel,
  IonBadge, IonModal, IonButtons, IonInput, IonSelect, IonSelectOption
} from '@ionic/vue'
import AppButton from '@/components/ui/AppButton.vue'
import { request } from '@/api/client'

const filter = ref<'all'|'planned'|'purchased'>('all')
const items = ref<any[]>([])
const banks = ref<any[]>([])
const editorOpen = ref(false)
const editingId = ref<string|null>(null)
const form = ref<any>({ title: '', description: '', asset_type: 'Other', estimated_cost: 0, priority: 'medium', target_date: '', shop: '', url: '', bank_id: '' })
const convertOpen = ref(false)
const convertingId = ref<string|null>(null)
const cform = ref<any>({ actual_cost: 0, purchase_date: '', bank_id: '', asset_name: '', category: '', description: '', location: '', depreciation_method: 'straight_line', useful_life_months: 0, salvage_value: 0 })
const depr = ref<Record<string, any>>({})

function setFilter(f:any){ filter.value=f; load() }

async function load() {
  const res = await request('/api/buy-list', { method: 'GET' })
  let list = (res as any) || []
  if (filter.value === 'planned') list = list.filter((x:any)=>x.status==='planned')
  if (filter.value === 'purchased') list = list.filter((x:any)=>x.status==='purchased')
  items.value = list
  // fetch depreciation for purchased items
  for (const it of items.value) {
    if (it.converted_asset_id) {
      try {
        const d = await request(`/api/assets/${it.converted_asset_id}/depreciation`, { method: 'GET' })
        depr.value[it.id] = d
      } catch (e) { /* ignore */ }
    }
  }
}

async function loadBanks() {
  const res = await request('/api/banks', { method: 'GET' })
  banks.value = (res as any) || []
}

function openCreate() {
  editingId.value = null
  form.value = { title: '', description: '', asset_type: 'Other', estimated_cost: 0, priority: 'medium', target_date: '', shop: '', url: '', bank_id: '' }
  editorOpen.value = true
}
function openEdit(item:any) {
  editingId.value = item.id
  form.value = { ...item }
  editorOpen.value = true
}
function closeEditor(){ editorOpen.value=false }
async function save() {
  const payload = { ...form.value }
  if (editingId.value) await request(`/api/buy-list/${editingId.value}`, { method: 'PUT', body: payload })
  else await request('/api/buy-list', { method: 'POST', body: payload })
  closeEditor()
  await load()
}

function openConvert(item:any) {
  convertingId.value = item.id
  cform.value = {
    actual_cost: item.estimated_cost,
    purchase_date: new Date().toISOString().slice(0,10),
    bank_id: item.bank_id || '',
    asset_name: item.title,
    category: '',
    description: item.description || '',
    location: '',
    depreciation_method: 'straight_line',
    useful_life_months: 0,
    salvage_value: 0
  }
  convertOpen.value = true
}
function closeConvert(){ convertOpen.value=false }
async function doConvert() {
  if (!convertingId.value) return
  await request(`/api/buy-list/${convertingId.value}/convert`, { method: 'POST', body: cform.value })
  closeConvert()
  await load()
}

async function remove(item:any) {
  await request(`/api/buy-list/${item.id}`, { method: 'DELETE' })
  await load()
}

function assetLink(id:string) {
  // check if assets detail exists generically - link to accounts/assets if pattern matches? router has /accounts/:entity/:id but for assets maybe /accounts/assets/:id? accounts page is generic
  // but we can link to /accounts/assets/:id style if entity is assets - yes based on router
  return `/accounts/assets/${id}`
}

onMounted(async ()=>{
  await loadBanks()
  await load()
})
</script>

<style scoped>
.content-container { padding: 16px; }
.toolbar { display: flex; gap: 12px; align-items: center; flex-wrap: wrap; margin-bottom: 16px; justify-content: space-between; }
.filters { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.row-actions { display: flex; gap: 8px; align-items: center; }
.depr { margin-top: 4px; }
</style>
