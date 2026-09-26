<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    variant?: 'gold' | 'green' | 'red' | 'blue' | 'wood'
    disabled?: boolean
    sparkle?: boolean
    size?: 'sm' | 'md' | 'lg'
    block?: boolean
    type?: 'button' | 'submit'
  }>(),
  { variant: 'gold', size: 'md', type: 'button', disabled: false, sparkle: false, block: false }
)

const emit = defineEmits<{ click: [e: MouseEvent] }>()

const classes = computed(() => [
  'gbtn',
  `gbtn-${props.variant}`,
  `gbtn-${props.size}`,
  { 'gbtn-block': props.block, 'gbtn-disabled': props.disabled, 'gbtn-sparkle': props.sparkle }
])

function onClick(e: MouseEvent) {
  if (props.disabled) {
    e.preventDefault()
    return
  }
  emit('click', e)
}
</script>

<template>
  <button :type="type" :class="classes" :disabled="disabled" @click="onClick">
    <span class="gbtn-shine" aria-hidden="true" />
    <span v-if="sparkle" class="gbtn-sparks" aria-hidden="true">
      <i v-for="n in 6" :key="n" class="gbtn-spark" :style="{ animationDelay: `${n * 0.18}s` }" />
    </span>
    <span class="gbtn-face">
      <slot />
    </span>
  </button>
</template>

<style scoped>
.gbtn {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 4px solid var(--ff-brown-deep);
  border-radius: 16px;
  font-family: var(--ff-font);
  font-weight: 800;
  letter-spacing: 0.05em;
  color: #2a1a0a;
  cursor: pointer;
  overflow: visible;
  transform: translateY(0);
  transition:
    transform 0.15s var(--ff-bounce),
    box-shadow 0.15s ease,
    filter 0.2s ease;
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.45);
  -webkit-user-select: none;
  user-select: none;
  z-index: 1;
}
.gbtn:active:not(.gbtn-disabled) {
  transform: translateY(5px);
}
.gbtn-face {
  position: relative;
  z-index: 2;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 0 4px;
}
.gbtn-shine {
  position: absolute;
  top: 3px;
  left: 6px;
  right: 6px;
  height: 42%;
  border-radius: 10px 10px 40% 40%;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.05));
  pointer-events: none;
}
.gbtn-sm {
  min-height: 38px;
  padding: 0 14px;
  font-size: 13px;
  border-width: 3px;
  border-radius: 12px;
}
.gbtn-md {
  min-height: 50px;
  padding: 0 22px;
  font-size: 17px;
}
.gbtn-lg {
  min-height: 62px;
  padding: 0 30px;
  font-size: 21px;
}
.gbtn-block {
  display: flex;
  width: 100%;
}
.gbtn-gold {
  background: linear-gradient(180deg, #ffe27a 0%, #f5c542 45%, #d9a012 100%);
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.65),
    inset 0 -5px 0 rgba(138, 90, 0, 0.45),
    0 6px 0 #8a5a00,
    0 10px 18px rgba(0, 0, 0, 0.45);
}
.gbtn-gold:active:not(.gbtn-disabled) {
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.5),
    0 2px 0 #8a5a00,
    0 4px 10px rgba(0, 0, 0, 0.4);
}
.gbtn-green {
  background: linear-gradient(180deg, #8ee08f 0%, #4caf50 45%, #2e7d32 100%);
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.55),
    inset 0 -5px 0 rgba(20, 80, 25, 0.4),
    0 6px 0 #1b5e20,
    0 10px 18px rgba(0, 0, 0, 0.45);
}
.gbtn-green:active:not(.gbtn-disabled) {
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.45),
    0 2px 0 #1b5e20,
    0 4px 10px rgba(0, 0, 0, 0.4);
}
.gbtn-red {
  background: linear-gradient(180deg, #ff9d94 0%, #e74c3c 45%, #922b21 100%);
  color: #2a0a05;
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.5),
    inset 0 -5px 0 rgba(90, 20, 12, 0.45),
    0 6px 0 #7a241b,
    0 10px 18px rgba(0, 0, 0, 0.45);
}
.gbtn-red:active:not(.gbtn-disabled) {
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.4),
    0 2px 0 #7a241b,
    0 4px 10px rgba(0, 0, 0, 0.4);
}
.gbtn-blue {
  background: linear-gradient(180deg, #8fd0f7 0%, #3498db 45%, #1a5276 100%);
  color: #08202e;
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.55),
    inset 0 -5px 0 rgba(10, 50, 80, 0.45),
    0 6px 0 #154360,
    0 10px 18px rgba(0, 0, 0, 0.45);
}
.gbtn-blue:active:not(.gbtn-disabled) {
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.45),
    0 2px 0 #154360,
    0 4px 10px rgba(0, 0, 0, 0.4);
}
.gbtn-wood {
  background:
    repeating-linear-gradient(90deg, rgba(0, 0, 0, 0.1) 0 2px, transparent 2px 18px),
    linear-gradient(180deg, #8a5a2b 0%, #5c3817 50%, #3a220d 100%);
  color: var(--ff-parchment);
  text-shadow: 0 2px 0 #1a0f00;
  box-shadow:
    inset 0 2px 0 rgba(255, 210, 140, 0.35),
    0 6px 0 #241405,
    0 10px 18px rgba(0, 0, 0, 0.5);
}
.gbtn-disabled {
  filter: grayscale(0.85) brightness(0.65);
  cursor: not-allowed;
  box-shadow:
    inset 0 3px 8px rgba(0, 0, 0, 0.5),
    0 3px 0 #1a0f00;
  transform: translateY(3px);
}
.gbtn-sparks {
  position: absolute;
  inset: -14px;
  pointer-events: none;
}
.gbtn-spark {
  position: absolute;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: radial-gradient(circle, #fff6d0, var(--ff-gold));
  top: 50%;
  left: 50%;
  opacity: 0;
  animation: sparkle 1.1s ease-in-out infinite;
  box-shadow: 0 0 8px var(--ff-gold);
}
.gbtn-spark:nth-child(1) { transform: translate(-70px, -22px); }
.gbtn-spark:nth-child(2) { transform: translate(64px, -18px); }
.gbtn-spark:nth-child(3) { transform: translate(-40px, 24px); }
.gbtn-spark:nth-child(4) { transform: translate(44px, 26px); }
.gbtn-spark:nth-child(5) { transform: translate(0, -32px); }
.gbtn-spark:nth-child(6) { transform: translate(0, 30px); }
</style>
