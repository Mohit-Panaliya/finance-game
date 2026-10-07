<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    variant?: 'primary' | 'success' | 'danger' | 'neutral' | 'ghost'
    disabled?: boolean
    size?: 'sm' | 'md' | 'lg'
    block?: boolean
    type?: 'button' | 'submit'
  }>(),
  { variant: 'primary', size: 'md', type: 'button', disabled: false, block: false }
)

const emit = defineEmits<{ click: [e: MouseEvent] }>()

const classes = computed(() => [
  'abtn',
  `abtn-${props.variant}`,
  `abtn-${props.size}`,
  { 'abtn-block': props.block, 'abtn-disabled': props.disabled }
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
    <span class="abtn-face">
      <slot />
    </span>
  </button>
</template>

<style scoped>
.abtn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-md);
  background: var(--accent);
  color: var(--on-accent);
  font-family: var(--font-body);
  font-size: 0.95rem;
  font-weight: 600;
  line-height: 1.2;
  cursor: pointer;
  transition:
    background 0.15s ease,
    border-color 0.15s ease,
    opacity 0.15s ease;
}
.abtn:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}
.abtn:active:not(.abtn-disabled) {
  opacity: 0.85;
}
.abtn-face {
  display: inline-flex;
  align-items: center;
  gap: var(--density-gap);
}
/* Horizontal padding rides the density row gutter so all three sizes keep
   their ladder relative to one density instead of freezing in px. */
.abtn-sm {
  min-height: 34px;
  padding: 0 calc(var(--density-row-pad-x) - 3px);
  font-size: 0.82rem;
  border-radius: var(--radius-sm);
}
.abtn-md {
  min-height: 42px;
  padding: 0 calc(var(--density-row-pad-x) + 3px);
}
.abtn-lg {
  min-height: 50px;
  padding: 0 calc(var(--density-row-pad-x) + 9px);
  font-size: 1.05rem;
}
.abtn-block {
  display: flex;
  width: 100%;
}
.abtn-primary {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--on-accent);
}
.abtn-success {
  background: var(--success);
  border-color: var(--success);
  color: var(--on-accent);
}
.abtn-danger {
  background: var(--danger);
  border-color: var(--danger);
  color: var(--on-accent);
}
.abtn-neutral {
  background: var(--surface-2);
  border-color: var(--border-strong);
  color: var(--text);
}
.abtn-ghost {
  background: transparent;
  border-color: transparent;
  color: var(--accent);
}
.abtn-disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>