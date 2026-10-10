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
          <app-input v-model="search" placeholder="Search notes" @update:model-value="debouncedSearch" />

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

          <div class="form-row-2">
            <app-select
              v-model="colorFilter"
              placeholder="All colors"
              :options="colorOptions"
              @update:model-value="loadNotes"
            />
            <app-select
              v-model="labelFilter"
              placeholder="All labels"
              :options="labelOptions"
              @update:model-value="loadNotes"
            />
          </div>
        </section>

        <section class="section">
          <div v-if="notes.length" class="notes-grid">
            <article
              v-for="note in notes"
              :key="note.id"
              class="card note-card"
              :class="noteCardClass(note)"
            >
              <div class="note-head">
                <p class="note-title clamp-1">{{ note.title || '(Untitled)' }}</p>
                <span v-if="note.archived && filter !== 'archived'" class="badge">Archived</span>
              </div>

              <p v-if="note.kind === 'text' && note.body_text" class="note-body text-muted">
                {{ note.body_text }}
              </p>

              <ul v-else-if="note.kind === 'list' && note.items && note.items.length" class="note-list">
                <li
                  v-for="it in note.items.slice(0, 10)"
                  :key="it.id"
                  class="note-check"
                  :class="{ 'note-check-done': it.checked }"
                >
                  <input type="checkbox" :checked="!!it.checked" disabled tabindex="-1" />
                  <span class="clamp-1">{{ it.text }}</span>
                </li>
              </ul>

              <div v-if="note.labels && note.labels.length" class="chips note-labels">
                <span v-for="lbl in note.labels" :key="lbl.id" class="chip">{{ lbl.name }}</span>
              </div>

              <div class="card-foot note-foot">
                <button
                  class="icon-btn"
                  :class="{ 'icon-btn-on': note.pinned }"
                  type="button"
                  :aria-label="note.pinned ? 'Unpin note' : 'Pin note'"
                  @click="togglePin(note)"
                >
                  <ion-icon :icon="note.pinned ? pin : pinOutline" />
                </button>
                <button class="icon-btn" type="button" aria-label="Edit note" @click="openEdit(note)">
                  <ion-icon :icon="createOutline" />
                </button>
              </div>
            </article>
          </div>

          <div v-else class="empty">
            <ion-icon class="empty-icon" :icon="documentTextOutline" />
            <p class="empty-title">{{ hasActiveQuery ? 'No notes match' : 'No notes yet' }}</p>
            <p class="empty-text">
              {{
                hasActiveQuery
                  ? 'Nothing here for the current search and filters.'
                  : 'Capture a reminder, a list or an idea — it stays private to your account.'
              }}
            </p>
            <app-button v-if="hasActiveQuery" variant="neutral" size="sm" @click="clearFilters">
              Show all notes
            </app-button>
            <app-button variant="primary" size="sm" @click="openCreate">New note</app-button>
          </div>
        </section>

        <app-button variant="primary" size="md" block @click="openCreate">
          <ion-icon :icon="addOutline" /> New note
        </app-button>
      </div>

      <app-modal
        :open="editorOpen"
        :title="editingId ? 'Edit note' : 'New note'"
        :sheet="true"
        @close="closeEditor"
      >
        <div class="form-grid">
          <app-input v-model="form.title" label="Title" placeholder="Title" />

          <div class="chips ff-hide-scrollbar">
            <button
              class="chip"
              type="button"
              :class="{ 'chip-active': form.kind === 'text' }"
              @click="form.kind = 'text'"
            >
              Text
            </button>
            <button
              class="chip"
              type="button"
              :class="{ 'chip-active': form.kind === 'list' }"
              @click="form.kind = 'list'"
            >
              List
            </button>
          </div>

          <textarea
            v-if="form.kind === 'text'"
            v-model="form.body_text"
            class="note-textarea"
            placeholder="Take a note..."
          />

          <div v-else class="note-items">
            <div v-for="(it, idx) in form.items" :key="idx" class="note-item-row">
              <input v-model="it.checked" type="checkbox" />
              <app-input v-model="it.text" placeholder="List item" />
              <button class="chip" type="button" @click="removeItem(idx)">Remove</button>
            </div>
            <app-button size="sm" variant="neutral" @click="addItem">Add item</app-button>
          </div>

          <div class="form-row-2">
            <app-select v-model="form.color" label="Color" :options="colorOptions" />
            <div class="check-group">
              <label class="check-label"><input v-model="form.pinned" type="checkbox" /> <span>Pinned</span></label>
              <label class="check-label"><input v-model="form.archived" type="checkbox" /> <span>Archived</span></label>
            </div>
          </div>

          <app-select v-model="form.label_ids" label="Labels" :options="labelOptions" multiple />

          <div v-if="formError" class="form-error">{{ formError }}</div>

          <div class="form-actions">
            <app-button v-if="editingTrashed" variant="neutral" @click="restoreEditing">Restore</app-button>
            <app-button v-else-if="editingId" variant="neutral" @click="trashEditing">Move to trash</app-button>
            <app-button v-if="editingTrashed" variant="danger" @click="askDelete">Delete forever</app-button>
            <app-button variant="primary" @click="saveNote">Save</app-button>
          </div>
        </div>
      </app-modal>

      <app-modal
        :open="pendingDelete !== null"
        title="Delete note"
        :subtitle="pendingDelete ? pendingDelete.title : ''"
        :sheet="false"
        @close="pendingDelete = null"
      >
        <p class="confirm-text">
          This permanently removes the note and its checklist items. It cannot be undone.
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
import { addOutline, createOutline, documentTextOutline, pin, pinOutline } from 'ionicons/icons'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppModal from '@/components/ui/AppModal.vue'
import SyncChip from '@/components/ui/SyncChip.vue'
import { request, ApiError } from '@/api/client'

const colors = ['DEFAULT', 'WHITE', 'RED', 'ORANGE', 'YELLOW', 'GREEN', 'TEAL', 'BLUE', 'DARKBLUE', 'PURPLE', 'PINK', 'BROWN', 'GRAY']

const FILTERS = [
  { key: 'all', label: 'All' },
  { key: 'pinned', label: 'Pinned' },
  { key: 'archived', label: 'Archived' },
  { key: 'trashed', label: 'Trash' }
] as const

type NoteFilter = (typeof FILTERS)[number]['key']

const notes = ref<any[]>([])
const labels = ref<any[]>([])
const search = ref('')
const filter = ref<NoteFilter>('all')
const colorFilter = ref('')
const labelFilter = ref('')
const editorOpen = ref(false)
const editingId = ref<string | null>(null)
const editingTrashed = ref(false)
const pendingDelete = ref<{ id: string; title: string } | null>(null)
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

const setFilter = (f: NoteFilter) => {
  filter.value = f
  loadNotes()
}

const colorOptions = colors.map((c) => ({ value: c, label: c }))
const labelOptions = computed(() => labels.value.map((l: any) => ({ value: l.id, label: l.name })))

const hasActiveQuery = computed(
  () => !!(search.value || colorFilter.value || labelFilter.value || filter.value !== 'all')
)

function clearFilters() {
  search.value = ''
  colorFilter.value = ''
  labelFilter.value = ''
  filter.value = 'all'
  loadNotes()
}

const noteCardClass = (note: any) => {
  const c = note.color
  if (c && c !== 'DEFAULT') return `notecard note-${String(c).toLowerCase()}`
  return 'notecard'
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
  editingTrashed.value = false
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
  editingTrashed.value = !!note.trashed_at
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

async function togglePin(note: any) {
  await request(`/api/notes/${note.id}`, { method: 'PUT', body: { pinned: note.pinned ? 0 : 1 } })
  await loadNotes()
}

async function trashEditing() {
  if (!editingId.value) return
  await request(`/api/notes/${editingId.value}/trash`, { method: 'PUT', body: {} })
  closeEditor()
  await loadNotes()
}

async function restoreEditing() {
  if (!editingId.value) return
  await request(`/api/notes/${editingId.value}`, { method: 'PUT', body: { trashed_at: '' } })
  closeEditor()
  await loadNotes()
}

function askDelete() {
  const note = notes.value.find((n: any) => n.id === editingId.value)
  if (!note) return
  pendingDelete.value = { id: note.id, title: note.title || '(Untitled)' }
}

async function doDelete() {
  const target = pendingDelete.value
  if (!target) return
  await request(`/api/notes/${target.id}`, { method: 'DELETE' })
  pendingDelete.value = null
  closeEditor()
  await loadNotes()
}

onMounted(async () => {
  await loadLabels()
  await loadNotes()
})
</script>

<style scoped>
/* ---- list ---- */
.notes-grid {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
}

/* ---- note card ---- */
.note-card {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
}
.note-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--density-gap);
}
.note-title {
  margin: 0;
  font-size: calc(0.95rem * var(--density-font-scale));
  font-weight: 650;
}
.note-body {
  margin: 0;
  white-space: pre-wrap;
  font-size: calc(0.86rem * var(--density-font-scale));
}
.note-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
}
.note-check {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  font-size: calc(0.86rem * var(--density-font-scale));
}
.note-check-done {
  color: var(--text-faint);
}
.note-check-done span {
  text-decoration: line-through;
}
.note-labels {
  flex-wrap: wrap;
}
.note-foot {
  display: flex;
  align-items: center;
  margin-top: 0;
}
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--density-gap);
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  font-size: calc(1.1rem * var(--density-font-scale));
  line-height: 1;
  cursor: pointer;
}
.icon-btn:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}
.icon-btn-on {
  color: var(--accent);
}

/* ---- editor ---- */
.note-textarea {
  width: 100%;
  min-height: 120px;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  background: var(--surface-2);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: calc(0.9rem * var(--density-font-scale));
  line-height: var(--density-leading);
  resize: vertical;
}
.note-textarea:focus-visible {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.note-items {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
}
.note-item-row {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
}
.check-group {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
  padding-top: var(--density-gap);
}
.check-label {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
}

/* ---- delete confirmation ---- */
.confirm-text {
  margin: 0;
  font-size: calc(0.9rem * var(--density-font-scale));
  line-height: var(--density-leading);
  color: var(--text-muted);
}

/* ---- note colours ---- */
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
