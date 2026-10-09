<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { IonIcon } from '@ionic/vue'
import { checkmarkCircleOutline } from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { ApiError } from '@/api/client'
import type { FinanceRow } from '@/types'
import { formatMoney } from '@/utils/money'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppModal from '@/components/ui/AppModal.vue'
import AppSelect from '@/components/ui/AppSelect.vue'

/**
 * Pay a credit card down from a bank (or straight from cash).
 *
 * The bank picker leads with an explicit "Cash (no bank)" option so a cash
 * repayment is a deliberate choice rather than an empty field, and the running
 * preview shows both movements the backend will make exactly once.
 */
const props = defineProps<{ open: boolean; card: FinanceRow | null }>()
const emit = defineEmits<{ close: [] }>()

const finance = useFinanceStore()

const owed = computed(() => round2(num(props.card?.current_balance)))
const currency = computed(() => String(props.card?.currency || 'INR'))

/** Explicit cash option first, then the user's own banks. */
const bankOptions = computed(() => [
  { value: '', label: 'Cash (no bank)' },
  ...finance
    .rows('banks')
    .filter((b) => b.id !== undefined && b.id !== null)
    .map((b) => ({ value: String(b.id), label: String(b.name ?? 'Account') }))
])

const amount = ref<string | number>('')
const bankId = ref('')
const amountError = ref('')
const error = ref('')
const saving = ref(false)
const result = ref<{ applied: number; bank: string | null; currency: string } | null>(null)

const amountNum = computed(() => {
  if (amount.value === '' || amount.value === null) return NaN
  return Number(amount.value)
})

/** Never apply more than is owed: the backend clamps to the same figure. */
const applied = computed(() => {
  if (!Number.isFinite(amountNum.value) || amountNum.value <= 0) return 0
  return round2(Math.min(amountNum.value, owed.value))
})

const clamped = computed(() => Number.isFinite(amountNum.value) && amountNum.value > owed.value)
const isCash = computed(() => bankId.value === '')
const bankLabel = computed(() =>
  bankOptions.value.find((o) => o.value === bankId.value)?.label ?? 'Cash (no bank)'
)

const preview = computed(() => {
  const appliedText = formatMoney(applied.value, currency.value)
  const source = isCash.value ? 'Cash' : bankLabel.value
  return `${source} −${appliedText} · Card −${appliedText}`
})

function num(value: unknown): number {
  const n = Number(value)
  return Number.isFinite(n) ? n : 0
}

function round2(n: number): number {
  return Math.round((n + Number.EPSILON) * 100) / 100
}

function reset() {
  amount.value = round2(owed.value)
  bankId.value = ''
  amountError.value = ''
  error.value = ''
  result.value = null
}

watch(
  () => [props.open, props.card ? String(props.card.id ?? '') : ''],
  () => {
    if (!props.open) return
    reset()
    void finance.fetchList('banks')
  },
  { immediate: true }
)

async function submit() {
  const card = props.card
  if (!card?.id) return
  amountError.value = ''
  error.value = ''

  if (!Number.isFinite(amountNum.value) || amountNum.value <= 0) {
    amountError.value = 'Enter an amount greater than zero'
    return
  }
  if (owed.value <= 0) {
    error.value = 'This card has no outstanding balance to repay'
    return
  }

  saving.value = true
  try {
    await finance.repayCard(String(card.id), applied.value, isCash.value ? undefined : bankId.value)
    result.value = { applied: applied.value, bank: isCash.value ? null : bankLabel.value, currency: currency.value }
  } catch (e) {
    error.value =
      e instanceof ApiError ? e.message : 'Could not record the repayment. Please try again.'
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <AppModal
    :open="open"
    title="Repay card"
    :subtitle="card ? String(card.name ?? '') : ''"
    :sheet="false"
    @close="emit('close')"
  >
    <div v-if="result" class="repay-done">
      <ion-icon class="repay-done-icon" :icon="checkmarkCircleOutline" />
      <p class="repay-done-title">Repayment recorded</p>
      <div class="meta-list">
        <div class="meta-row">
          <span class="meta-key">Applied</span>
          <span class="meta-val">{{ formatMoney(result.applied, result.currency) }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-key">From</span>
          <span class="meta-val">{{ result.bank ?? 'Cash (no bank)' }}</span>
        </div>
      </div>
    </div>

    <div v-else class="form-grid">
      <p v-if="error" class="form-error">{{ error }}</p>

      <div class="repay-owed">
        <span class="repay-owed-label">Outstanding</span>
        <span class="repay-owed-value">{{ formatMoney(owed, currency) }}</span>
      </div>

      <AppInput
        :model-value="amount"
        type="number"
        label="Repayment amount"
        required
        :suffix="currency"
        :error="amountError"
        :hint="clamped ? `Will be clamped to the ${formatMoney(owed, currency)} owed` : `Up to ${formatMoney(owed, currency)}`"
        @update:model-value="(v) => (amount = v)"
      />

      <AppSelect
        :options="bankOptions"
        :model-value="bankId"
        label="Pay from"
        @update:model-value="(v) => (bankId = String(v))"
      />

      <p class="repay-preview">{{ preview }}</p>
    </div>

    <template #footer>
      <AppButton v-if="result" variant="primary" size="md" block @click="emit('close')">Done</AppButton>
      <template v-else>
        <AppButton variant="neutral" size="md" @click="emit('close')">Cancel</AppButton>
        <AppButton variant="success" size="md" :disabled="saving" @click="submit">
          {{ saving ? 'Recording…' : 'Repay' }}
        </AppButton>
      </template>
    </template>
  </AppModal>
</template>

<style scoped>
.repay-owed {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--density-gap);
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}
.repay-owed-label {
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.repay-owed-value {
  font-size: 1.1rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}
.repay-preview {
  margin: 0;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text-muted);
  font-size: 0.85rem;
  font-variant-numeric: tabular-nums;
  text-align: center;
}
.repay-done {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--density-gap);
}
.repay-done-icon {
  font-size: 34px;
  color: var(--success);
}
.repay-done-title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}
.repay-done .meta-list {
  width: 100%;
}
</style>
