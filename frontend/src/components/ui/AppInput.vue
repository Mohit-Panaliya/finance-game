<script setup lang="ts">
import { computed, ref } from 'vue'

const props = withDefaults(
  defineProps<{
    modelValue?: string | number
    type?: 'text' | 'number' | 'password' | 'tel' | 'email'
    label?: string
    placeholder?: string
    suffix?: string
    hint?: string
    /** Inline field error; also switches the shell to the invalid style. */
    error?: string
    /** Renders the mandatory `*` marker next to the label. */
    required?: boolean
    disabled?: boolean
    min?: number
    max?: number
    step?: number
  }>(),
  {
    modelValue: '',
    type: 'text',
    label: '',
    placeholder: '',
    suffix: '',
    hint: '',
    error: '',
    required: false,
    disabled: false
  }
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
  <label class="ainput" :class="{ 'ainput-focused': focused, 'ainput-disabled': disabled, 'ainput-invalid': !!error }">
    <span v-if="label" class="ainput-label">
      {{ label }}<span v-if="required" class="ainput-req" aria-hidden="true">*</span>
    </span>
    <span class="ainput-shell">
      <input
        class="ainput-field"
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
      <span v-if="suffix" class="ainput-suffix">{{ suffix }}</span>
    </span>
    <span v-if="error" class="ainput-error">{{ error }}</span>
    <span v-else-if="hint" class="ainput-hint">{{ hint }}</span>
  </label>
</template>

<style scoped>
.ainput {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
  width: 100%;
}
.ainput-label {
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.ainput-req {
  margin-left: 3px;
  color: var(--danger);
}
.ainput-shell {
  display: flex;
  align-items: center;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}
.ainput-focused .ainput-shell {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.ainput-field {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text);
  font-size: 1rem;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  -webkit-user-select: text;
  user-select: text;
}
.ainput-field::placeholder {
  color: var(--text-faint);
}
.ainput-field::-webkit-outer-spin-button,
.ainput-field::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
.ainput-suffix {
  color: var(--text-muted);
  font-size: 0.85rem;
  padding-right: var(--density-row-pad-x);
  white-space: nowrap;
}
.ainput-hint {
  font-size: 0.72rem;
  color: var(--text-faint);
}
.ainput-invalid .ainput-shell {
  border-color: var(--danger);
}
.ainput-invalid.ainput-focused .ainput-shell {
  box-shadow: 0 0 0 3px var(--danger-soft);
}
.ainput-error {
  font-size: 0.72rem;
  color: var(--danger);
}
.ainput-disabled {
  opacity: 0.55;
}
</style>