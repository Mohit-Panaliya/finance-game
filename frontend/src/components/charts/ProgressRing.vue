<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    value: number
    size?: number
    thickness?: number
    label?: string
    sublabel?: string
    color?: string
  }>(),
  { size: 120, thickness: 10, label: '', sublabel: '', color: '' }
)

const VB = 100

const clamped = computed(() => {
  const v = Number.isFinite(props.value) ? props.value : 0
  return Math.min(100, Math.max(0, v))
})

const thicknessPx = computed(() => {
  const t = Number.isFinite(props.thickness) ? props.thickness : 10
  return Math.min(VB / 2 - 6, Math.max(2, t))
})

const radius = computed(() => (VB - thicknessPx.value) / 2 - 1)
const circumference = computed(() => 2 * Math.PI * radius.value)
const offset = computed(() => circumference.value * (1 - clamped.value / 100))

const tone = computed(() => {
  if (props.color) return props.color
  if (clamped.value >= 90) return 'var(--danger)'
  if (clamped.value >= 60) return 'var(--warning)'
  return 'var(--success)'
})

const pctText = computed(() => `${Math.round(clamped.value)}%`)
const ariaLabel = computed(() => `${props.label || 'Progress'} ${pctText.value}`)
</script>

<template>
  <div class="progress-ring" :style="{ maxWidth: `${size}px` }">
    <svg
      class="chart ring-svg"
      :viewBox="`0 0 ${VB} ${VB}`"
      preserveAspectRatio="xMidYMid meet"
      role="img"
      :aria-label="ariaLabel"
    >
      <circle class="ring-track" :cx="VB / 2" :cy="VB / 2" :r="radius" :stroke-width="thicknessPx" fill="none" />
      <circle
        class="ring-value"
        :cx="VB / 2"
        :cy="VB / 2"
        :r="radius"
        :stroke-width="thicknessPx"
        :stroke="tone"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="offset"
        fill="none"
        :transform="`rotate(-90 ${VB / 2} ${VB / 2})`"
      >
        <title>{{ label || 'Progress' }}: {{ pctText }}</title>
      </circle>
      <text class="ring-value-text" :x="VB / 2" :y="label || sublabel ? VB / 2 - 1 : VB / 2 + 2" text-anchor="middle">
        {{ pctText }}
      </text>
      <text v-if="label" class="ring-label" :x="VB / 2" :y="VB / 2 + 12" text-anchor="middle">{{ label }}</text>
      <text v-if="sublabel" class="ring-sublabel" :x="VB / 2" :y="VB / 2 + 24" text-anchor="middle">{{ sublabel }}</text>
    </svg>
  </div>
</template>

<style scoped>
.progress-ring {
  width: 100%;
  margin: 0 auto;
}

.ring-svg {
  width: 100%;
  height: auto;
  display: block;
}

.ring-track {
  stroke: var(--surface-3);
}

.ring-value {
  stroke-linecap: round;
  transition: stroke-dashoffset 0.6s cubic-bezier(0.22, 1, 0.36, 1), stroke 0.3s ease;
}

.ring-value-text {
  fill: var(--text);
  font-size: 20px;
  font-weight: 700;
  font-family: var(--font-body);
}

.ring-label {
  fill: var(--text-muted);
  font-size: 8px;
  font-family: var(--font-body);
}

.ring-sublabel {
  fill: var(--text-faint);
  font-size: 7px;
  font-family: var(--font-body);
}
</style>
