<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { IonIcon } from '@ionic/vue'
import { checkmarkCircleOutline } from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { ApiError } from '@/api/client'
import type { DebtRow } from '@/types'
import { formatMoney } from '@/utils/money'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppModal from '@/components/ui/AppModal.vue'

/**
 * Grow a debt's principal ("I owe 100, add 100 more").
 *
 * The preview walks the user from the current principal to the resulting total,
 * and the modal surfaces the backend's own 400/404 message if the write fails.
 */
const props = defineProps<{ open: boolean; debt: DebtRow | null }>()
const emit = defineEmits<{ close: [] }>()

const finance = useFinanceStore()

const currency = computed(() => String(props.debt?.currency || 'INR'))
const currentAmount = computed(() => round2(num(props.debt?.amount)))

const amount = ref<string | number>('')
const amountError = ref('')
const error = ref('')
const saving = ref(false)
const result = ref<{ added: number; total: number; currency: string } | null>(null)

const add = computed(() => {
  if (amount.value === '' || amount.value === null) return NaN
  const n = Number(amount.value)
  return Number.isFinite(n) && n > 0 ? round2(n) : NaN
})

const resulting = computed(() =>
  Number.isFinite(add.value) ? round2(currentAmount.value + add.value) : currentAmount.value
)

const preview = computed(
  () => `${formatMoney(currentAmount.value, currency.value)} → ${formatMoney(resulting.value, currency.value)}`
)

function num(value: unknown): number {
  const n = Number(value)
  return Number.isFinite(n) ? n : 0
}

function round2(n: number): number {
  return Math.round((n + Number.EPSILON) * 100) / 100
}

function reset() {
  amount.value = ''
  amountError.value = ''
  error.value = ''
  result.value = null
}

watch(
  () => [props.open, props.debt ? String(props.debt.id ?? '') : ''],
  () => {
    if (props.open) reset()
  },
  { immediate: true }
)

async function submit() {
  const debt = props.debt
  if (!debt?.id) return
  amountError.value = ''
  error.value = ''

  if (!Number.isFinite(add.value) || add.value <= 0) {
    amountError.value = 'Enter an amount greater than zero'
    return
  }

  saving.value = true
  try {
    await finance.topUpDebt(String(debt.id), add.value)
    result.value = { added: add.value, total: resulting.value, currency: currency.value }
  } catch (e) {
    error.value = e instanceof ApiError ? e.message : 'Could not add to this debt. Please try again.'
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <AppModal
    :open="open"
    title="Add to debt"
    :subtitle="debt ? String(debt.counterparty ?? '') : ''"
    :sheet="false"
    @close="emit('close')"
  >
    <div v-if="result" class="topup-done">
      <ion-icon class="topup-done-icon" :icon="checkmarkCircleOutline" />
      <p class="topup-done-title">Principal updated</p>
      <div class="meta-list">
        <div class="meta-row">
          <span class="meta-key">Added</span>
          <span class="meta-val">{{ formatMoney(result.added, result.currency) }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-key">New total</span>
          <span class="meta-val">{{ formatMoney(result.total, result.currency) }}</span>
        </div>
      </div>
    </div>

    <div v-else class="form-grid">
      <p v-if="error" class="form-error">{{ error }}</p>

      <div class="topup-current">
        <span class="topup-current-label">Current principal</span>
        <span class="topup-current-value">{{ formatMoney(currentAmount, currency) }}</span>
      </div>

      <AppInput
        :model-value="amount"
        type="number"
        label="Amount to add"
        required
        :suffix="currency"
        :error="amountError"
        @update:model-value="(v) => (amount = v)"
      />

      <p class="topup-preview">{{ preview }}</p>
    </div>

    <template #footer>
      <AppButton v-if="result" variant="primary" size="md" block @click="emit('close')">Done</AppButton>
      <template v-else>
        <AppButton variant="neutral" size="md" @click="emit('close')">Cancel</AppButton>
        <AppButton variant="primary" size="md" :disabled="saving" @click="submit">
          {{ saving ? 'Adding…' : 'Add' }}
        </AppButton>
      </template>
    </template>
  </AppModal>
</template>

<style scoped>
.topup-current {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--density-gap);
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}
.topup-current-label {
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.topup-current-value {
  font-size: 1.1rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}
.topup-preview {
  margin: 0;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text);
  font-size: 1rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
  text-align: center;
}
.topup-done {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--density-gap);
}
.topup-done-icon {
  font-size: 34px;
  color: var(--success);
}
.topup-done-title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}
.topup-done .meta-list {
  width: 100%;
}
</style>
