<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { IonIcon } from '@ionic/vue'
import { calendarOutline, chevronBackOutline, chevronForwardOutline } from 'ionicons/icons'
import { formatDate, parseDateParts, todayISO } from '@/utils/date'

const props = withDefaults(
  defineProps<{
    modelValue?: string | null
    label?: string
    placeholder?: string
    disabled?: boolean
    clearable?: boolean
  }>(),
  { modelValue: null, placeholder: 'Select date', disabled: false, clearable: true, label: '' }
)

const emit = defineEmits<{ 'update:modelValue': [value: string | null] }>()

const open = ref(false)

const MONTH_NAMES = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December']
const WEEKDAYS = ['Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa', 'Su']

const today = todayISO()
const selected = computed(() => (parseDateParts(props.modelValue) ? String(props.modelValue).slice(0, 10) : null))

const initialParts = parseDateParts(props.modelValue) ?? parseDateParts(today)!
const cursor = ref<{ y: number; m: number }>({ y: initialParts.y, m: initialParts.m })

watch(open, (isOpen) => {
  if (isOpen) {
    const parts = parseDateParts(props.modelValue) ?? parseDateParts(today)!
    cursor.value = { y: parts.y, m: parts.m }
  }
})

function pad(n: number): string {
  return String(n).padStart(2, '0')
}

const monthLabel = computed(() => `${MONTH_NAMES[Math.min(11, Math.max(0, cursor.value.m - 1))]} ${cursor.value.y}`)

const cells = computed(() => {
  const { y, m } = cursor.value
  const first = new Date(Date.UTC(y, m - 1, 1))
  const lead = (first.getUTCDay() + 6) % 7
  const days = new Date(Date.UTC(y, m, 0)).getUTCDate()
  const out: Array<string | null> = Array.from({ length: lead }, () => null)
  for (let d = 1; d <= days; d++) out.push(`${y}-${pad(m)}-${pad(d)}`)
  while (out.length % 7 !== 0) out.push(null)
  return out
})

const todayIso = today

function shift(delta: number) {
  let m = cursor.value.m + delta
  let y = cursor.value.y
  while (m < 1) {
    m += 12
    y -= 1
  }
  while (m > 12) {
    m -= 12
    y += 1
  }
  cursor.value = { y, m }
}

function pick(iso: string) {
  emit('update:modelValue', iso)
  open.value = false
}

function clear() {
  emit('update:modelValue', null)
  open.value = false
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && open.value) open.value = false
}
</script>

<template>
  <div class="adp" :class="{ 'adp-disabled': disabled }">
    <span v-if="label" class="adp-label">{{ label }}</span>
    <div class="adp-shell">
      <button class="adp-trigger" type="button" :disabled="disabled" @click="open = true">
        <ion-icon class="adp-icon" :icon="calendarOutline" />
        <span class="adp-text clamp-1">{{ selected ? formatDate(selected) : placeholder }}</span>
      </button>
      <button v-if="clearable && selected" class="adp-clear" type="button" aria-label="Clear date" @click="clear">
        &times;
      </button>
    </div>

    <teleport to="body">
      <transition name="adp-fade">
        <div v-if="open" class="adp-overlay" @click.self="open = false" @keydown="onKey">
          <transition name="adp-pop" appear>
            <div class="adp-panel">
              <div class="adp-head">
                <button class="adp-nav" type="button" aria-label="Previous month" @click="shift(-1)">
                  <ion-icon :icon="chevronBackOutline" />
                </button>
                <span class="adp-title">{{ monthLabel }}</span>
                <button class="adp-nav" type="button" aria-label="Next month" @click="shift(1)">
                  <ion-icon :icon="chevronForwardOutline" />
                </button>
              </div>

              <div class="adp-week">
                <span v-for="d in WEEKDAYS" :key="d" class="adp-weekday">{{ d }}</span>
              </div>

              <div class="adp-grid">
                <button
                  v-for="(iso, i) in cells"
                  :key="i"
                  class="adp-day"
                  :class="{
                    'adp-day-empty': !iso,
                    'adp-day-selected': iso === selected,
                    'adp-day-today': iso === todayIso
                  }"
                  type="button"
                  :disabled="!iso"
                  @click="iso && pick(iso)"
                >
                  {{ iso ? Number(iso.slice(8)) : '' }}
                </button>
              </div>

              <div class="adp-foot">
                <button class="adp-foot-btn" type="button" @click="pick(todayIso)">Today</button>
                <button class="adp-foot-btn" type="button" @click="open = false">Cancel</button>
              </div>
            </div>
          </transition>
        </div>
      </transition>
    </teleport>
  </div>
</template>

<style scoped>
.adp {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
  width: 100%;
}
.adp-label {
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.adp-shell {
  display: flex;
  align-items: stretch;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}
.adp-trigger {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  min-height: 44px;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  background: transparent;
  border: none;
  color: var(--text);
  font-size: 0.95rem;
  cursor: pointer;
  text-align: left;
}
.adp-icon {
  font-size: 17px;
  color: var(--accent);
  flex-shrink: 0;
}
.adp-text {
  flex: 1;
}
.adp-clear {
  width: 34px;
  background: transparent;
  border: none;
  border-left: 1px solid var(--border);
  color: var(--text-muted);
  font-size: 20px;
  cursor: pointer;
}
.adp-disabled {
  opacity: 0.55;
  pointer-events: none;
}
.adp-overlay {
  position: fixed;
  inset: 0;
  z-index: 10001;
  background: var(--scrim);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--density-page-pad);
}
.adp-panel {
  width: min(360px, 100%);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: var(--density-card-pad);
  box-shadow: var(--shadow-lg);
}
.adp-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--density-gap);
  margin-bottom: var(--density-gap);
}
.adp-title {
  font-size: 1rem;
  font-weight: 600;
}
.adp-nav {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text);
  font-size: 17px;
  cursor: pointer;
}
.adp-week,
.adp-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 4px;
}
.adp-weekday {
  text-align: center;
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--text-faint);
  padding-bottom: 6px;
}
.adp-day {
  aspect-ratio: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text);
  font-size: 0.86rem;
  cursor: pointer;
}
.adp-day:not(:disabled):hover {
  background: var(--surface-2);
}
.adp-day-empty {
  cursor: default;
}
.adp-day-today {
  border-color: var(--border-strong);
}
.adp-day-selected {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--on-accent);
  font-weight: 600;
}
.adp-foot {
  display: flex;
  gap: var(--density-gap);
  margin-top: var(--density-section-gap);
}
.adp-foot-btn {
  flex: 1;
  min-height: 38px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text);
  font-size: 0.88rem;
  font-weight: 600;
  cursor: pointer;
}
.adp-foot-btn:last-child {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--on-accent);
}
.adp-fade-enter-active,
.adp-fade-leave-active {
  transition: opacity 0.18s ease;
}
.adp-fade-enter-from,
.adp-fade-leave-to {
  opacity: 0;
}
.adp-pop-enter-active,
.adp-pop-leave-active {
  transition: transform 0.2s ease;
}
.adp-pop-enter-from,
.adp-pop-leave-to {
  transform: scale(0.94);
}
</style>