<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  IonPage,
  IonContent,
  IonHeader,
  IonFab,
  IonFabButton,
  IonItemSliding,
  IonItem,
  IonItemOptions,
  IonItemOption,
  onIonViewWillEnter
} from '@ionic/vue'
import GameModal from '@/components/game/GameModal.vue'
import GameButton from '@/components/game/GameButton.vue'
import GameInput from '@/components/game/GameInput.vue'
import GameSelect from '@/components/game/GameSelect.vue'
import GameDatePicker from '@/components/game/GameDatePicker.vue'
import SyncChip from '@/components/game/SyncChip.vue'
import { useFinanceStore } from '@/stores/financeStore'
import { entityConfig, rowTitle, rowValue, formatGold } from '@/entityConfig'
import type { EntityType, FinanceRow } from '@/types'

const props = defineProps<{ startNew?: boolean }>()

const route = useRoute()
const router = useRouter()
const finance = useFinanceStore()

const entity = computed<EntityType>(() => {
  const e = String(route.params.entity ?? 'banks')
  return e as EntityType
})
const cfg = computed(() => entityConfig(entity.value))
const valid = computed(() => entity.value in finance.lists)

const accentCss = computed(() => entityConfig(entity.value).accent)
const search = ref('')
let searchTimer = 0
const list = computed(() => finance.lists[entity.value])
const totalValue = computed(() => {
  if (!list.value) return 0
  return list.value.items.reduce((s, r) => s + rowValue(cfg.value, r), 0)
})

const modalOpen = ref(false)
const editingId = ref<number | string | null>(null)
const saving = ref(false)
const formError = ref('')
const form = reactive<Record<string, unknown>>({})
const pickerOpen = ref(false)
const pickerKey = ref('')
const confirmDelete = ref<FinanceRow | null>(null)

watch(search, () => {
  window.clearTimeout(searchTimer)
  searchTimer = window.setTimeout(() => {
    void finance.fetchList(entity.value, { reset: true, search: search.value })
  }, 350)
})

watch(entity, () => {
  search.value = ''
  void finance.fetchList(entity.value, { reset: true })
})

function fv(key: string): string {
  return String(form[key] ?? '')
}

function setFv(key: string, v: string | number) {
  form[key] = v
}

function resetForm() {
  for (const k of Object.keys(form)) delete form[k]
  for (const f of cfg.value.fields) {
    if (f.kind === 'select' && f.options?.length) form[f.key] = f.options[0].value
    else form[f.key] = ''
  }
  formError.value = ''
  editingId.value = null
  pickerOpen.value = false
  pickerKey.value = ''
}

function openCreate() {
  resetForm()
  modalOpen.value = true
}

function openEdit(row: FinanceRow) {
  resetForm()
  editingId.value = row.id ?? null
  for (const f of cfg.value.fields) {
    form[f.key] = row[f.key] ?? ''
  }
  modalOpen.value = true
}

function closeForm() {
  modalOpen.value = false
}

async function save() {
  const required = cfg.value.fields.filter((f) => f.kind !== 'date')
  for (const f of required) {
    const v = form[f.key]
    if (v === '' || v === undefined || v === null) {
      formError.value = `${f.label} is required`
      return
    }
  }
  const payload: FinanceRow = {}
  for (const f of cfg.value.fields) {
    let v = form[f.key]
    if ((f.kind === 'number' || f.kind === 'money') && v !== '' && v !== null && v !== undefined) {
      v = Number(v)
      if (!Number.isFinite(v)) {
        formError.value = `${f.label} must be a number`
        return
      }
    }
    payload[f.key] = v
  }
  saving.value = true
  formError.value = ''
  try {
    if (editingId.value !== null && editingId.value !== undefined) {
      await finance.update(entity.value, editingId.value, payload)
    } else {
      await finance.create(entity.value, payload)
    }
    modalOpen.value = false
    await finance.fetchList(entity.value, { reset: true })
    await finance.fetchEntitySummary(entity.value)
  } catch (e) {
    formError.value = e instanceof Error ? e.message : 'Save failed'
  } finally {
    saving.value = false
  }
}

async function doDelete() {
  const row = confirmDelete.value
  if (!row || row.id === undefined) return
  try {
    await finance.remove(entity.value, row.id)
    confirmDelete.value = null
    await finance.fetchList(entity.value, { reset: true })
  } catch {
    /* keep modal open */
  }
}

function openPicker(key: string) {
  pickerKey.value = key
  pickerOpen.value = true
}

function formattedDate(key: string): string {
  const v = String(form[key] ?? '')
  if (!v) return 'Pick date…'
  return v
}

function displayValue(row: FinanceRow): string {
  return `${formatGold(rowValue(cfg.value, row))} 🪙`
}

function fmt(n: number): string {
  return formatGold(n)
}

onMounted(() => {
  if (!valid.value) {
    void router.replace('/records')
    return
  }
  void finance.fetchList(entity.value, { reset: true })
  if (props.startNew) openCreate()
})

onIonViewWillEnter(() => {
  if (valid.value) void finance.fetchList(entity.value, { reset: true })
})
</script>

<template>
  <ion-page>
    <ion-header class="ehead">
      <div class="ehead-row">
        <button class="back-btn" type="button" @click="router.push('/records')">‹</button>
        <div class="ehead-title">
          <span class="ehead-icon">{{ cfg.icon }}</span>
          <span class="carved carved-gold ehead-name">{{ cfg.label }}</span>
        </div>
        <SyncChip />
      </div>
      <div class="ehead-search">
        <GameInput v-model="search" label="" placeholder="Search the archives…" />
      </div>
      <div class="ehead-total carved carved-sm">
        Vault total: <span class="ehead-total-v">🪙 {{ fmt(totalValue) }}</span>
        · {{ list?.items.length ?? 0 }} scrolls
      </div>
    </ion-header>

    <ion-content :fullscreen="true" class="ent-content">
      <div class="ent-scroll ff-hide-scrollbar">
        <div v-if="list?.loading && !list.items.length" class="ent-empty carved">
          Unrolling scrolls…
        </div>
        <div v-else-if="!list?.items.length" class="ent-empty">
          <div class="ent-empty-icon">📜</div>
          <p class="carved">No {{ cfg.label }} recorded yet</p>
          <GameButton variant="gold" @click="openCreate">+ ADD FIRST</GameButton>
        </div>

        <ion-item-sliding
          v-for="(row, i) in list?.items ?? []"
          :key="String(row.id ?? i)"
          class="swipe-row"
        >
          <ion-item class="row-item" lines="none" :button="false" :detail="false">
            <div
              class="row-inner stone-card"
              :style="{ animationDelay: `${Math.min(i, 10) * 45}ms`, '--accent': cfg.accent }"
              @click="openEdit(row)"
            >
              <span class="row-icon">{{ cfg.icon }}</span>
              <div class="row-mid">
                <span class="carved row-title clamp-1">{{ rowTitle(cfg, row) }}</span>
                <span class="carved carved-sm row-sub clamp-1">
                  {{ String(row.date ?? row.purchase_date ?? row.maturity_date ?? row.due_date ?? '') }}
                </span>
              </div>
              <span class="carved row-val" :style="{ color: cfg.accent }">
                {{ displayValue(row) }}
              </span>
            </div>
          </ion-item>
          <ion-item-options side="end">
            <ion-item-option class="opt opt-edit" @click="openEdit(row)">EDIT</ion-item-option>
            <ion-item-option class="opt opt-del" @click="confirmDelete = row">SLAY</ion-item-option>
          </ion-item-options>
        </ion-item-sliding>
      </div>

      <ion-fab slot="fixed" vertical="bottom" horizontal="end" class="ent-fab">
        <ion-fab-button @click="openCreate">
          <span class="fab-glyph">+</span>
        </ion-fab-button>
      </ion-fab>

      <!-- ============ CREATE / EDIT FORM ============ -->
      <GameModal
        :show="modalOpen"
        :title="editingId ? `Edit ${cfg.label}` : `New ${cfg.label.replace(/s$/, '')}`"
        @update:show="closeForm"
      >
        <div class="form-grid">
          <template v-for="f in cfg.fields" :key="f.key">
            <GameSelect
              v-if="f.kind === 'select'"
              :model-value="String(form[f.key] ?? '')"
              :label="f.label"
              :options="f.options ?? []"
              @update:model-value="setFv(f.key, $event)"
            />
            <div v-else-if="f.kind === 'date'" class="date-field">
              <span class="date-label carved carved-sm">{{ f.label }}</span>
              <button class="date-trigger" type="button" @click="openPicker(f.key)">
                <span class="date-ico">🗓️</span>
                <span class="carved" :class="{ 'date-placeholder': !form[f.key] }">
                  {{ formattedDate(f.key) }}
                </span>
                <span class="date-caret">▾</span>
              </button>
            </div>
            <GameInput
              v-else
              :model-value="String(form[f.key] ?? '')"
              :label="f.label"
              :type="f.kind === 'text' ? 'text' : 'number'"
              :placeholder="f.placeholder ?? ''"
              :suffix="f.suffix ?? ''"
              @update:model-value="setFv(f.key, $event)"
            />
          </template>
        </div>
        <p v-if="formError" class="form-error carved carved-sm">{{ formError }}</p>

        <template #footer>
          <GameButton variant="wood" size="sm" :disabled="saving" @click="closeForm">CANCEL</GameButton>
          <GameButton variant="green" size="sm" :disabled="saving" @click="save">
            {{ saving ? 'SAVING…' : 'SAVE' }}
          </GameButton>
        </template>

        <GameDatePicker
          v-if="pickerKey"
          :model-value="fv(pickerKey)"
          :open="pickerOpen"
          :title="cfg.fields.find((x) => x.key === pickerKey)?.label ?? 'Choose Date'"
          @update:model-value="form[pickerKey] = $event"
          @update:open="pickerOpen = $event"
        />
      </GameModal>

    </ion-content>

    <!-- ============ DELETE CONFIRM ============ -->
    <GameModal
      :show="!!confirmDelete"
      title="Slay this record?"
      @update:show="(v) => { if (!v) confirmDelete = null }"
    >
      <p class="confirm-text carved">
        “{{ confirmDelete ? rowTitle(cfg, confirmDelete) : '' }}” will vanish from the archives.
      </p>
      <template #footer>
        <GameButton variant="wood" size="sm" @click="confirmDelete = null">KEEP</GameButton>
        <GameButton variant="red" size="sm" @click="doDelete">SLAY IT</GameButton>
      </template>
    </GameModal>

  </ion-page>
</template>

<style scoped>
.ehead {
  position: relative;
  z-index: 20;
  background: linear-gradient(180deg, rgba(26, 15, 0, 0.97), rgba(42, 26, 10, 0.94));
  border-bottom: 3px solid rgba(245, 197, 66, 0.35);
  padding: calc(10px + env(safe-area-inset-top)) 12px 10px;
}
.ehead-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.back-btn {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  border: 3px solid #0a0500;
  background: linear-gradient(180deg, #ffe27a, var(--ff-gold) 55%, var(--ff-gold-dark));
  color: var(--ff-brown-deep);
  font-size: 26px;
  font-weight: 900;
  line-height: 1;
  cursor: pointer;
  box-shadow: 0 4px 0 #5c3c00;
  transition: transform 0.18s var(--ff-bounce);
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.back-btn:active {
  transform: translateY(4px);
  box-shadow: none;
}
.ehead-title {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.ehead-icon {
  font-size: 24px;
}
.ehead-name {
  font-size: 19px;
  letter-spacing: 0.06em;
}
.ehead-search {
  margin-top: 8px;
}
.ehead-total {
  margin-top: 6px;
  opacity: 0.75;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 11px;
}
.ehead-total-v {
  color: var(--ff-gold);
}
.ent-content {
  --background:
    radial-gradient(circle at 80% 0%, color-mix(in srgb, v-bind(accentCss) 16%, transparent), transparent 50%),
    radial-gradient(circle at 50% 130%, #4a2c12, #120a00 75%);
}
.ent-scroll {
  padding: 14px 12px calc(110px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.ent-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 54px 16px;
  text-align: center;
}
.ent-empty-icon {
  font-size: 62px;
  animation: bob 2.5s ease-in-out infinite;
  opacity: 0.85;
}
.swipe-row {
  border-radius: 14px;
}
.row-item {
  --background: transparent;
  --padding-start: 0;
  --inner-padding-end: 0;
  --min-height: 0;
}
.row-inner {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 12px 14px;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--accent) 14%, transparent), transparent 55%),
    linear-gradient(180deg, #3e2712, #241405);
  border: 3px solid #120a00;
  border-radius: 14px;
  box-shadow:
    inset 0 2px 0 rgba(255, 220, 150, 0.2),
    0 4px 0 #0d0700;
  cursor: pointer;
  animation: bounce-in 0.4s var(--ff-bounce) both;
  transition: transform 0.2s var(--ff-bounce);
}
.row-inner:active {
  transform: translateY(3px);
  box-shadow: inset 0 2px 0 rgba(255, 220, 150, 0.15);
}
.row-icon {
  font-size: 28px;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.35);
  border: 3px solid #120a00;
  border-radius: 12px;
  flex-shrink: 0;
}
.row-mid {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.row-title {
  font-size: 15px;
  color: var(--ff-parchment);
}
.row-sub {
  opacity: 0.55;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
.row-val {
  font-family: var(--ff-font);
  font-size: 15px;
  white-space: nowrap;
}
.ent-fab {
  margin-bottom: calc(76px + env(safe-area-inset-bottom));
}
.fab-glyph {
  font-size: 34px;
  font-family: var(--ff-font);
  line-height: 1;
}
.form-grid {
  display: flex;
  flex-direction: column;
  gap: 13px;
}
.date-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.date-label {
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.12em;
  color: #d8b56a;
}
.date-trigger {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  min-height: 52px;
  padding: 10px 14px;
  background: linear-gradient(180deg, #33200c, #241405);
  border: 3px solid #120a00;
  border-radius: 14px;
  color: var(--ff-parchment);
  font-family: var(--ff-font);
  font-size: 16px;
  cursor: pointer;
  box-shadow:
    inset 0 4px 10px rgba(0, 0, 0, 0.7),
    0 3px 0 rgba(0, 0, 0, 0.55);
  transition: border-color 0.25s ease, box-shadow 0.25s ease;
}
.date-trigger:active {
  border-color: var(--ff-gold);
  box-shadow:
    inset 0 4px 10px rgba(0, 0, 0, 0.7),
    0 0 16px rgba(245, 197, 66, 0.55);
}
.date-placeholder {
  opacity: 0.45;
}
.date-caret {
  margin-left: auto;
  color: var(--ff-gold);
}
.form-error {
  color: #ff9d94;
  text-align: center;
  margin: 12px 0 0;
}
.confirm-text {
  text-align: center;
  font-size: 16px;
  line-height: 1.5;
  margin: 6px 0;
}
ion-item-option.opt {
  --background: transparent;
  width: 86px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--ff-font);
  font-size: 15px;
  border: 3px solid #120a00;
  margin: 4px 2px;
  border-radius: 12px;
  color: #fff;
  text-shadow: 0 2px 0 rgba(0, 0, 0, 0.5);
}
.opt-edit {
  background: linear-gradient(180deg, #8fd0f7, var(--ff-blue) 55%, var(--ff-blue-dark)) !important;
}
.opt-del {
  background: linear-gradient(180deg, #ff9d94, var(--ff-red) 55%, var(--ff-red-dark)) !important;
}
</style>
