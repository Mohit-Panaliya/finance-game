<script setup lang="ts">
import { computed, ref, onBeforeUnmount, watch } from 'vue'
import { IonIcon } from '@ionic/vue'
import { checkmarkOutline, chevronDownOutline } from 'ionicons/icons'

export interface AppSelectOption {
  value: string | number
  label: string
}

const props = withDefaults(
  defineProps<{
    options: AppSelectOption[]
    modelValue?: string | number | null
    placeholder?: string
    disabled?: boolean
    label?: string
  }>(),
  { modelValue: null, placeholder: 'Choose…', disabled: false, label: '' }
)

const emit = defineEmits<{ 'update:modelValue': [value: string | number]; open: []; close: [] }>()

const open = ref(false)

const selected = computed(
  () => props.options.find((o) => String(o.value) === String(props.modelValue)) ?? null
)

function toggle() {
  if (props.disabled) return
  open.value = !open.value
  if (open.value) emit('open')
  else emit('close')
}

function pick(opt: AppSelectOption) {
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
  <div class="asel" :class="{ 'asel-disabled': disabled }">
    <span v-if="label" class="asel-label">{{ label }}</span>

    <button
      class="asel-trigger"
      :class="{ 'asel-trigger-open': open }"
      type="button"
      :disabled="disabled"
      :aria-expanded="open"
      @click="toggle"
    >
      <span class="asel-text clamp-1">
        {{ selected ? selected.label : placeholder }}
      </span>
      <ion-icon class="asel-caret" :class="{ 'asel-caret-up': open }" :icon="chevronDownOutline" />
    </button>

    <teleport to="body">
      <transition name="asel-fade">
        <div v-if="open" class="asel-overlay" @click.self="open = false; emit('close')">
          <transition name="asel-sheet" appear>
            <div class="asel-sheet">
              <div class="asel-sheet-head">
                <span class="asel-sheet-title">{{ label || 'Select' }}</span>
                <button class="asel-x" type="button" aria-label="Close" @click="open = false; emit('close')">
                  <ion-icon :icon="checkmarkOutline" class="asel-x-icon asel-x-hidden" />
                  <span aria-hidden="true">&times;</span>
                </button>
              </div>
              <div class="asel-list ff-hide-scrollbar">
                <button
                  v-for="(opt, i) in options"
                  :key="String(opt.value)"
                  class="asel-item"
                  :class="{ 'asel-item-selected': String(opt.value) === String(modelValue) }"
                  type="button"
                  @click="pick(opt)"
                >
                  <span class="asel-item-label">{{ opt.label }}</span>
                  <ion-icon
                    v-if="String(opt.value) === String(modelValue)"
                    class="asel-check"
                    :icon="checkmarkOutline"
                  />
                </button>
                <div v-if="!options.length" class="asel-empty">No options</div>
              </div>
            </div>
          </transition>
        </div>
      </transition>
    </teleport>
  </div>
</template>

<style scoped>
.asel {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
  width: 100%;
}
.asel-label {
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.asel-trigger {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  width: 100%;
  min-height: 44px;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text);
  font-family: var(--font-body);
  font-size: 0.95rem;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}
.asel-trigger-open {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.asel-text {
  flex: 1;
  text-align: left;
}
.asel-caret {
  flex-shrink: 0;
  font-size: 16px;
  color: var(--text-muted);
  transition: transform 0.2s ease;
}
.asel-caret-up {
  transform: rotate(180deg);
}
.asel-disabled {
  opacity: 0.55;
  pointer-events: none;
}
.asel-overlay {
  position: fixed;
  inset: 0;
  z-index: 10001;
  background: var(--scrim);
  display: flex;
  align-items: flex-end;
  justify-content: center;
}
.asel-sheet {
  width: min(520px, 100%);
  max-height: 72vh;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border: 1px solid var(--border);
  border-bottom: none;
  border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  padding: var(--density-card-pad) var(--density-card-pad)
    calc(var(--density-card-pad) + env(safe-area-inset-bottom));
  box-shadow: var(--shadow-lg);
}
.asel-sheet-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2px 6px var(--density-gap);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.asel-sheet-title {
  font-size: 0.78rem;
  font-weight: 700;
  color: var(--text-muted);
}
.asel-x {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: var(--surface-2);
  color: var(--text-muted);
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  padding: 0;
}
.asel-x-hidden {
  display: none;
}
.asel-list {
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
  padding-bottom: 4px;
}
.asel-item {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  width: 100%;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text);
  font-family: var(--font-body);
  font-size: 0.95rem;
  cursor: pointer;
  text-align: left;
  transition:
    border-color 0.15s ease,
    background 0.15s ease;
}
.asel-item-selected {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.asel-item-label {
  flex: 1;
}
.asel-check {
  font-size: 18px;
  color: var(--accent);
  flex-shrink: 0;
}
.asel-empty {
  text-align: center;
  padding: var(--density-card-pad);
  color: var(--text-faint);
}
.asel-fade-enter-active,
.asel-fade-leave-active {
  transition: opacity 0.18s ease;
}
.asel-fade-enter-from,
.asel-fade-leave-to {
  opacity: 0;
}
.asel-sheet-enter-active,
.asel-sheet-leave-active {
  transition: transform 0.24s ease;
}
.asel-sheet-enter-from,
.asel-sheet-leave-to {
  transform: translateY(100%);
}
</style>