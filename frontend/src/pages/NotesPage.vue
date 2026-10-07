<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div class="page-head-text">
            <h1 class="page-title">Notes</h1>
            <p class="page-subtitle">Capture ideas, lists and reminders</p>
          </div>
          <sync-chip />
        </header>

        <section class="section">
          <div class="row-list" style="padding: var(--density-card-pad)">
            <div class="row-item">
              <div class="row-main">
                <app-input v-model="search" placeholder="Search notes..." @update:model-value="debouncedSearch" />
              </div>
            </div>
            <div class="row-item">
              <div class="row-main" style="display: flex; gap: var(--density-gap); flex-wrap: wrap; align-items: center">
                <div class="chips">
                  <button class="chip" :class="{ 'chip-active': filter === 'all' }" @click="setFilter('all')">All</button>
                  <button class="chip" :class="{ 'chip-active': filter === 'pinned' }" @click="setFilter('pinned')">Pinned</button>
                  <button class="chip" :class="{ 'chip-active': filter === 'archived' }" @click="setFilter('archived')">Archived</button>
                  <button class="chip" :class="{ 'chip-active': filter === 'trashed' }" @click="setFilter('trashed')">Trash</button>
                </div>
                <app-select v-model="colorFilter" placeholder="All colors" :options="colorOptions" @update:model-value="loadNotes" />
                <app-select v-model="labelFilter" placeholder="All labels" :options="labelOptions" @update:model-value="loadNotes" />
                <app-button @click="openCreate">New note</app-button>
              </div>
            </div>
          </div>
        </section>

        <section class="section">
          <div class="notes-grid">
            <article v-for="note in notes" :key="note.id" class="card note-card" :class="noteCardClass(note)">
              <div class="card-tight" style="display: flex; justify-content: space-between; align-items: flex-start; gap: var(--density-gap)">
                <div class="card-label clamp-1">{{ note.title || '(Untitled)' }}</div>
                <ion-icon v-if="note.pinned" name="pin" style="font-size: 20px; color: var(--accent)" />
              </div>
              <div class="card-tight" v-if="note.kind === 'text' && note.body_text">
                <div class="text-muted" style="white-space: pre-wrap">{{ note.body_text }}</div>
              </div>
              <div class="card-tight" v-if="note.kind === 'list' && note.items">
                <div v-for="it in note.items.slice(0, 10)" :key="it.id" style="display: flex; align-items: center; gap: var(--density-gap); margin-bottom: 4px">
                  <input type="checkbox" :checked="!!it.checked" disabled />
                  <span :class="{ 'text-faint': it.checked }" style="text-decoration: line-through" v-if="it.checked">{{ it.text }}</span>
                  <span v-else>{{ it.text }}</span>
                </div>
              </div>
              <div class="card-tight" v-if="note.labels && note.labels.length">
                <div class="chips">
                  <span v-for="lbl in note.labels" :key="lbl.id" class="chip">{{ lbl.name }}</span>
                </div>
              </div>
              <div class="card-foot" style="display: flex; gap: var(--density-gap); flex-wrap: wrap; font-size: 18px; color: var(--text-muted)">
                <ion-icon name="color-palette-outline" style="cursor: pointer" @click.stop="toggleColorPicker(note)" />
                <ion-icon :name="note.pinned ? 'pin' : 'pin-outline'" style="cursor: pointer" @click.stop="togglePin(note)" />
                <ion-icon :name="note.archived ? 'archive' : 'archive-outline'" style="cursor: pointer" @click.stop="toggleArchive(note)" />
                <ion-icon v-if="!note.trashed_at" name="trash-outline" style="cursor: pointer" @click.stop="trash(note)" />
                <ion-icon v-else name="arrow-undo-outline" style="cursor: pointer" @click.stop="restore(note)" />
                <ion-icon v-if="note.trashed_at" name="close-outline" style="cursor: pointer" @click.stop="remove(note)" />
                <ion-icon name="create-outline" style="cursor: pointer" @click.stop="openEdit(note)" />
              </div>
              <div v-if="showColorPickerFor === note.id" class="card-tight" style="display: flex; gap: 6px; flex-wrap: wrap">
                <button v-for="c in colors" :key="c" class="swatch" :class="swatchClass(c)" @click="setColor(note, c)"></button>
              </div>
            </article>
          </div>
          <div v-if="notes.length === 0" class="empty">
            <div class="empty-icon">🗒️</div>
            <div class="empty-title">No notes yet</div>
            <div class="empty-text">Create your first note to get started.</div>
          </div>
        </section>
      </div>

      <app-modal :open="editorOpen" :title="editingId ? 'Edit note' : 'New note'" :sheet="true" @close="closeEditor">
        <div class="form-grid" style="padding: var(--density-card-pad)">
          <div class="form-row-2">
            <app-input v-model="form.title" label="Title" placeholder="Title" />
          </div>
          <div class="row-item">
            <div class="row-main" style="display: flex; gap: var(--density-gap)">
              <button class="chip" :class="{ 'chip-active': form.kind === 'text' }" @click="form.kind = 'text'">Text</button>
              <button class="chip" :class="{ 'chip-active': form.kind === 'list' }" @click="form.kind = 'list'">List</button>
            </div>
          </div>
          <div class="row-item" v-if="form.kind === 'text'">
            <div class="row-main">
              <textarea v-model="form.body_text" placeholder="Take a note..." style="width: 100%; min-height: 120px; background: var(--surface-2); color: var(--text); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: var(--density-row-pad-y) var(--density-row-pad-x); font-family: var(--font-body)"></textarea>
            </div>
          </div>
          <div class="row-item" v-if="form.kind === 'list'">
            <div class="row-main" style="display: flex; flex-direction: column; gap: var(--density-gap)">
              <div v-for="(it, idx) in form.items" :key="idx" style="display: flex; gap: var(--density-gap); align-items: center">
                <input type="checkbox" v-model="it.checked" />
                <app-input v-model="it.text" placeholder="List item" />
                <button class="chip" @click="removeItem(idx)">Remove</button>
              </div>
              <app-button size="sm" @click="addItem">Add item</app-button>
            </div>
          </div>
          <div class="form-row-2">
            <app-select v-model="form.color" label="Color" :options="colorOptions" />
            <div style="display: flex; flex-direction: column; gap: var(--density-gap); padding-top: 20px">
              <label style="display: flex; align-items: center; gap: 8px"><input type="checkbox" v-model="form.pinned" /> Pinned</label>
              <label style="display: flex; align-items: center; gap: 8px"><input type="checkbox" v-model="form.archived" /> Archived</label>
            </div>
          </div>
          <div class="form-row-2">
            <app-select v-model="form.label_ids" label="Labels" :options="labelOptions" multiple />
          </div>
          <div v-if="formError" class="form-error">{{ formError }}</div>
          <div class="form-actions">
            <app-button @click="saveNote">Save</app-button>
          </div>
        </div>
      </app-modal>
    </ion-content>
  </ion-page>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { IonPage, IonContent, IonIcon } from '@ionic/vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppModal from '@/components/ui/AppModal.vue'
import SyncChip from '@/components/ui/SyncChip.vue'
import { request, ApiError } from '@/api/client'

const colors = ['DEFAULT', 'WHITE', 'RED', 'ORANGE', 'YELLOW', 'GREEN', 'TEAL', 'BLUE', 'DARKBLUE', 'PURPLE', 'PINK', 'BROWN', 'GRAY']

const notes = ref<any[]>([])
const labels = ref<any[]>([])
const search = ref('')
const filter = ref<'all' | 'pinned' | 'archived' | 'trashed'>('all')
const colorFilter = ref('')
const labelFilter = ref('')
const showColorPickerFor = ref<string | null>(null)
const editorOpen = ref(false)
const editingId = ref<string | null>(null)
const form = ref<any>({
  title: '',
  body_text: '',
  kind: 'text',
  color: 'DEFAULT',
  pinned: false,
  archived: false,
  label_ids: [] as string[],
  items: [] as any[]
})
const originalItems = ref<any[]>([])
const formError = ref('')

let debounce: any
const debouncedSearch = () => {
  clearTimeout(debounce)
  debounce = setTimeout(loadNotes, 250)
}

const setFilter = (f: any) => {
  filter.value = f
  loadNotes()
}

const colorOptions = colors.map((c) => ({ value: c, label: c }))
const labelOptions = computed(() => labels.value.map((l: any) => ({ value: l.id, label: l.name })))

const noteCardClass = (note: any) => {
  const c = note.color
  if (c && c !== 'DEFAULT') return `notecard note-${String(c).toLowerCase()}`
  return 'notecard'
}

const swatchClass = (c: string) => {
  const cls = String(c).toLowerCase()
  return cls === 'default' ? 'notecard' : `note-${cls}`
}

async function loadNotes() {
  const params: any = {}
  if (search.value) params.q = search.value
  if (colorFilter.value) params.color = colorFilter.value
  if (labelFilter.value) params.label = labelFilter.value
  if (filter.value === 'pinned') params.pinned = 1
  if (filter.value === 'archived') params.archived = 1
  if (filter.value === 'trashed') params.trashed = 1
  const res: any = await request('/api/notes', { method: 'GET', query: params })
  notes.value = res?.data || res || []
}

async function loadLabels() {
  const res: any = await request('/api/notes/labels', { method: 'GET' })
  labels.value = res || []
}

function openCreate() {
  editingId.value = null
  form.value = {
    title: '',
    body_text: '',
    kind: 'text',
    color: 'DEFAULT',
    pinned: false,
    archived: false,
    label_ids: [],
    items: []
  }
  originalItems.value = []
  formError.value = ''
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
    label_ids: (note.labels || []).map((l: any) => l.id),
    items: (note.items || []).map((it: any) => ({
      id: it.id,
      text: it.text,
      checked: !!it.checked,
      parent_id: it.parent_id ?? null,
      position: it.position
    }))
  }
  originalItems.value = (note.items || []).map((it: any) => ({
    id: it.id,
    text: it.text,
    checked: !!it.checked,
    parent_id: it.parent_id ?? null,
    position: it.position
  }))
  formError.value = ''
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
  try {
    formError.value = ''
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
    if (editingId.value) {
      await request(`/api/notes/${editingId.value}`, { method: 'PUT', body: payload })
      if (form.value.kind === 'list') {
        const orig = originalItems.value
        const curr = form.value.items as any[]
        const origById = new Map(orig.filter((i: any) => i.id).map((i: any) => [i.id, i]))
        const currIdsSet = new Set(curr.filter((i: any) => i.id).map((i: any) => i.id))
        const origIdsSet = new Set(origById.keys())
        for (const it of curr) {
          if (!it.id) {
            await request(`/api/notes/${editingId.value}/items`, {
              method: 'POST',
              body: { text: it.text, checked: it.checked ? 1 : 0 }
            })
            continue
          }
          const o = origById.get(it.id)
          if (o && (o.text !== it.text || (!!o.checked) !== (!!it.checked))) {
            await request(`/api/notes/${editingId.value}/items/${it.id}`, {
              method: 'PUT',
              body: { text: it.text, checked: it.checked ? 1 : 0 }
            })
          }
        }
        for (const oid of origIdsSet) {
          if (!currIdsSet.has(oid)) {
            await request(`/api/notes/${editingId.value}/items/${oid}`, { method: 'DELETE' })
          }
        }
      }
    } else {
      const payloadNew: any = { ...payload }
      if (form.value.kind === 'list') {
        payloadNew.items = form.value.items.map((it: any) => ({
          text: it.text,
          checked: it.checked ? 1 : 0,
          parent_id: it.parent_id ?? null
        }))
      }
      await request('/api/notes', { method: 'POST', body: payloadNew })
    }
    closeEditor()
    await loadNotes()
  } catch (e) {
    formError.value = e instanceof ApiError ? e.message : String(e)
  }
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
.notecard { border: 1px solid var(--border-strong); }
.note-red { --note-rgb: 210, 80, 70; }
.note-orange { --note-rgb: 220, 140, 60; }
.note-yellow { --note-rgb: 210, 170, 50; }
.note-green { --note-rgb: 90, 160, 90; }
.note-teal { --note-rgb: 60, 160, 160; }
.note-blue { --note-rgb: 90, 150, 230; }
.note-darkblue { --note-rgb: 90, 110, 220; }
.note-purple { --note-rgb: 150, 110, 220; }
.note-pink { --note-rgb: 230, 110, 170; }
.note-brown { --note-rgb: 150, 110, 80; }
.note-gray { --note-rgb: 140, 140, 135; }
.note-white { --note-rgb: 200, 198, 190; }
.notecard {
  background: linear-gradient(0deg, rgba(var(--note-rgb), .16), rgba(var(--note-rgb), .16)), var(--surface);
  border: 1px solid rgba(var(--note-rgb), .55);
}
</style>
