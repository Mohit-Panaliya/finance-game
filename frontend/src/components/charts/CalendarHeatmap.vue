<script setup lang="ts">
import { computed, ref } from 'vue'

export interface HeatmapDay {
  date: string
  value: number
}

const props = withDefaults(
  defineProps<{
    days: HeatmapDay[]
    weeks?: number
    formatValue?: (n: number) => string
    emptyText?: string
  }>(),
  { weeks: 26, formatValue: undefined, emptyText: 'No activity recorded yet.' }
)

const emit = defineEmits<{ select: [date: string] }>()

const CELL = 12
const GAP = 3
const PAD_L = 26
const PAD_T = 16
const ROWS = 7
const STEPS = 4
const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
const WEEKDAYS = ['', 'Mon', '', 'Wed', '', 'Fri', '']

const active = ref('')

function defaultFormat(n: number): string {
  if (!Number.isFinite(n)) return '0'
  const a = Math.abs(n)
  const sign = n < 0 ? '-' : ''
  if (a >= 1e7) return `${sign}${Math.round((a / 1e7) * 10) / 10}Cr`
  if (a >= 1e5) return `${sign}${Math.round((a / 1e5) * 10) / 10}L`
  if (a >= 1e3) return `${sign}${Math.round((a / 1e3) * 10) / 10}K`
  return `${sign}${Math.round(a * 100) / 100}`
}

const fmt = computed(() => {
  const f: ((n: number) => string) | undefined = props.formatValue
  return (n: number): string => {
    if (!Number.isFinite(n)) return '0'
    if (!f) return defaultFormat(n)
    try {
      const out = f(n)
      return typeof out === 'string' && out ? out : defaultFormat(n)
    } catch {
      return defaultFormat(n)
    }
  }
})

function parseDate(iso: string): number | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(String(iso ?? '').trim())
  if (!m) return null
  const y = Number(m[1])
  const mo = Number(m[2])
  const d = Number(m[3])
  if (mo < 1 || mo > 12 || d < 1 || d > 31) return null
  const t = Date.UTC(y, mo - 1, d)
  const check = new Date(t)
  if (check.getUTCFullYear() !== y || check.getUTCMonth() !== mo - 1 || check.getUTCDate() !== d) return null
  return t
}

function isoOf(t: number): string {
  const d = new Date(t)
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getUTCFullYear()}-${p(d.getUTCMonth() + 1)}-${p(d.getUTCDate())}`
}

const DAY_MS = 86400000

const values = computed(() => {
  const list = Array.isArray(props.days) ? props.days : []
  const map = new Map<string, number>()
  for (const d of list) {
    const t = parseDate(d?.date)
    if (t === null) continue
    const v = Number.isFinite(d?.value) ? d.value : 0
    map.set(isoOf(t), (map.get(isoOf(t)) ?? 0) + v)
  }
  return map
})

const weekCount = computed(() => {
  const w = Number.isFinite(props.weeks) ? Math.round(props.weeks) : 26
  return Math.min(53, Math.max(4, w))
})

const maxValue = computed(() => {
  let m = 0
  for (const v of values.value.values()) if (v > m) m = v
  return m
})

const isEmpty = computed(() => values.value.size === 0 || maxValue.value <= 0)

function levelOf(v: number): number {
  if (maxValue.value <= 0 || v <= 0) return 0
  const ratio = v / maxValue.value
  return Math.min(STEPS, Math.max(1, Math.ceil(ratio * STEPS)))
}

interface Cell {
  iso: string
  x: number
  y: number
  level: number
  value: number
  label: string
  has: boolean
}

const columns = computed<Cell[][]>(() => {
  const total = weekCount.value
  const keys = [...values.value.keys()]
  let endT = keys.length ? Math.max(...keys.map((k) => parseDate(k) ?? 0)) : Date.now()
  if (!Number.isFinite(endT) || endT <= 0) endT = Date.now()
  const weekStart = endT - new Date(endT).getUTCDay() * DAY_MS
  const start = weekStart - (total - 1) * ROWS * DAY_MS

  const out: Cell[][] = []
  for (let w = 0; w < total; w++) {
    const col: Cell[] = []
    for (let r = 0; r < ROWS; r++) {
      const t = start + (w * ROWS + r) * DAY_MS
      const iso = isoOf(t)
      const has = values.value.has(iso)
      const v = values.value.get(iso) ?? 0
      col.push({
        iso,
        x: PAD_L + w * (CELL + GAP),
        y: PAD_T + r * (CELL + GAP),
        level: levelOf(v),
        value: v,
        has,
        label: has ? `${iso} · ${fmt.value(v)}` : `${iso} · no activity`
      })
    }
    out.push(col)
  }
  return out
})

const cells = computed(() => columns.value.flat())

const monthLabels = computed(() => {
  const out: Array<{ x: number; y: number; text: string }> = []
  let last = -1
  for (const col of columns.value) {
    const first = col[0]
    if (!first) continue
    const m = new Date(first.iso).getUTCMonth()
    if (m !== last) {
      out.push({ x: first.x + CELL / 2, y: PAD_T - 5, text: MONTHS[m] ?? '' })
      last = m
    }
  }
  return out
})

const gridW = computed(() => PAD_L + weekCount.value * (CELL + GAP))
const gridH = computed(() => PAD_T + ROWS * (CELL + GAP))
const activeCell = computed(() => cells.value.find((c) => c.iso === active.value) ?? null)
const lastCell = computed(() => cells.value[cells.value.length - 1] ?? null)
const ariaLabel = computed(() =>
  isEmpty.value ? 'Activity calendar, no data' : `Activity calendar, ${weekCount.value} weeks to ${cells.value[cells.value.length - 1]?.iso ?? ''}`
)
</script>

<template>
  <div class="heatmap">
    <p v-if="isEmpty" class="empty-text">{{ emptyText }}</p>
    <template v-else>
      <svg
        class="chart heatmap-svg"
        :viewBox="`0 0 ${gridW} ${gridH}`"
        preserveAspectRatio="xMidYMid meet"
        role="img"
        :aria-label="ariaLabel"
      >
        <text
          v-for="(w, i) in WEEKDAYS"
          :key="`wd-${i}`"
          class="chart-axis-text"
          :x="PAD_L - 5"
          :y="PAD_T + i * (CELL + GAP) + CELL / 2 + 3"
          text-anchor="end"
        >
          {{ w }}
        </text>

        <text
          v-for="(m, i) in monthLabels"
          :key="`ml-${i}`"
          class="chart-axis-text"
          :x="m.x"
          :y="m.y"
          text-anchor="middle"
        >
          {{ m.text }}
        </text>

        <rect
          v-for="c in cells"
          :key="c.iso"
          class="heat-cell"
          :class="[`heat-${c.level}`, { 'is-empty': !c.has, 'is-active': active === c.iso }]"
          :x="c.x"
          :y="c.y"
          :width="CELL"
          :height="CELL"
          rx="2.5"
          @pointerenter="active = c.iso"
          @pointerleave="active = ''"
          @click="emit('select', c.iso)"
        >
          <title>{{ c.label }}</title>
        </rect>
      </svg>

      <div class="heat-foot">
        <span class="heat-readout">{{ activeCell ? activeCell.label : lastCell ? lastCell.iso : '' }}</span>
        <span class="heat-scale">
          Less
          <span class="heat-key heat-0" />
          <span class="heat-key heat-1" />
          <span class="heat-key heat-2" />
          <span class="heat-key heat-3" />
          <span class="heat-key heat-4" />
          More
        </span>
      </div>
    </template>
  </div>
</template>

<style scoped>
.heatmap {
  width: 100%;
}

.heatmap-svg {
  width: 100%;
  height: auto;
  overflow: visible;
}

.heat-cell {
  cursor: pointer;
  transition: opacity 0.14s ease, stroke 0.14s ease;
  stroke: transparent;
  stroke-width: 1;
}

.heat-0 {
  --heat-fill: var(--surface-3);
}

.heat-1 {
  --heat-fill: color-mix(in oklab, var(--accent) 30%, var(--surface-3));
}

.heat-2 {
  --heat-fill: color-mix(in oklab, var(--accent) 55%, var(--surface-3));
}

.heat-3 {
  --heat-fill: color-mix(in oklab, var(--accent) 78%, var(--surface-3));
}

.heat-4 {
  --heat-fill: var(--accent);
}

.heat-cell {
  fill: var(--heat-fill);
}

.heat-cell.is-empty {
  fill: var(--surface-2);
  opacity: 0.55;
}

.heat-cell.is-active {
  stroke: var(--text);
  opacity: 1;
}

.heat-key {
  width: 10px;
  height: 10px;
  border-radius: 2.5px;
  display: inline-block;
  background: var(--heat-fill);
}

.heat-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
  margin-top: 10px;
  font-size: 0.74rem;
  color: var(--text-muted);
}

.heat-readout {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.heat-scale {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  color: var(--text-faint);
}
</style>
