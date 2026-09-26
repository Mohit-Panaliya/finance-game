<script setup lang="ts">
import { computed, ref } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue?: string | number
    type?: 'text' | 'number' | 'password' | 'tel' | 'email'
    label?: string
    placeholder?: string
    suffix?: string
    disabled?: boolean
    min?: number
    max?: number
    step?: number
  }>(),
  { modelValue: '', type: 'text', label: '', placeholder: '', suffix: '', disabled: false }
)

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
  focus: []
  blur: []
}>()

const focused = ref(false)

const inner = computed(() =>
  props.modelValue === undefined || props.modelValue === null ? '' : String(props.modelValue)
)

function onInput(e: Event) {
  const el = e.target as HTMLInputElement
  if (props.type === 'number') {
    const n = el.value === '' ? '' : Number(el.value)
    emit('update:modelValue', n === '' ? '' : (n as number))
  } else {
    emit('update:modelValue', el.value)
  }
}
</script>

<template>
  <label class="ginput" :class="{ 'ginput-focused': focused, 'ginput-disabled': disabled }">
    <span v-if="label" class="ginput-label carved carved-sm">{{ label }}</span>
    <span class="ginput-shell">
      <span class="ginput-rivet ginput-rivet-l" aria-hidden="true" />
      <input
        class="ginput-field"
        :type="type === 'number' ? 'text' : type"
        :inputmode="type === 'number' ? 'decimal' : undefined"
        :value="inner"
        :placeholder="placeholder"
        :disabled="disabled"
        :min="min"
        :max="max"
        :step="step"
        autocomplete="off"
        autocapitalize="off"
        spellcheck="false"
        @input="onInput"
        @focus="focused = true; emit('focus')"
        @blur="focused = false; emit('blur')"
      />
      <span v-if="suffix" class="ginput-suffix">{{ suffix }}</span>
      <span class="ginput-rivet ginput-rivet-r" aria-hidden="true" />
      <span class="ginput-ring" aria-hidden="true" />
    </span>
  </label>
</template>

<style scoped>
.ginput {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 100%;
}
.ginput-label {
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.12em;
  color: #d8b56a;
}
.ginput-shell {
  position: relative;
  display: flex;
  align-items: center;
  background:
    radial-gradient(circle at 25% 20%, rgba(255, 255, 255, 0.07), transparent 50%),
    linear-gradient(180deg, #33200c 0%, #241405 100%);
  border: 3px solid #120a00;
  border-radius: 14px;
  box-shadow:
    inset 0 5px 12px rgba(0, 0, 0, 0.75),
    inset 0 -2px 0 rgba(255, 220, 150, 0.12),
    0 3px 0 rgba(0, 0, 0, 0.55);
  overflow: hidden;
  transition: box-shadow 0.3s var(--ff-bounce);
}
.ginput-focused .ginput-shell {
  border-color: var(--ff-gold);
  box-shadow:
    inset 0 5px 12px rgba(0, 0, 0, 0.75),
    0 0 0 3px rgba(245, 197, 66, 0.35),
    0 0 22px rgba(245, 197, 66, 0.55);
  animation: pulse-glow 1.6s ease-in-out infinite;
}
.ginput-field {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  color: var(--ff-parchment);
  font-family: var(--ff-font-body);
  font-weight: 700;
  font-size: 16px;
  padding: 13px 14px;
  caret-color: var(--ff-gold);
  -webkit-user-select: text;
  user-select: text;
}
.ginput-field::placeholder {
  color: rgba(242, 226, 194, 0.35);
}
.ginput-field::-webkit-outer-spin-button,
.ginput-field::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
.ginput-suffix {
  color: var(--ff-gold);
  font-family: var(--ff-font);
  padding-right: 14px;
  font-size: 15px;
  text-shadow: 0 1px 0 #5c3c00;
}
.ginput-rivet {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: radial-gradient(circle at 35% 30%, #ffe27a, #8a5a00);
  box-shadow: inset 0 -1px 2px rgba(0, 0, 0, 0.6);
  flex-shrink: 0;
}
.ginput-rivet-l { margin-left: 8px; }
.ginput-rivet-r { margin-right: 8px; }
.ginput-ring {
  position: absolute;
  inset: 0;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.3s ease;
}
.ginput-focused .ginput-ring {
  opacity: 1;
}
.ginput-disabled {
  opacity: 0.5;
}
</style>
