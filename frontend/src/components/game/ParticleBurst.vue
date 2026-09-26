<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount } from 'vue'

const props = withDefaults(
  defineProps<{
    active: boolean
    count?: number
    colors?: string[]
    originX?: string
    originY?: string
  }>(),
  {
    active: false,
    count: 22,
    colors: () => ['#f5c542', '#fff6d0', '#4caf50', '#3498db', '#9b59b6', '#e74c3c'],
    originX: '50%',
    originY: '50%'
  }
)

interface P {
  id: number
  px: number
  py: number
  size: number
  color: string
  delay: number
  radius: string
}

const runId = ref(0)
const parts = ref<P[]>([])
let clearTimer = 0

function spawn() {
  const list: P[] = []
  for (let i = 0; i < props.count; i++) {
    const angle = (Math.PI * 2 * i) / props.count + Math.random() * 0.5
    const dist = 60 + Math.random() * 110
    list.push({
      id: i,
      px: Math.cos(angle) * dist,
      py: Math.sin(angle) * dist,
      size: 6 + Math.random() * 10,
      color: props.colors[i % props.colors.length],
      delay: Math.random() * 0.12,
      radius: Math.random() > 0.5 ? '50%' : '2px'
    })
  }
  parts.value = list
  runId.value++
  window.clearTimeout(clearTimer)
  clearTimer = window.setTimeout(() => {
    parts.value = []
  }, 1100)
}

watch(
  () => props.active,
  (v) => {
    if (v) spawn()
  }
)

onBeforeUnmount(() => window.clearTimeout(clearTimer))

const style = computed(() => ({ left: props.originX, top: props.originY }))
</script>

<template>
  <div v-if="parts.length" class="pb-root" :style="style">
    <span
      v-for="p in parts"
      :key="`${runId}-${p.id}`"
      class="pb-p"
      :style="{
        width: `${p.size}px`,
        height: `${p.size}px`,
        background: p.color,
        borderRadius: p.radius,
        animationDelay: `${p.delay}s`,
        boxShadow: `0 0 8px ${p.color}`,
        '--px': `${p.px}px`,
        '--py': `${p.py}px`
      }"
    />
  </div>
</template>

<style scoped>
.pb-root {
  position: absolute;
  z-index: 60;
  pointer-events: none;
  width: 0;
  height: 0;
}
.pb-p {
  position: absolute;
  left: 0;
  top: 0;
  animation: particle-fly 0.95s cubic-bezier(0.22, 1.2, 0.36, 1) forwards;
}
</style>
