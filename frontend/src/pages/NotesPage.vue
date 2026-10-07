<template>
  <ion-page>
    <ion-header :translucent="true">
      <ion-toolbar>
        <ion-title>Notes</ion-title>
      </ion-toolbar>
    </ion-header>
    <ion-content :fullscreen="true">
      <div class="content-container">
        <div class="toolbar">
          <ion-searchbar v-model="search" placeholder="Search notes..." @ionInput="debouncedSearch" />
          <div class="filters">
            <ion-chip :color="filter==='all'?'primary':''" @click="setFilter('all')">All</ion-chip>
            <ion-chip :color="filter==='pinned'?'primary':''" @click="setFilter('pinned')">Pinned</ion-chip>
            <ion-chip :color="filter==='archived'?'primary':''" @click="setFilter('archived')">Archived</ion-chip>
            <ion-chip :color="filter==='trashed'?'primary':''" @click="setFilter('trashed')">Trash</ion-chip>
            <ion-select interface="popover" placeholder="Color" v-model="colorFilter" @ionChange="loadNotes">
              <ion-select-option value="">All colors</ion-select-option>
              <ion-select-option v-for="c in colors" :key="c" :value="c">{{ c }}</ion-select-option>
            </ion-select>
            <ion-select interface="popover" placeholder="Label" v-model="labelFilter" @ionChange="loadNotes">
              <ion-select-option value="">All labels</ion-select-option>
              <ion-select-option v-for="l in labels" :key="l.id" :value="l.id">{{ l.name }}</ion-select-option>
            </ion-select>
          </div>
          <app-button @click="openCreate">New note</app-button>
        </div>

        <div class="notes-grid">
          <ion-card v-for="note in notes" :key="note.id" :style="cardStyle(note)" class="note-card">
            <ion-card-header>
              <div class="card-top">
                <ion-card-title>{{ note.title || '(Untitled)' }}</ion-card-title>
                <div>
                  <ion-icon v-if="note.pinned" name="pin" class="pin" />
                </div>
              </div>
            </ion-card-header>
            <ion-card-content>
              <div v-if="note.kind === 'text' && note.body_text" class="body">{{ note.body_text }}</div>
              <div v-if="note.kind === 'list' && note.items" class="list">
                <div v-for="it in note.items.slice(0, 10)" :key="it.id" class="list-item">
                  <ion-checkbox :checked="!!it.checked" disabled />
                  <span :class="{ checked: it.checked }">{{ it.text }}</span>
                </div>
              </div>
              <div v-if="note.labels && note.labels.length" class="labels">
                <ion-chip v-for="lbl in note.labels" :key="lbl.id" outline>{{ lbl.name }}</ion-chip>
              </div>
              <div class="actions">
                <ion-icon name="color-palette-outline" @click.stop="toggleColorPicker(note)" />
                <ion-icon :name="note.pinned ? 'pin' : 'pin-outline'" @click.stop="togglePin(note)" />
                <ion-icon :name="note.archived ? 'archive' : 'archive-outline'" @click.stop="toggleArchive(note)" />
                <ion-icon v-if="!note.trashed_at" name="trash-outline" @click.stop="trash(note)" />
                <ion-icon v-else name="arrow-undo-outline" @click.stop="restore(note)" />
                <ion-icon v-if="note.trashed_at" name="close-outline" @click.stop="remove(note)" />
                <ion-icon name="create-outline" @click.stop="openEdit(note)" />
              </div>
              <div v-if="showColorPickerFor === note.id" class="color-picker">
                <div v-for="c in colors" :key="c" class="swatch" :style="swatchStyle(c)" @click="setColor(note, c)"></div>
              </div>
            </ion-card-content>
          </ion-card>
        </div>
      </div>

      <ion-modal :is-open="editorOpen" @didDismiss="closeEditor">
        <ion-header>
          <ion-toolbar>
            <ion-title>{{ editingId ? 'Edit note' : 'New note' }}</ion-title>
            <ion-buttons slot="end">
              <app-button @click="saveNote">Save</app-button>
            </ion-buttons>
          </ion-toolbar>
        </ion-header>
        <ion-content class="modal-content">
          <ion-item>
            <ion-input v-model="form.title" placeholder="Title" />
          </ion-item>
          <div class="kind-toggle">
            <ion-chip :color="form.kind==='text'?'primary':''" @click="form.kind='text'">Text</ion-chip>
            <ion-chip :color="form.kind==='list'?'primary':''" @click="form.kind='list'">List</ion-chip>
          </div>
          <ion-item v-if="form.kind==='text'">
            <ion-textarea v-model="form.body_text" auto-grow placeholder="Take a note..." />
          </ion-item>
          <div v-if="form.kind==='list'" class="list-editor">
            <div v-for="(it, idx) in form.items" :key="idx" class="list-row">
              <ion-checkbox v-model="it.checked" />
              <ion-input v-model="it.text" placeholder="List item" />
              <ion-icon name="close-outline" @click="removeItem(idx)" />
            </div>
            <app-button @click="addItem">Add item</app-button>
          </div>
          <div class="editor-meta">
            <ion-select interface="popover" placeholder="Color" v-model="form.color">
              <ion-select-option v-for="c in colors" :key="c" :value="c">{{ c }}</ion-select-option>
            </ion-select>
            <ion-checkbox v-model="form.pinned">Pinned</ion-checkbox>
            <ion-checkbox v-model="form.archived">Archived</ion-checkbox>
            <ion-select interface="popover" placeholder="Labels" v-model="form.label_ids" multiple>
              <ion-select-option v-for="l in labels" :key="l.id" :value="l.id">{{ l.name }}</ion-select-option>
            </ion-select>
          </div>
        </ion-content>
      </ion-modal>
    </ion-content>
  </ion-page>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  IonPage, IonHeader, IonToolbar, IonTitle, IonContent, IonSearchbar, IonChip, IonCard, IonCardHeader,
  IonCardTitle, IonCardContent, IonIcon, IonModal, IonButtons, IonItem, IonInput, IonTextarea,
  IonSelect, IonSelectOption, IonCheckbox
} from '@ionic/vue'
import AppButton from '@/components/ui/AppButton.vue'
import { request } from '@/api/client'

const colors = ['DEFAULT','WHITE','RED','ORANGE','YELLOW','GREEN','TEAL','BLUE','DARKBLUE','PURPLE','PINK','BROWN','GRAY']

const notes = ref<any[]>([])
const labels = ref<any[]>([])
const search = ref('')
const filter = ref<'all'|'pinned'|'archived'|'trashed'>('all')
const colorFilter = ref('')
const labelFilter = ref('')
const showColorPickerFor = ref<string|null>(null)
const editorOpen = ref(false)
const editingId = ref<string|null>(null)
const form = ref<any>({
  title: '', body_text: '', kind: 'text', color: 'DEFAULT', pinned: false, archived: false, label_ids: [], items: []
})

let debounce: any
const debouncedSearch = () => {
  clearTimeout(debounce)
  debounce = setTimeout(loadNotes, 250)
}

const setFilter = (f: any) => { filter.value = f; loadNotes() }

const cardStyle = (note: any) => {
  const bg = getColorBg(note.color)
  return { background: bg, borderColor: 'var(--border)' }
}
const swatchStyle = (c: string) => ({ background: getColorBg(c) })

function getColorBg(c: string) {
  // Use theme tokens lightly - approximate
  switch(c) {
    case 'RED': return '#ffe5e5'
    case 'ORANGE': return '#fff2e5'
    case 'YELLOW': return '#fffbe5'
    case 'GREEN': return '#e6f7ee'
    case 'TEAL': return '#e6f7f7'
    case 'BLUE': return '#e6f4ff'
    case 'DARKBLUE': return '#e6ecff'
    case 'PURPLE': return '#f0e6ff'
    case 'PINK': return '#ffe6f2'
    case 'BROWN': return '#f5efe6'
    case 'GRAY': return '#f2f2f2'
    case 'WHITE': return '#ffffff'
    default: return 'var(--surface)'
  }
}

async function loadNotes() {
  const params: any = {}
  if (search.value) params.q = search.value
  if (colorFilter.value) params.color = colorFilter.value
  if (labelFilter.value) params.label = labelFilter.value
  if (filter.value === 'pinned') params.pinned = true
  if (filter.value === 'archived') params.archived = true
  if (filter.value === 'trashed') params.trashed = true
  const res = await request('/api/notes', { method: 'GET', query: params })
  notes.value = (res as any).data || (res as any) || []
}

async function loadLabels() {
  const res = await request('/api/labels', { method: 'GET' })
  labels.value = (res as any) || []
}

function openCreate() {
  editingId.value = null
  form.value = { title: '', body_text: '', kind: 'text', color: 'DEFAULT', pinned: false, archived: false, label_ids: [], items: [] }
  editorOpen.value = true
}

function openEdit(note: any) {
  editingId.value = note.id
  form.value = {
    title: note.title,
    body_text: note.body_text || '',
    kind: note.kind,
    color: note.color,
    pinned: !!note.pinned,
    archived: !!note.archived,
    label_ids: (note.labels || []).map((l:any)=>l.id),
    items: (note.items || []).map((it:any)=>({ text: it.text, checked: !!it.checked, parent_id: it.parent_id }))
  }
  editorOpen.value = true
}

function closeEditor() {
  editorOpen.value = false
}

function addItem() {
  form.value.items.push({ text: '', checked: false })
}

function removeItem(idx: number) {
  form.value.items.splice(idx, 1)
}

async function saveNote() {
  const payload: any = {
    title: form.value.title,
    kind: form.value.kind,
    color: form.value.color,
    pinned: form.value.pinned ? 1 : 0,
    archived: form.value.archived ? 1 : 0,
    label_ids: form.value.label_ids
  }
  if (form.value.kind === 'text') {
    payload.body_text = form.value.body_text
  }
  if (form.value.kind === 'list') {
    payload.items = form.value.items.map((it:any)=>({ text: it.text, checked: it.checked ? 1 : 0 }))
  }
  if (editingId.value) {
    await request(`/api/notes/${editingId.value}`, { method: 'PUT', body: payload })
  } else {
    await request('/api/notes', { method: 'POST', body: payload })
  }
  closeEditor()
  await loadNotes()
}

function toggleColorPicker(note: any) {
  showColorPickerFor.value = showColorPickerFor.value === note.id ? null : note.id
}

async function setColor(note: any, c: string) {
  await request(`/api/notes/${note.id}`, { method: 'PUT', body: { color: c } })
  showColorPickerFor.value = null
  await loadNotes()
}

async function togglePin(note: any) {
  await request(`/api/notes/${note.id}`, { method: 'PUT', body: { pinned: note.pinned ? 0 : 1 } })
  await loadNotes()
}

async function toggleArchive(note: any) {
  await request(`/api/notes/${note.id}`, { method: 'PUT', body: { archived: note.archived ? 0 : 1 } })
  await loadNotes()
}

async function trash(note: any) {
  await request(`/api/notes/${note.id}/trash`, { method: 'PUT', body: {} })
  await loadNotes()
}

async function restore(note: any) {
  await request(`/api/notes/${note.id}`, { method: 'PUT', body: { trashed_at: '' } })
  await loadNotes()
}

async function remove(note: any) {
  await request(`/api/notes/${note.id}`, { method: 'DELETE' })
  await loadNotes()
}

onMounted(async () => {
  await loadLabels()
  await loadNotes()
})
</script>

<style scoped>
.content-container { padding: 16px; }
.toolbar { display: flex; gap: 12px; align-items: center; flex-wrap: wrap; margin-bottom: 16px; }
.filters { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.notes-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 12px; }
.note-card { margin: 0; border: 1px solid var(--border); }
.card-top { display: flex; justify-content: space-between; align-items: start; }
.pin { font-size: 20px; }
.body { white-space: pre-wrap; }
.list-item { display: flex; align-items: center; gap: 8px; margin-bottom: 4px; }
.list-item .checked { text-decoration: line-through; opacity: 0.6; }
.labels { display: flex; gap: 4px; flex-wrap: wrap; margin-top: 8px; }
.actions { display: flex; gap: 12px; margin-top: 12px; font-size: 20px; cursor: pointer; }
.color-picker { display: flex; gap: 6px; flex-wrap: wrap; margin-top: 8px; }
.swatch { width: 20px; height: 20px; border-radius: 50%; border: 1px solid var(--border); cursor: pointer; }
.kind-toggle { display: flex; gap: 8px; padding: 8px 16px; }
.list-editor { padding: 8px 16px; }
.list-row { display: flex; gap: 8px; align-items: center; margin-bottom: 8px; }
.editor-meta { display: flex; flex-direction: column; gap: 8px; padding: 8px 16px; }
</style>
