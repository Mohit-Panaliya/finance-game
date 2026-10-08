<script setup lang="ts">
import { computed, ref } from 'vue'

export interface LineSeries {
  name: string
  color?: string
  values: number[]
}

const props = withDefaults(
  defineProps<{
    series: LineSeries[]
    labels?: string[]
    height?: number
    formatValue?: (n: number) => string
    zeroBaseline?: boolean
  }>(),
  { labels: () => [], height: 220, formatValue: undefined, zeroBaseline: true }
)

const emit = defineEmits<{ select: [index: number] }>()

const W = 320
const PAD_L = 40
const PAD_R = 10
const PAD_T = 12
const PAD_B = 24
const GRID_LINES = 4
const MAX_LABEL_ROWS = 6

const active = ref(-1)
const uid = `lc-${Math.random().toString(36).slice(2, 9)}`

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
  name: string
  color: string
  values: number[]
}

const rows = computed<Row[]>(() => {
  const list = Array.isArray(props.series) ? props.series : []
  return list.map((s, i) => ({
    name: String(s?.name ?? ''),
    color: typeof s?.color === 'string' && s.color ? s.color : `var(--chart-c${i % 8})`,
    values: (Array.isArray(s?.values) ? s.values : []).map((v) => (Number.isFinite(v) ? v : 0))
  }))
})

const pointCount = computed(() => rows.value.reduce((m, r) => Math.max(m, r.values.length), 0))
const isEmpty = computed(() => pointCount.value === 0)

const bounds = computed(() => {
  let min = 0
  let max = 0
  for (const r of rows.value) {
    for (const v of r.values) {
      if (v < min) min = v
      if (v > max) max = v
    }
  }
  if (max === min) max = min === 0 ? 1 : min + Math.abs(min) * 0.5
  return { min, max }
})

const plotW = computed(() => W - PAD_L - PAD_R)
const plotH = computed(() => Math.max(20, props.height - PAD_T - PAD_B))

function xOf(i: number): number {
  if (pointCount.value <= 1) return PAD_L + plotW.value / 2
  return PAD_L + (plotW.value / (pointCount.value - 1)) * i
}

function yOf(v: number): number {
  const { min, max } = bounds.value
  const span = max - min || 1
  return PAD_T + plotH.value - ((v - min) / span) * plotH.value
}

const zeroY = computed(() => yOf(0))
const showZero = computed(() => props.zeroBaseline && bounds.value.min < 0)

const gridlines = computed(() => {
  const { min, max } = bounds.value
  const out: Array<{ y: number; label: string }> = []
  for (let i = 0; i <= GRID_LINES; i++) {
    const v = max - ((max - min) / GRID_LINES) * i
    out.push({ y: PAD_T + (plotH.value / GRID_LINES) * i, label: fmt.value(v) })
  }
  return out
})

function lineFor(values: number[]): string {
  return values.map((v, i) => `${i === 0 ? 'M' : 'L'}${xOf(i).toFixed(2)},${yOf(v).toFixed(2)}`).join(' ')
}

const drawn = computed(() =>
  rows.value
    .filter((r) => r.values.length > 0)
    .map((r, i) => ({
      name: r.name,
      color: r.color,
      path: lineFor(r.values.slice(0, pointCount.value)),
      area: i === 0 ? areaFor(r.values.slice(0, pointCount.value)) : '',
      dots: r.values
        .slice(0, pointCount.value)
        .map((v, idx) => ({ index: idx, x: xOf(idx), y: yOf(v), color: r.color, text: `${r.name} · ${fmt.value(v)}` }))
    }))
)

function areaFor(values: number[]): string {
  if (values.length === 0) return ''
  const base = yOf(Math.max(0, bounds.value.min))
  const head = values.map((v, i) => `${i === 0 ? 'M' : 'L'}${xOf(i).toFixed(2)},${yOf(v).toFixed(2)}`).join(' ')
  const last = xOf(values.length - 1).toFixed(2)
  const first = xOf(0).toFixed(2)
  return `${head} L${last},${base.toFixed(2)} L${first},${base.toFixed(2)} Z`
}

const dots = computed(() => drawn.value.flatMap((d) => d.dots))

const xLabels = computed(() => {
  const total = pointCount.value
  if (!total) return []
  const step = Math.max(1, Math.ceil(total / MAX_LABEL_ROWS))
  const slot = total > 1 ? plotW.value / (total - 1) : plotW.value
  const maxChars = Math.max(2, Math.floor(slot / 5.2))
  const out: Array<{ index: number; x: number; y: number; text: string }> = []
  for (let i = 0; i < total; i += step) {
    const raw = String(props.labels?.[i] ?? '')
    if (!raw) continue
    out.push({
      index: i,
      x: xOf(i),
      y: PAD_T + plotH.value + 13,
      text: raw.length > maxChars ? `${raw.slice(0, maxChars - 1)}…` : raw
    })
  }
  return out
})

const hitAreas = computed(() => {
  const total = pointCount.value
  if (!total) return []
  const step = total > 1 ? plotW.value / (total - 1) : plotW.value
  const out: Array<{ index: number; x: number; y: number; w: number; h: number }> = []
  for (let i = 0; i < total; i++) {
    const cx = xOf(i)
    out.push({
      index: i,
      x: total > 1 ? cx - step / 2 : cx - plotW.value / 2,
      y: PAD_T - 6,
      w: total > 1 ? step : plotW.value,
      h: plotH.value + 12
    })
  }
  return out
})

const tipRows = computed(() => {
  const i = active.value
  if (i < 0 || !rows.value.length) return []
  return rows.value
    .filter((r) => r.values.length > i)
    .map((r) => ({ name: r.name, color: r.color, text: fmt.value(r.values[i]) }))
})

const tipLabel = computed(() => String(props.labels?.[active.value] ?? ''))
const tipX = computed(() => (active.value < 0 ? 0 : Math.min(Math.max(xOf(active.value), PAD_L + 22), W - PAD_R - 22)))
const tipY = computed(() => {
  const i = active.value
  if (i < 0) return PAD_T
  let top = Number.POSITIVE_INFINITY
  for (const r of rows.value) {
    if (r.values.length > i) top = Math.min(top, yOf(r.values[i]))
  }
  return Number.isFinite(top) ? Math.max(PAD_T + 12, top - 14) : PAD_T + 12
})

const ariaLabel = computed(() => {
  if (isEmpty.value) return 'Line chart, no data'
  return `Line chart: ${rows.value.map((r) => `${r.name} ${r.values.length} points`).join(', ')}`
})
</script>

<template>
  <div class="line-chart">
    <p v-if="isEmpty" class="empty-text">No trend data yet.</p>
    <template v-else>
      <svg
        class="chart"
        :viewBox="`0 0 ${W} ${height}`"
        preserveAspectRatio="xMidYMid meet"
        role="img"
        :aria-label="ariaLabel"
      >
        <defs>
          <linearGradient :id="`${uid}-fill`" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--accent)" stop-opacity="0.28" />
            <stop offset="100%" stop-color="var(--accent)" stop-opacity="0" />
          </linearGradient>
        </defs>

        <g>
          <template v-for="(g, i) in gridlines" :key="`g-${i}`">
            <line class="chart-grid" :x1="PAD_L" :y1="g.y" :x2="W - PAD_R" :y2="g.y" />
            <text class="chart-axis-text" :x="PAD_L - 6" :y="g.y + 3" text-anchor="end">{{ g.label }}</text>
          </template>
          <line v-if="showZero" class="chart-zero" :x1="PAD_L" :y1="zeroY" :x2="W - PAD_R" :y2="zeroY" />
        </g>

        <path v-if="drawn[0]" class="line-area" :d="drawn[0].area" :fill="`url(#${uid}-fill)`" />

        <path
          v-for="(d, i) in drawn"
          :key="`l-${i}`"
          class="line-path"
          :d="d.path"
          :stroke="d.color"
        >
          <title>{{ d.name }}</title>
        </path>

        <circle
          v-for="d in dots"
          :key="`d-${d.index}-${d.color}${d.y}`"
          class="line-dot"
          :class="{ 'is-active': active === d.index }"
          :cx="d.x"
          :cy="d.y"
          :r="active === d.index ? 3.4 : 2.2"
          :fill="d.color"
        >
          <title>{{ d.text }}</title>
        </circle>

        <g v-if="active >= 0 && tipRows.length" class="line-tip" :transform="`translate(${tipX}, ${tipY})`">
          <text class="line-tip-title" x="0" y="-2" text-anchor="middle">{{ tipLabel }}</text>
          <text v-for="(r, i) in tipRows" :key="`t-${i}`" class="line-tip-text" x="0" :y="10 + i * 10" text-anchor="middle">
            {{ r.name }} {{ r.text }}
          </text>
        </g>

        <rect
          v-for="h in hitAreas"
          :key="`h-${h.index}`"
          class="line-hit"
          :x="h.x"
          :y="h.y"
          :width="h.w"
          :height="h.h"
          fill="transparent"
          @pointerenter="active = h.index"
          @pointerleave="active = -1"
          @click="emit('select', h.index)"
        >
          <title>{{ labels?.[h.index] ?? `Point ${h.index + 1}` }}</title>
        </rect>

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

      <ul class="legend line-legend">
        <li v-for="(d, i) in drawn" :key="`lg-${i}`" class="legend-item">
          <span class="legend-dot" :style="{ background: d.color }" />
          {{ d.name }}
        </li>
      </ul>
    </template>
  </div>
</template>

<style scoped>
.line-chart {
  --chart-c0: var(--accent);
  --chart-c1: var(--success);
  --chart-c2: var(--warning);
  --chart-c3: color-mix(in oklab, var(--accent) 55%, var(--warning));
  --chart-c4: color-mix(in oklab, var(--success) 55%, var(--accent));
  --chart-c5: color-mix(in oklab, var(--warning) 50%, var(--danger));
  --chart-c6: color-mix(in oklab, var(--accent) 60%, var(--success));
  --chart-c7: color-mix(in oklab, var(--danger) 50%, var(--accent));
  width: 100%;
  position: relative;
}

.chart-zero {
  stroke: var(--border-strong);
  stroke-width: 1;
  stroke-dasharray: 3 3;
}

.line-area {
  pointer-events: none;
}

.line-path {
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.line-dot {
  stroke: var(--surface);
  stroke-width: 1.2;
  transition: r 0.15s ease;
}

.line-hit {
  cursor: pointer;
}

.line-tip {
  pointer-events: none;
}

.line-tip-title {
  font-variant-numeric: tabular-nums;
  fill: var(--text);
  font-size: 8.5px;
  font-weight: 700;
}

.line-tip-text {
  font-variant-numeric: tabular-nums;
  fill: var(--text-muted);
  font-size: 8px;
}

.line-legend {
  list-style: none;
  padding: 0;
  margin-bottom: 0;
}
</style>
