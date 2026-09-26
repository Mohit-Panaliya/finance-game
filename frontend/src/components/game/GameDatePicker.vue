<script setup lang="ts">
import { computed, ref, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue?: string
    open?: boolean
    min?: number
    max?: number
    title?: string
  }>(),
  {
    modelValue: '',
    open: false,
    min: 1950,
    max: () => new Date().getFullYear() + 5,
    title: 'Choose Date'
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'update:open': [value: boolean]
  confirm: [value: string]
  cancel: []
}>()

const ITEM_H = 46
const VISIBLE = 5

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']

function parse(v: string): { y: number; m: number; d: number } {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(v || '')
  if (match) {
    return { y: Number(match[1]), m: Number(match[2]), d: Number(match[3]) }
  }
  const now = new Date()
  return { y: now.getFullYear(), m: now.getMonth() + 1, d: now.getDate() }
}

const initial = parse(props.modelValue)
const yearIdx = ref(0)
const monthIdx = ref(initial.m - 1)
const dayIdx = ref(initial.d - 1)
const dragging = ref<'y' | 'm' | 'd' | null>(null)

const years = computed<number[]>(() => {
  const out: number[] = []
  for (let y = props.min; y <= props.max; y++) out.push(y)
  return out
})
const daysInMonth = computed(() => {
  const y = years.value[yearIdx.value] ?? props.max
  return new Date(y, monthIdx.value + 1, 0).getDate()
})
const dayLabels = computed(() => Array.from({ length: daysInMonth.value }, (_, i) => i + 1))

const currentYear = computed(() => years.value[yearIdx.value] ?? props.min)
const value = computed(() => {
  const y = currentYear.value
  const m = String(monthIdx.value + 1).padStart(2, '0')
  const d = String(Math.min(dayIdx.value + 1, daysInMonth.value)).padStart(2, '0')
  return `${y}-${m}-${d}`
})

function syncFromModel() {
  if (!props.modelValue) return
  const p = parse(props.modelValue)
  const yi = years.value.indexOf(p.y)
  if (yi >= 0) yearIdx.value = yi
  monthIdx.value = Math.min(11, Math.max(0, p.m - 1))
  dayIdx.value = Math.min(daysInMonth.value - 1, Math.max(0, p.d - 1))
}

watch(
  () => props.modelValue,
  () => syncFromModel()
)
watch(
  () => props.open,
  (v) => {
    if (v) {
      syncFromModel()
      document.body.style.overflow = 'hidden'
    } else {
      document.body.style.overflow = ''
    }
  },
  { immediate: true }
)
watch([dayIdx, monthIdx, yearIdx], () => {
  if (dayIdx.value > daysInMonth.value - 1) dayIdx.value = daysInMonth.value - 1
  emit('update:modelValue', value.value)
})
watch(value, (v) => {
  if (v !== props.modelValue) emit('update:modelValue', v)
})

function clampIdx(i: number, len: number): number {
  return Math.min(len - 1, Math.max(0, i))
}

function offsetFor(index: number): number {
  return -index * ITEM_H + Math.floor(VISIBLE / 2) * ITEM_H
}

/* ---- wheel spin via touch drag ---- */
type Col = 'y' | 'm' | 'd'
const touch = ref({ col: null as Col | null, startY: 0, startIdx: 0 })
const dragOffset = ref({ y: 0, m: 0, d: 0 })

function colLen(col: Col): number {
  if (col === 'y') return years.value.length
  if (col === 'm') return 12
  return daysInMonth.value
}
function colIdx(col: Col): number {
  return col === 'y' ? yearIdx.value : col === 'm' ? monthIdx.value : dayIdx.value
}
function setColIdx(col: Col, i: number) {
  const v = clampIdx(i, colLen(col))
  if (col === 'y') yearIdx.value = v
  else if (col === 'm') monthIdx.value = v
  else dayIdx.value = v
}

function onTouchStart(col: Col, e: TouchEvent) {
  dragging.value = col
  touch.value = { col, startY: e.touches[0].clientY, startIdx: colIdx(col) }
  dragOffset.value[col] = 0
}
function onTouchMove(e: TouchEvent) {
  if (!touch.value.col) return
  const col = touch.value.col
  const dy = touch.value.startY - e.touches[0].clientY
  dragOffset.value[col] = dy
  const next = touch.value.startIdx + Math.round(dy / ITEM_H)
  if (next !== colIdx(col)) setColIdx(col, next)
}
function onTouchEnd() {
  if (!touch.value.col) return
  const col = touch.value.col
  const dy = dragOffset.value[col]
  if (Math.abs(dy) > ITEM_H / 2) {
    const steps = Math.round(dy / ITEM_H)
    setColIdx(col, touch.value.startIdx + steps)
  }
  dragOffset.value[col] = 0
  touch.value.col = null
  dragging.value = null
}

function nudge(col: Col, delta: number) {
  setColIdx(col, colIdx(col) + delta)
}
function pickIndex(col: Col, i: number) {
  setColIdx(col, i)
}
function onWheel(col: Col, e: WheelEvent) {
  e.preventDefault()
  nudge(col, e.deltaY > 0 ? 1 : -1)
}

function confirm() {
  emit('update:modelValue', value.value)
  emit('confirm', value.value)
  emit('update:open', false)
}
function cancel() {
  syncFromModel()
  emit('cancel')
  emit('update:open', false)
}

function styleFor(col: Col): Record<string, string> {
  const idx = colIdx(col)
  const y = offsetFor(idx) + (dragging.value === col ? 0 : 0)
  return {
    height: `${ITEM_H * VISIBLE}px`,
    '--wheel-y': `${y}px`
  }
}
</script>

<template>
  <teleport to="body">
    <transition name="gdp-fade">
      <div v-if="open" class="gdp-overlay" @click.self="cancel">
        <transition name="gdp-pop" appear>
          <div class="gdp-frame ornate panel-wood noise">
            <div class="gdp-header">
              <span class="gdp-gem" />
              <h3 class="gdp-title carved carved-gold">{{ title }}</h3>
              <span class="gdp-gem" />
            </div>

            <div class="gdp-wheels">
              <div
                v-for="col in (['d', 'm', 'y'] as Col[])"
                :key="col"
                class="gdp-col"
                @wheel="onWheel(col, $event)"
              >
                <button class="gdp-nudge" type="button" aria-label="Previous" @click="nudge(col, -1)">▲</button>
                <div
                  class="gdp-window"
                  :class="{ 'gdp-dragging': dragging === col }"
                  :style="styleFor(col)"
                  @touchstart.prevent="onTouchStart(col, $event)"
                  @touchmove.prevent="onTouchMove"
                  @touchend.prevent="onTouchEnd"
                  @touchcancel.prevent="onTouchEnd"
                >
                  <div
                    class="gdp-track"
                    :style="{
                      transform: `translateY(${offsetFor(colIdx(col)) + (dragging === col ? dragOffset[col] : 0)}px)`
                    }"
                  >
                    <div
                      v-for="(item, i) in
                        col === 'y' ? years : col === 'm' ? MONTHS : dayLabels"
                      :key="String(item)"
                      class="gdp-item"
                      :class="{
                        'gdp-item-active': i === colIdx(col),
                        'gdp-item-near': Math.abs(i - colIdx(col)) === 1
                      }"
                      :style="{ height: `${ITEM_H}px` }"
                      @click="pickIndex(col, i)"
                    >
                      {{ col === 'm' ? item : col === 'd' ? String(item).padStart(2, '0') : item }}
                    </div>
                  </div>
                  <div class="gdp-band" aria-hidden="true" />
                </div>
                <button class="gdp-nudge" type="button" aria-label="Next" @click="nudge(col, 1)">▼</button>
              </div>
            </div>

            <div class="gdp-value carved carved-gold">{{ value }}</div>

            <div class="gdp-actions">
              <button class="gdp-chest gdp-chest-cancel" type="button" @click="cancel">
                <span class="gdp-chest-lid" />
                <span class="gdp-chest-label">Cancel</span>
              </button>
              <button class="gdp-chest gdp-chest-confirm" type="button" @click="confirm">
                <span class="gdp-chest-lid" />
                <span class="gdp-chest-label">Confirm</span>
              </button>
            </div>
          </div>
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<style scoped>
.gdp-overlay {
  position: fixed;
  inset: 0;
  z-index: 10005;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px;
  background:
    radial-gradient(circle at 50% 20%, rgba(120, 80, 20, 0.35), transparent 60%),
    radial-gradient(circle at 50% 50%, rgba(60, 35, 10, 0.85), rgba(6, 3, 0, 0.95));
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}
.gdp-frame {
  width: min(400px, 100%);
  padding: 16px 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.gdp-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
}
.gdp-title {
  margin: 0;
  font-size: 19px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
.gdp-gem {
  width: 14px;
  height: 14px;
  background: radial-gradient(circle at 35% 30%, #fff, var(--ff-purple) 60%, #5b2c82);
  transform: rotate(45deg);
  border: 2px solid var(--ff-brown-deep);
  box-shadow: 0 0 8px rgba(155, 89, 182, 0.9);
}
.gdp-wheels {
  display: flex;
  gap: 8px;
  justify-content: center;
}
.gdp-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  flex: 1;
  min-width: 0;
}
.gdp-nudge {
  width: 100%;
  height: 30px;
  border: 3px solid var(--ff-brown-deep);
  border-radius: 9px;
  background: linear-gradient(180deg, #ffe27a, var(--ff-gold) 50%, var(--ff-gold-dark));
  color: var(--ff-brown-deep);
  font-size: 11px;
  font-weight: 900;
  cursor: pointer;
  box-shadow: 0 3px 0 #5c3c00;
  transition: transform 0.15s var(--ff-bounce);
  line-height: 1;
  padding: 0;
}
.gdp-nudge:active {
  transform: translateY(3px);
  box-shadow: 0 0 0 #5c3c00;
}
.gdp-window {
  position: relative;
  width: 100%;
  overflow: hidden;
  border: 3px solid #120a00;
  border-radius: 12px;
  background:
    linear-gradient(180deg, rgba(0, 0, 0, 0.65), rgba(0, 0, 0, 0.2) 35%, rgba(0, 0, 0, 0.2) 65%, rgba(0, 0, 0, 0.65)),
    linear-gradient(180deg, #2c1a08, #1c1004);
  box-shadow:
    inset 0 4px 10px rgba(0, 0, 0, 0.8),
    inset 0 -4px 10px rgba(0, 0, 0, 0.8);
  touch-action: none;
}
.gdp-band {
  position: absolute;
  left: 0;
  right: 0;
  top: calc(50% - 23px);
  height: 46px;
  pointer-events: none;
  border-top: 2px solid rgba(245, 197, 66, 0.75);
  border-bottom: 2px solid rgba(245, 197, 66, 0.75);
  background: linear-gradient(180deg, rgba(245, 197, 66, 0.14), rgba(245, 197, 66, 0.05));
  box-shadow: 0 0 14px rgba(245, 197, 66, 0.35);
  border-radius: 6px;
}
.gdp-track {
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  transition: transform 0.42s cubic-bezier(0.22, 1.4, 0.36, 1);
  will-change: transform;
}
.gdp-dragging .gdp-track {
  transition: none;
}
.gdp-item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  color: rgba(242, 226, 194, 0.4);
  font-family: var(--ff-font);
  font-size: 15px;
  letter-spacing: 0.04em;
  cursor: pointer;
  transition:
    color 0.25s ease,
    transform 0.25s var(--ff-bounce),
    text-shadow 0.25s ease;
}
.gdp-item-near {
  color: rgba(242, 226, 194, 0.7);
  font-size: 17px;
}
.gdp-item-active {
  color: var(--ff-gold);
  font-size: 22px;
  text-shadow:
    0 1px 0 #8a5a00,
    0 0 14px rgba(245, 197, 66, 0.8);
  transform: scale(1.06);
}
.gdp-value {
  text-align: center;
  font-size: 16px;
  letter-spacing: 0.14em;
}
.gdp-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}
.gdp-chest {
  position: relative;
  flex: 1;
  min-height: 56px;
  border: 4px solid var(--ff-brown-deep);
  border-radius: 14px;
  cursor: pointer;
  font-family: var(--ff-font);
  font-size: 16px;
  overflow: hidden;
  transition: transform 0.18s var(--ff-bounce);
  padding-top: 6px;
}
.gdp-chest:active {
  transform: translateY(5px);
}
.gdp-chest-lid {
  position: absolute;
  top: 4px;
  left: 8px;
  right: 8px;
  height: 38%;
  border-radius: 8px 8px 40% 40%;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.45), rgba(255, 255, 255, 0.04));
  pointer-events: none;
}
.gdp-chest-label {
  position: relative;
  z-index: 1;
  text-shadow: 0 2px 0 rgba(0, 0, 0, 0.35);
}
.gdp-chest-confirm {
  background: linear-gradient(180deg, #8ee08f, var(--ff-green) 50%, var(--ff-green-dark));
  color: #10300f;
  box-shadow: 0 6px 0 var(--ff-green-dark), 0 10px 16px rgba(0, 0, 0, 0.45);
}
.gdp-chest-cancel {
  background: linear-gradient(180deg, #c9b8a0, #8d7b62 50%, #5c4e3c);
  color: #241405;
  box-shadow: 0 6px 0 #3d3226, 0 10px 16px rgba(0, 0, 0, 0.45);
}
.gdp-fade-enter-active,
.gdp-fade-leave-active {
  transition: opacity 0.25s ease;
}
.gdp-fade-enter-from,
.gdp-fade-leave-to {
  opacity: 0;
}
.gdp-pop-enter-active {
  animation: bounce-in 0.5s var(--ff-bounce) both;
}
.gdp-pop-leave-active {
  animation: gdp-pop-out 0.2s ease both;
}
@keyframes gdp-pop-out {
  to {
    transform: scale(0.7) translateY(30px);
    opacity: 0;
  }
}
</style>
