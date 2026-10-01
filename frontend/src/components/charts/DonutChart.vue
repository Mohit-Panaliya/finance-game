<script setup lang="ts">
import { computed, ref } from 'vue'

export interface DonutSlice {
  label: string
  value: number
  color?: string
}

const props = withDefaults(
  defineProps<{
    slices: DonutSlice[]
    height?: number
    thickness?: number
    centerValue?: string
    centerLabel?: string
    emptyText?: string
  }>(),
  {
    height: 180,
    thickness: 18,
    centerValue: '',
    centerLabel: '',
    emptyText: 'Nothing to break down yet.'
  }
)

const emit = defineEmits<{ select: [label: string] }>()

const VB = 200
const CX = VB / 2
const SEG_GAP = 2

const active = ref(-1)

function clamp(n: number, min: number, max: number): number {
  if (!Number.isFinite(n)) return min
  return Math.min(max, Math.max(min, n))
}

function trim(n: number): string {
  return String(Math.round(n * 10) / 10)
}

function fmt(n: number): string {
  if (!Number.isFinite(n)) return '0'
  const a = Math.abs(n)
  const sign = n < 0 ? '-' : ''
  if (a >= 1e7) return `${sign}${trim(a / 1e7)}Cr`
  if (a >= 1e5) return `${sign}${trim(a / 1e5)}L`
  if (a >= 1e3) return `${sign}${trim(a / 1e3)}K`
  return `${sign}${Math.round(a * 100) / 100}`
}

function fmtPct(p: number): string {
  if (!Number.isFinite(p)) return '0%'
  return `${Math.round(p * 10) / 10}%`
}

const rows = computed<DonutSlice[]>(() => {
  const list = Array.isArray(props.slices) ? props.slices : []
  return list.map((s, i) => ({
    label: String(s?.label ?? ''),
    value: Number.isFinite(s?.value) ? Math.max(0, s.value) : 0,
    color: typeof s?.color === 'string' && s.color ? s.color : `var(--chart-c${i % 8})`
  }))
})

const total = computed(() => rows.value.reduce((acc, r) => acc + r.value, 0))
const isEmpty = computed(() => total.value <= 0)

const thicknessPx = computed(() => clamp(props.thickness, 4, VB / 2 - 14))
const radius = computed(() => Math.max(8, (VB - thicknessPx.value) / 2 - 4))
const circumference = computed(() => 2 * Math.PI * radius.value)

const segments = computed(() => {
  if (isEmpty.value) return []
  const sum = total.value
  const c = circumference.value
  const positives = rows.value.filter((r) => r.value > 0)
  const gap = positives.length === 1 ? 0 : SEG_GAP
  let acc = 0
  return positives.map((s) => {
    const frac = s.value / sum
    const raw = frac * c
    const len = Math.max(0, raw - Math.min(gap, raw * 0.4))
    const seg = {
      ...s,
      frac,
      pct: frac * 100,
      dash: `${len} ${Math.max(0, c - len)}`,
      offset: -acc
    }
    acc += raw
    return seg
  })
})

const ariaLabel = computed(() => {
  if (isEmpty.value) return props.emptyText
  return `Breakdown: ${segments.value.map((s) => `${s.label} ${fmtPct(s.pct)}`).join(', ')}`
})
</script>

<template>
  <div class="donut-chart">
    <p v-if="isEmpty" class="empty-text">{{ emptyText }}</p>
    <template v-else>
      <svg
        class="chart donut-svg"
        :style="{ maxWidth: `${height}px` }"
        :viewBox="`0 0 ${VB} ${VB}`"
        preserveAspectRatio="xMidYMid meet"
        role="img"
        :aria-label="ariaLabel"
      >
        <circle class="donut-track" :cx="CX" :cy="CX" :r="radius" :stroke-width="thicknessPx" fill="none" />
        <g :transform="`rotate(-90 ${CX} ${CX})`">
          <circle
            v-for="(s, i) in segments"
            :key="`${s.label}-${i}`"
            class="donut-slice"
            :class="{ 'is-active': active === i }"
            :cx="CX"
            :cy="CX"
            :r="radius"
            :stroke-width="thicknessPx"
            :stroke-dasharray="s.dash"
            :stroke-dashoffset="s.offset"
            :style="{ stroke: s.color }"
            fill="none"
            @pointerenter="active = i"
            @pointerleave="active = -1"
            @click="emit('select', s.label)"
          >
            <title>{{ s.label }} · {{ fmt(s.value) }} ({{ fmtPct(s.pct) }})</title>
          </circle>
        </g>
        <text v-if="centerValue" class="donut-center-value" :x="CX" :y="centerLabel ? CX - 4 : CX + 2" text-anchor="middle">
          {{ centerValue }}
        </text>
        <text v-if="centerLabel" class="donut-center-label" :x="CX" :y="CX + 16" text-anchor="middle">{{ centerLabel }}</text>
      </svg>

      <ul class="legend donut-legend">
        <li
          v-for="(s, i) in segments"
          :key="`lg-${s.label}-${i}`"
          class="legend-item"
          :class="{ 'is-active': active === i }"
        >
          <span class="legend-dot" :style="{ background: s.color }" />
          <span class="legend-name">{{ s.label }}</span>
          <span class="legend-value">{{ fmt(s.value) }}</span>
          <span class="legend-pct">{{ fmtPct(s.pct) }}</span>
        </li>
      </ul>
    </template>
  </div>
</template>

<style scoped>
.donut-chart {
  --chart-c0: var(--accent);
  --chart-c1: var(--success);
  --chart-c2: var(--warning);
  --chart-c3: color-mix(in oklab, var(--accent) 55%, var(--warning));
  --chart-c4: color-mix(in oklab, var(--success) 55%, var(--accent));
  --chart-c5: color-mix(in oklab, var(--warning) 50%, var(--danger));
  --chart-c6: color-mix(in oklab, var(--accent) 60%, var(--success));
  --chart-c7: color-mix(in oklab, var(--danger) 50%, var(--accent));
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.donut-svg {
  width: 100%;
  height: auto;
}

.donut-track {
  stroke: var(--surface-3);
}

.donut-slice {
  cursor: pointer;
  transition: opacity 0.18s ease, filter 0.18s ease;
}

.donut-slice:hover,
.donut-slice.is-active {
  opacity: 1;
  filter: brightness(1.15) saturate(1.05);
}

.donut-center-value {
  fill: var(--text);
  font-size: 26px;
  font-weight: 700;
  font-family: var(--font-body);
}

.donut-center-label {
  fill: var(--text-muted);
  font-size: 11px;
  font-family: var(--font-body);
}

.donut-legend {
  align-self: stretch;
  flex-direction: column;
  gap: 6px;
  margin-top: 12px;
  list-style: none;
  margin-bottom: 0;
  padding: 0;
}

.donut-legend .legend-item {
  gap: 8px;
}

.legend-item.is-active .legend-name {
  color: var(--text);
}

.legend-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.legend-value {
  font-variant-numeric: tabular-nums;
  color: var(--text);
  font-weight: 600;
}

.legend-pct {
  font-variant-numeric: tabular-nums;
  color: var(--text-faint);
  min-width: 42px;
  text-align: right;
}
</style>
