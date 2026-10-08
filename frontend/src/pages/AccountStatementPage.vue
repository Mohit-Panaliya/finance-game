<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import {
  arrowBackOutline,
  cardOutline,
  pricetagOutline,
  statsChartOutline,
  walletOutline
} from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import { entityConfig, isEntityType, rowSubtitle, rowValue } from '@/entityConfig'
import type { EntityType, FinanceRow, StatementEntry } from '@/types'
import { api, ApiError } from '@/api/client'
import { formatMoney, formatSigned, rowCurrency } from '@/utils/money'
import { formatDate } from '@/utils/date'
import SyncChip from '@/components/ui/SyncChip.vue'

const route = useRoute()
const router = useRouter()
const finance = useFinanceStore()
const sync = useSyncStore()

/** One of the two routes above; used to address the right list + endpoint. */
const entity = computed<EntityType>(() =>
  route.name === 'card-statement' ? 'credit-cards' : 'banks'
)

const cfg = computed(() => entityConfig(entity.value))

const accountId = computed<string>(() => {
  const raw = route.params.id
  return Array.isArray(raw) ? raw[0] : (raw ?? '')
})

const account = computed<FinanceRow | null>(() => {
  const id = accountId.value
  if (!id) return null
  return finance.rows(entity.value).find((r) => String(r.id) === id) ?? null
})

const accountLoading = computed(() => finance.lists[entity.value].loading)

const statement = ref<StatementEntry[]>([])
const statementLoading = ref(false)
const statementError = ref('')
/** Guards against a slow response landing after the route changed. */
const statementFor = ref('')

/** Header meta line: the account's configured subtitle columns, or a plain fallback. */
const accountMeta = computed(() => {
  const row = account.value
  if (!row) return ''
  const sub = rowSubtitle(cfg.value, row)
  return sub || (entity.value === 'banks' ? 'Bank account' : 'Card')
})

const currency = computed(() => (account.value ? rowCurrency(account.value) : null))

const balance = computed(() => (account.value ? rowValue(cfg.value, account.value) : 0))

async function loadStatement(): Promise<void> {
  const id = accountId.value
  statementFor.value = id
  statementError.value = ''
  statement.value = []
  if (!id) return
  statementLoading.value = true
  try {
    const rows = await api.get<StatementEntry[]>(`/${entity.value}/${id}/statement`)
    if (statementFor.value !== id) return
    statement.value = Array.isArray(rows) ? rows : []
  } catch (e) {
    if (statementFor.value !== id) return
    statementError.value = e instanceof ApiError ? e.message : 'Statement unavailable'
  } finally {
    if (statementFor.value === id) statementLoading.value = false
  }
}

/** Each entry links to the record it came from (`/accounts/incomes/{id}`, …). */
function entryHref(entry: StatementEntry): string {
  const plural = `${entry.entry_type}s`
  return isEntityType(plural) ? `/accounts/${plural}/${entry.id}` : ''
}

function entryIcon(entry: StatementEntry) {
  if (entry.entry_type === 'income') return statsChartOutline
  if (entry.entry_type === 'expense') return pricetagOutline
  return walletOutline
}

watch(accountId, () => {
  void loadStatement()
})

onMounted(async () => {
  await finance.fetchList(entity.value)
  void loadStatement()
})
</script>

<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div class="head-left">
            <button class="back-btn" type="button" aria-label="Back" @click="router.back()">
              <ion-icon :icon="arrowBackOutline" />
            </button>
            <div>
              <h1 class="page-title">Statement</h1>
              <p class="page-subtitle">{{ cfg.label }}</p>
            </div>
          </div>
          <SyncChip :status="sync.status" :pending="sync.queueCount" @sync="finance.fetchList(entity)" />
        </header>

        <p v-if="!accountLoading && !account && !statementLoading" class="form-error">
          <template v-if="entity === 'banks'">Bank account</template>
          <template v-else>Card</template>
          not found. It may have been deleted.
        </p>

        <section v-else class="card account-hero">
          <template v-if="account">
            <p class="card-label">
              {{ entity === 'banks' ? 'Current balance' : 'Outstanding balance' }}
            </p>
            <p class="card-value balance">{{ formatMoney(balance, currency) }}</p>
            <p class="card-foot account-meta">
              {{ accountMeta }} · {{ entity === 'banks' ? 'Bank' : 'Card' }}
            </p>
          </template>
          <p v-else class="text-sm text-muted">Loading account…</p>
        </section>

        <section class="section">
          <div class="section-head">
            <h2 class="section-title">Ledger trail</h2>
            <span v-if="statement.length" class="section-note">
              {{ statement.length }} {{ statement.length === 1 ? 'movement' : 'movements' }}
            </span>
          </div>

          <p v-if="statementLoading" class="loading-note">Loading statement…</p>
          <p v-else-if="statementError" class="form-error">{{ statementError }}</p>

          <div v-else-if="!statement.length" class="empty">
            <ion-icon class="empty-icon" :icon="cardOutline" />
            <p class="empty-title">No movements yet</p>
            <p class="empty-text">
              Record an income or expense against this account and it will show up here.
            </p>
          </div>

          <div v-else class="row-list">
            <router-link
              v-for="e in statement"
              :key="`${e.entry_type}-${e.id}`"
              class="row-item stmt-row"
              :to="entryHref(e)"
            >
              <span class="row-icon" :class="e.signed_amount < 0 ? 'text-danger' : 'text-success'">
                <ion-icon :icon="entryIcon(e)" />
              </span>
              <span class="row-main">
                <span class="row-title clamp-1">{{ e.title || e.entry_type }}</span>
                <span class="row-sub clamp-1">{{ formatDate(e.occurred_on) }} · {{ e.entry_type }}</span>
              </span>
              <span class="row-value">
                <span :class="e.signed_amount < 0 ? 'text-danger' : 'text-success'">
                  {{ formatSigned(e.signed_amount, currency) }}
                </span>
                <span class="row-extra">{{ formatMoney(e.balance_after, currency) }}</span>
              </span>
            </router-link>
          </div>
        </section>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.head-left {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  min-width: 0;
}
.back-btn {
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 50%;
  color: var(--text);
  font-size: 18px;
  cursor: pointer;
}
.account-hero {
  display: flex;
  flex-direction: column;
}
.balance {
  margin: 0;
}
.account-meta {
  margin: 6px 0 0;
}
.loading-note {
  padding: var(--density-card-pad);
  text-align: center;
  color: var(--text-muted);
  font-size: 0.88rem;
}
/* Running-balance column should align with the signed amount above it. */
.row-extra {
  font-variant-numeric: tabular-nums;
}
</style>