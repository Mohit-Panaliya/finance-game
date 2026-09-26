<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import type { Resources } from '@/types'

const props = withDefaults(
  defineProps<{ resources?: Partial<Resources>; compact?: boolean }>(),
  { resources: () => ({}), compact: false }
)

interface Slot {
  key: keyof Resources
  icon: string
  target: number
  shown: number
  raf: number | null
}

const slots = ref<Slot[]>([
  { key: 'gold', icon: '🪙', target: 0, shown: 0, raf: null },
  { key: 'elixir', icon: '🧪', target: 0, shown: 0, raf: null },
  { key: 'gems', icon: '💎', target: 0, shown: 0, raf: null },
  { key: 'trophies', icon: '🏆', target: 0, shown: 0, raf: null }
])

const flyingCoins = ref<{ id: number; slot: number }[]>([])
let coinId = 0

function animateSlot(slot: Slot) {
  if (slot.raf) cancelAnimationFrame(slot.raf)
  const start = performance.now()
  const from = slot.shown
  const delta = slot.target - from
  if (delta === 0) return
  const dur = Math.min(900, 350 + Math.abs(delta) * 2)
  const step = (now: number) => {
    const t = Math.min(1, (now - start) / dur)
    const eased = 1 - Math.pow(1 - t, 3)
    slot.shown = Math.round(from + delta * eased)
    if (t < 1) slot.raf = requestAnimationFrame(step)
    else slot.raf = null
  }
  slot.raf = requestAnimationFrame(step)
}

watch(
  () => props.resources,
  (res) => {
    for (const slot of slots.value) {
      const next = Number(res?.[slot.key] ?? 0)
      if (next > slot.target) {
        if (slot.target > 0 || slot.shown > 0) {
          const id = coinId++
          flyingCoins.value.push({ id, slot: slots.value.indexOf(slot) })
          window.setTimeout(() => {
            flyingCoins.value = flyingCoins.value.filter((c) => c.id !== id)
          }, 900)
        }
      }
      slot.target = next
      animateSlot(slot)
    }
  },
  { deep: true, immediate: true }
)

function fmt(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 10_000) return `${(n / 1000).toFixed(1)}K`
  return String(Math.round(n))
}

onMounted(() => {
  for (const slot of slots.value) {
    slot.target = Number(props.resources?.[slot.key] ?? 0)
    slot.shown = slot.target
  }
})
onBeforeUnmount(() => {
  for (const slot of slots.value) if (slot.raf) cancelAnimationFrame(slot.raf)
})
</script>

<template>
  <div class="rbar" :class="{ 'rbar-compact': compact }">
    <div v-for="(slot, i) in slots" :key="slot.key" class="rbar-slot">
      <span class="rbar-icon" :class="{ bob: slot.key === 'gold' }">{{ slot.icon }}</span>
      <span class="rbar-count carved carved-sm" :class="`rbar-count-${slot.key}`">
        {{ fmt(slot.shown) }}
      </span>
      <transition-group name="coin">
        <span
          v-for="c in flyingCoins.filter((f) => f.slot === i)"
          :key="c.id"
          class="rbar-coin"
          :style="{ animationDelay: `${(c.id % 3) * 0.08}s` }"
        >
          {{ slot.key === 'gems' ? '💎' : slot.key === 'trophies' ? '⭐' : slot.key === 'elixir' ? '✨' : '🪙' }}
        </span>
      </transition-group>
    </div>
    <div class="rbar-extra">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.rbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px calc(8px + env(safe-area-inset-top) * 0);
  width: 100%;
}
.rbar-slot {
  position: relative;
  display: flex;
  align-items: center;
  gap: 4px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.08), transparent 45%),
    linear-gradient(180deg, #241405, #140b02);
  border: 3px solid #0a0500;
  border-radius: 999px;
  padding: 4px 10px 4px 6px;
  min-width: 74px;
  justify-content: center;
  box-shadow:
    inset 0 2px 0 rgba(255, 220, 150, 0.18),
    0 3px 0 rgba(0, 0, 0, 0.6);
  flex: 1;
}
.rbar-compact .rbar-slot {
  min-width: 58px;
  padding: 3px 7px 3px 5px;
}
.rbar-icon {
  font-size: 15px;
  line-height: 1;
  filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.7));
}
.rbar-count {
  font-size: 14px;
  letter-spacing: 0.03em;
}
.rbar-count-gold { color: var(--ff-gold); }
.rbar-count-elixir { color: #e8a0ff; }
.rbar-count-gems { color: #d38cff; }
.rbar-count-trophies { color: #ffd76a; }
.rbar-coin {
  position: absolute;
  top: -4px;
  left: 50%;
  font-size: 15px;
  pointer-events: none;
  animation: coin-fly 0.85s ease-out forwards;
}
.rbar-extra {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
</style>
