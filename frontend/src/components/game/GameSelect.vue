<script setup lang="ts">
import { computed, ref, onBeforeUnmount, watch } from 'vue'

export interface GameSelectOption {
  value: string | number
  label: string
  icon?: string
}

const props = withDefaults(
  defineProps<{
    options: GameSelectOption[]
    modelValue?: string | number | null
    placeholder?: string
    disabled?: boolean
    label?: string
  }>(),
  { modelValue: null, placeholder: 'Choose…', disabled: false, label: '' }
)

const emit = defineEmits<{ 'update:modelValue': [value: string | number]; open: []; close: [] }>()

const open = ref(false)

const selected = computed(() => props.options.find((o) => o.value === props.modelValue) ?? null)

function toggle() {
  if (props.disabled) return
  open.value = !open.value
  if (open.value) emit('open')
  else emit('close')
}

function pick(opt: GameSelectOption) {
  emit('update:modelValue', opt.value)
  open.value = false
  emit('close')
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && open.value) {
    open.value = false
    emit('close')
  }
}

watch(open, (v) => {
  if (v) document.addEventListener('keydown', onKey)
  else document.removeEventListener('keydown', onKey)
  document.body.style.overflow = v ? 'hidden' : ''
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKey)
  document.body.style.overflow = ''
})
</script>

<template>
  <div class="gsel" :class="{ 'gsel-disabled': disabled }">
    <span v-if="label" class="gsel-label carved carved-sm">{{ label }}</span>

    <button
      class="gsel-trigger"
      :class="{ 'gsel-trigger-open': open }"
      type="button"
      :disabled="disabled"
      @click="toggle"
    >
      <span class="gsel-gem" aria-hidden="true">
        <span v-if="selected?.icon">{{ selected.icon }}</span>
        <span v-else class="gsel-gem-glyph">◆</span>
      </span>
      <span class="gsel-text clamp-1">
        {{ selected ? selected.label : placeholder }}
      </span>
      <span class="gsel-chevron" :class="{ 'gsel-chevron-up': open }" aria-hidden="true">▾</span>
    </button>

    <teleport to="body">
      <transition name="gsel-fade">
        <div v-if="open" class="gsel-overlay" @click.self="open = false; emit('close')">
          <transition name="gsel-sheet" appear>
            <div class="gsel-sheet panel-wood noise">
              <div class="gsel-sheet-head">
                <span class="carved carved-gold">{{ label || 'Select' }}</span>
                <button class="gsel-x" type="button" @click="open = false; emit('close')">×</button>
              </div>
              <div class="gsel-list ff-hide-scrollbar">
                <button
                  v-for="(opt, i) in options"
                  :key="String(opt.value)"
                  class="gsel-item"
                  :class="{ 'gsel-item-selected': opt.value === modelValue }"
                  :style="{ animationDelay: `${i * 45}ms` }"
                  type="button"
                  @click="pick(opt)"
                >
                  <span v-if="opt.icon" class="gsel-item-icon">{{ opt.icon }}</span>
                  <span class="gsel-item-label">{{ opt.label }}</span>
                  <span v-if="opt.value === modelValue" class="gsel-check">✦</span>
                </button>
                <div v-if="!options.length" class="gsel-empty">No options</div>
              </div>
            </div>
          </transition>
        </div>
      </transition>
    </teleport>
  </div>
</template>

<style scoped>
.gsel {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 100%;
}
.gsel-label {
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.12em;
  color: #d8b56a;
}
.gsel-trigger {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 12px;
  min-height: 52px;
  background:
    radial-gradient(circle at 20% 25%, rgba(255, 255, 255, 0.09), transparent 55%),
    linear-gradient(180deg, #6b431f 0%, #4a2c12 60%, #331d0a 100%);
  border: 4px solid var(--ff-brown-deep);
  border-radius: 14px;
  color: var(--ff-parchment);
  font-family: var(--ff-font);
  font-size: 16px;
  cursor: pointer;
  box-shadow:
    inset 0 2px 0 rgba(255, 210, 140, 0.4),
    inset 0 -5px 0 rgba(0, 0, 0, 0.4),
    0 5px 0 #120a00,
    0 8px 16px rgba(0, 0, 0, 0.5);
  transition:
    transform 0.2s var(--ff-bounce),
    box-shadow 0.2s ease;
}
.gsel-trigger:active {
  transform: translateY(4px);
  box-shadow:
    inset 0 2px 0 rgba(255, 210, 140, 0.3),
    0 1px 0 #120a00;
}
.gsel-trigger-open {
  border-color: var(--ff-gold);
}
.gsel-gem {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: radial-gradient(circle at 35% 30%, #ffe27a, var(--ff-gold) 50%, var(--ff-gold-dark));
  border: 3px solid var(--ff-brown-deep);
  border-radius: 9px;
  transform: rotate(45deg) scale(0.82);
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.6),
    0 3px 6px rgba(0, 0, 0, 0.5);
}
.gsel-gem > span {
  transform: rotate(-45deg) scale(1.2);
  font-size: 15px;
}
.gsel-gem-glyph {
  color: var(--ff-brown-deep);
}
.gsel-text {
  flex: 1;
  text-align: left;
  text-shadow: 0 2px 0 #1a0f00;
}
.gsel-chevron {
  color: var(--ff-gold);
  font-size: 18px;
  transition: transform 0.35s var(--ff-bounce);
  text-shadow: 0 1px 0 #5c3c00;
}
.gsel-chevron-up {
  transform: rotate(180deg);
}
.gsel-disabled {
  opacity: 0.55;
  pointer-events: none;
}
.gsel-overlay {
  position: fixed;
  inset: 0;
  z-index: 10001;
  background: radial-gradient(circle at 50% 80%, rgba(60, 35, 10, 0.6), rgba(5, 2, 0, 0.9));
  backdrop-filter: blur(5px);
  -webkit-backdrop-filter: blur(5px);
  display: flex;
  align-items: flex-end;
  justify-content: center;
}
.gsel-sheet {
  width: min(520px, 100%);
  max-height: 72vh;
  display: flex;
  flex-direction: column;
  border: 4px solid var(--ff-gold-dark);
  border-bottom: none;
  border-radius: 22px 22px 0 0;
  padding: 12px 12px calc(18px + env(safe-area-inset-bottom));
  box-shadow:
    0 -6px 0 rgba(245, 197, 66, 0.35),
    0 -18px 50px rgba(0, 0, 0, 0.7);
}
.gsel-sheet-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 6px 12px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
.gsel-x {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  border: 3px solid var(--ff-brown-deep);
  background: radial-gradient(circle at 35% 30%, #ff9d94, var(--ff-red));
  color: white;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
  box-shadow: 0 4px 0 var(--ff-red-dark);
  transition: transform 0.3s var(--ff-bounce);
}
.gsel-x:active {
  transform: scale(0.9) rotate(90deg);
}
.gsel-list {
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-bottom: 4px;
}
.gsel-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 13px 14px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.07), transparent 40%),
    linear-gradient(180deg, #4a3218, #2c1a08);
  border: 3px solid #120a00;
  border-radius: 12px;
  color: var(--ff-parchment);
  font-family: var(--ff-font);
  font-size: 16px;
  cursor: pointer;
  box-shadow:
    inset 0 2px 0 rgba(255, 220, 150, 0.2),
    0 4px 0 #0d0700;
  animation: gs-item-in 0.4s var(--ff-bounce) both;
  transition:
    transform 0.2s var(--ff-bounce),
    border-color 0.2s ease,
    box-shadow 0.2s ease;
  text-align: left;
}
.gsel-item:hover,
.gsel-item:active {
  transform: translateY(-2px) scale(1.015);
  border-color: var(--ff-gold);
  box-shadow:
    inset 0 2px 0 rgba(255, 220, 150, 0.3),
    0 0 16px rgba(245, 197, 66, 0.55),
    0 4px 0 #0d0700;
}
.gsel-item-selected {
  border-color: var(--ff-gold);
  background:
    linear-gradient(180deg, rgba(245, 197, 66, 0.2), transparent 50%),
    linear-gradient(180deg, #5a3d1a, #331f0a);
}
.gsel-item-icon {
  font-size: 20px;
  width: 30px;
  text-align: center;
}
.gsel-item-label {
  flex: 1;
}
.gsel-check {
  color: var(--ff-gold);
  font-size: 20px;
  text-shadow: 0 0 10px rgba(245, 197, 66, 0.9);
  animation: bounce-in 0.4s var(--ff-bounce) both;
}
.gsel-empty {
  text-align: center;
  padding: 24px;
  color: rgba(242, 226, 194, 0.5);
}
.gsel-fade-enter-active,
.gsel-fade-leave-active {
  transition: opacity 0.25s ease;
}
.gsel-fade-enter-from,
.gsel-fade-leave-to {
  opacity: 0;
}
.gsel-sheet-enter-active {
  animation: gsel-sheet-in 0.45s var(--ff-bounce) both;
}
.gsel-sheet-leave-active {
  animation: gsel-sheet-in 0.25s ease reverse both;
}
@keyframes gsel-sheet-in {
  0% {
    transform: translateY(80%);
    opacity: 0.4;
  }
  70% {
    transform: translateY(-14px);
    opacity: 1;
  }
  100% {
    transform: translateY(0);
    opacity: 1;
  }
}
@keyframes gs-item-in {
  0% {
    transform: translateY(26px) scale(0.92);
    opacity: 0;
  }
  100% {
    transform: translateY(0) scale(1);
    opacity: 1;
  }
}
</style>
