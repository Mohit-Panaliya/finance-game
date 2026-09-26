<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'

const props = withDefaults(
  defineProps<{
    text: string
    x?: number
    y?: number
    color?: string
    size?: number
    duration?: number
  }>(),
  { x: 50, y: 50, color: '#f5c542', size: 30, duration: 1400 }
)

const emit = defineEmits<{ done: [] }>()
const alive = ref(true)
const timer = window.setTimeout(() => {
  alive.value = false
  emit('done')
}, props.duration)

onBeforeUnmount(() => window.clearTimeout(timer))
</script>

<template>
  <span
    v-if="alive"
    class="fnum"
    :style="{
      left: `${x}px`,
      top: `${y}px`,
      color,
      fontSize: `${size}px`,
      textShadow: `0 2px 0 #1a0f00, 0 0 14px ${color}`
    }"
  >
    {{ text }}
  </span>
</template>

<style scoped>
.fnum {
  position: absolute;
  z-index: 50;
  pointer-events: none;
  font-family: var(--ff-font);
  font-weight: 800;
  white-space: nowrap;
  animation: float-up 1.4s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
}
</style>
