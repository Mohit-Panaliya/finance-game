<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import ParticleBurst from '@/components/game/ParticleBurst.vue'

const props = withDefaults(
  defineProps<{ level?: number; xp?: number; xpToNext?: number }>(),
  { level: 1, xp: 0, xpToNext: 100 }
)

const displayXp = ref(props.xp)
const burst = ref(false)
const levelUpShown = ref(false)
let raf = 0
let hideTimer = 0

const pct = computed(() => {
  const v = Math.max(0, Math.min(100, (displayXp.value / Math.max(1, props.xpToNext)) * 100))
  return v
})

watch(
  () => props.xp,
  (next) => {
    const from = displayXp.value
    const start = performance.now()
    cancelAnimationFrame(raf)
    const step = (now: number) => {
      const t = Math.min(1, (now - start) / 700)
      displayXp.value = from + (next - from) * (1 - Math.pow(1 - t, 3))
      if (t < 1) raf = requestAnimationFrame(step)
    }
    raf = requestAnimationFrame(step)
  },
  { immediate: true }
)

watch(
  () => props.level,
  (next, prev) => {
    if (prev !== undefined && next > prev) {
      burst.value = false
      levelUpShown.value = true
      requestAnimationFrame(() => {
        burst.value = true
      })
      window.clearTimeout(hideTimer)
      hideTimer = window.setTimeout(() => {
        burst.value = false
        levelUpShown.value = false
      }, 2400)
    }
  }
)
</script>

<template>
  <div class="xp">
    <div class="xp-badge" :class="{ 'xp-badge-pop': levelUpShown }">
      <span class="xp-level">{{ level }}</span>
    </div>
    <div class="xp-track">
      <div class="xp-fill" :style="{ width: `${pct}%` }">
        <div class="xp-shimmer" />
      </div>
      <span class="xp-text carved carved-sm">{{ Math.floor(displayXp) }} / {{ xpToNext }} XP</span>
      <ParticleBurst :active="burst" :count="30" origin-x="50%" origin-y="50%" />
      <transition name="lvl">
        <div v-if="levelUpShown" class="xp-levelup carved carved-gold">LEVEL UP!</div>
      </transition>
    </div>
  </div>
</template>

<style scoped>
.xp {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px 8px;
  width: 100%;
}
.xp-badge {
  position: relative;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: radial-gradient(circle at 35% 30%, #ffe27a, var(--ff-gold) 55%, var(--ff-gold-dark));
  border: 3px solid var(--ff-brown-deep);
  clip-path: polygon(50% 0%, 93% 25%, 93% 75%, 50% 100%, 7% 75%, 7% 25%);
  box-shadow: 0 4px 0 #5c3c00, 0 0 14px rgba(245, 197, 66, 0.5);
  transition: transform 0.4s var(--ff-bounce);
}
.xp-badge-pop {
  animation: xp-pop 0.6s var(--ff-bounce);
}
@keyframes xp-pop {
  0% { transform: scale(1); }
  40% { transform: scale(1.45) rotate(-8deg); }
  100% { transform: scale(1) rotate(0); }
}
.xp-level {
  font-family: var(--ff-font);
  font-size: 17px;
  color: var(--ff-brown-deep);
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.4);
  z-index: 1;
}
.xp-track {
  position: relative;
  flex: 1;
  height: 26px;
  border: 3px solid var(--ff-brown-deep);
  border-radius: 999px;
  background: linear-gradient(180deg, #1c1004, #0d0700);
  box-shadow: inset 0 3px 8px rgba(0, 0, 0, 0.8);
  overflow: visible;
}
.xp-fill {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  border-radius: 999px;
  background: linear-gradient(180deg, #8ee08f 0%, var(--ff-green) 50%, var(--ff-green-dark) 100%);
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.5),
    0 0 12px rgba(76, 175, 80, 0.6);
  transition: width 0.7s var(--ff-bounce);
  overflow: hidden;
  min-width: 6px;
}
.xp-shimmer {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    100deg,
    transparent 20%,
    rgba(255, 255, 255, 0.65) 50%,
    transparent 80%
  );
  background-size: 200% 100%;
  animation: shimmer 1.8s linear infinite;
}
.xp-text {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: #fff8e0;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.9);
  pointer-events: none;
}
.xp-levelup {
  position: absolute;
  left: 50%;
  top: -34px;
  transform: translateX(-50%);
  font-size: 22px;
  letter-spacing: 0.1em;
  pointer-events: none;
  white-space: nowrap;
}
.lvl-enter-active {
  animation: bounce-in 0.55s var(--ff-bounce) both;
}
.lvl-leave-active {
  transition: opacity 0.4s ease, transform 0.4s ease;
}
.lvl-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-20px) scale(1.3);
}
</style>
