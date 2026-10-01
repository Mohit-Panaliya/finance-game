<script setup lang="ts">
import { computed, ref } from 'vue'

export interface BarDatum {
  label: string
  value: number
  color?: string
}

const props = withDefaults(
  defineProps<{
    data: BarDatum[]
    height?: number
    showValues?: boolean
    formatValue?: (n: number) => string
    emphasisIndex?: number
  }>(),
  { height: 200, showValues: false, formatValue: undefined, emphasisIndex: -1 }
)

const emit = defineEmits<{ select: [index: number, label: string] }>()

const W = 320
const PAD_L = 40
const PAD_R = 8
const PAD_T = 14
const PAD_B = 26
const GRID_LINES = 4
const MIN_BAR_W = 3

const active = ref(-1)

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

interface Row {
  label: string
  value: number
  color: string
}

const rows = computed<Row[]>(() => {
  const list = Array.isArray(props.data) ? props.data : []
  return list.map((d, i) => ({
    label: String(d?.label ?? ''),
    value: Number.isFinite(d?.value) ? d.value : 0,
    color:
      typeof d?.color === 'string' && d.color
        ? d.color
        : i === props.emphasisIndex
          ? 'var(--accent)'
          : 'var(--chart-c1)'
  }))
})

const isEmpty = computed(() => rows.value.length === 0)

const bounds = computed(() => {
  let min = 0
  let max = 0
  for (const r of rows.value) {
    if (r.value < min) min = r.value
    if (r.value > max) max = r.value
  }
  if (max === min) max = min === 0 ? 1 : min + Math.abs(min) * 0.5
  return { min, max }
})

const plotW = computed(() => W - PAD_L - PAD_R)
const plotH = computed(() => Math.max(20, props.height - PAD_T - PAD_B))

function yOf(v: number): number {
  const { min, max } = bounds.value
  const span = max - min || 1
  return PAD_T + plotH.value - ((v - min) / span) * plotH.value
}

const zeroY = computed(() => yOf(0))

const gridlines = computed(() => {
  const { min, max } = bounds.value
  const out: Array<{ y: number; label: string }> = []
  for (let i = 0; i <= GRID_LINES; i++) {
    const v = max - ((max - min) / GRID_LINES) * i
    out.push({ y: PAD_T + (plotH.value / GRID_LINES) * i, label: fmt.value(v) })
  }
  return out
})

interface Bar {
  index: number
  label: string
  value: number
  x: number
  y: number
  w: number
  h: number
  color: string
  capTop: number
  capBottom: number
  text: string
}

const bars = computed<Bar[]>(() => {
  const n = rows.value.length
  if (!n) return []
  const slot = plotW.value / n
  const w = Math.max(MIN_BAR_W, Math.min(30, slot * 0.62))
  const radius = Math.min(4, w / 2)
  return rows.value.map((r, i) => {
    const vTop = yOf(Math.max(0, r.value))
    const vBottom = yOf(Math.min(0, r.value))
    const top = Math.min(vTop, vBottom)
    const h = Math.max(1, Math.abs(vBottom - vTop))
    return {
      index: i,
      label: r.label,
      value: r.value,
      x: PAD_L + slot * i + (slot - w) / 2,
      y: top,
      w,
      h,
      color: r.color,
      capTop: r.value >= 0 ? radius : 0,
      capBottom: r.value < 0 ? radius : 0,
      text: fmt.value(r.value)
    }
  })
})

const labelStep = computed(() => Math.max(1, Math.ceil(rows.value.length / 6)))

const xLabels = computed(() => {
  const slot = plotW.value / Math.max(1, rows.value.length)
  const maxChars = Math.max(2, Math.floor(slot / 5.2))
  return bars.value
    .filter((b) => b.index % labelStep.value === 0)
    .map((b) => {
      const raw = b.label
      const text = raw.length > maxChars ? `${raw.slice(0, maxChars - 1)}…` : raw
      return {
        index: b.index,
        text,
        x: b.x + b.w / 2,
        y: PAD_T + plotH.value + 13
      }
    })
})

const valueLabels = computed(() =>
  bars.value
    .filter((b) => props.showValues || active.value === b.index)
    .map((b) => ({
      index: b.index,
      x: b.x + b.w / 2,
      y: b.value >= 0 ? b.y - 4 : b.y + b.h + 9,
      text: b.text
    }))
)

const ariaLabel = computed(() => {
  if (isEmpty.value) return 'Bar chart, no data'
  return `Bar chart: ${rows.value.map((r) => `${r.label} ${fmt.value(r.value)}`).join(', ')}`
})
</script>

<template>
  <div class="bar-chart">
    <p v-if="isEmpty" class="empty-text">No data to chart yet.</p>
    <svg
      v-else
      class="chart"
      :viewBox="`0 0 ${W} ${height}`"
      preserveAspectRatio="xMidYMid meet"
      role="img"
      :aria-label="ariaLabel"
    >
      <g>
        <template v-for="(g, i) in gridlines" :key="`g-${i}`">
          <line class="chart-grid" :x1="PAD_L" :y1="g.y" :x2="W - PAD_R" :y2="g.y" />
          <text class="chart-axis-text" :x="PAD_L - 6" :y="g.y + 3" text-anchor="end">{{ g.label }}</text>
        </template>
        <line
          v-if="bounds.min < 0"
          class="chart-zero"
          :x1="PAD_L"
          :y1="zeroY"
          :x2="W - PAD_R"
          :y2="zeroY"
        />
      </g>

      <g v-for="b in bars" :key="`b-${b.index}`">
        <path
          class="bar-mark"
          :class="{ 'is-active': active === b.index }"
          :d="`M${b.x},${b.y + b.h} L${b.x},${b.y + b.capTop} Q${b.x},${b.y} ${b.x + b.capTop},${b.y} L${b.x + b.w - b.capTop},${b.y} Q${b.x + b.w},${b.y} ${b.x + b.w},${b.y + b.capTop} L${b.x + b.w},${b.y + b.h} Q${b.x + b.w},${b.y + b.h} ${b.x + b.w - b.capBottom},${b.y + b.h} L${b.x + b.capBottom},${b.y + b.h} Q${b.x},${b.y + b.h} ${b.x},${b.y + b.h} Z`"
          :fill="b.color"
          @pointerenter="active = b.index"
          @pointerleave="active = -1"
          @click="emit('select', b.index, b.label)"
        >
          <title>{{ b.label }} · {{ b.text }}</title>
        </path>
        <rect
          class="bar-hit"
          :x="b.x - 4"
          :y="PAD_T - 6"
          :width="b.w + 8"
          :height="plotH + 12"
          fill="transparent"
          @pointerenter="active = b.index"
          @pointerleave="active = -1"
          @click="emit('select', b.index, b.label)"
        />
      </g>

      <text
        v-for="v in valueLabels"
        :key="`v-${v.index}`"
        class="chart-value-text"
        :x="v.x"
        :y="v.y"
        text-anchor="middle"
      >
        {{ v.text }}
      </text>

      <text
        v-for="l in xLabels"
        :key="`x-${l.index}`"
        class="chart-axis-text"
        :x="l.x"
        :y="l.y"
        text-anchor="middle"
      >
        {{ l.text }}
      </text>
    </svg>
  </div>
</template>

<style scoped>
.bar-chart {
  --chart-c1: var(--success);
  width: 100%;
}

.chart-zero {
  stroke: var(--border-strong);
  stroke-width: 1;
  stroke-dasharray: 3 3;
}

.bar-mark {
  cursor: pointer;
  transition: opacity 0.16s ease;
  opacity: 0.92;
}

.bar-mark.is-active {
  opacity: 1;
}

.bar-hit {
  cursor: pointer;
}

.chart-value-text {
  fill: var(--text);
  font-size: 8px;
  font-weight: 600;
  font-family: var(--font-body);
  pointer-events: none;
}
</style>
