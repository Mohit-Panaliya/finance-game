<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import { addOutline, chevronForwardOutline, walletOutline } from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import { ACCOUNT_ENTITIES, entityConfig, rowTitle, rowValue, summaryHead } from '@/entityConfig'
import type { EntityConfig } from '@/entityConfig'
import type { AccountEntityType, FinanceRow } from '@/types'
import { formatMoney, rowCurrency } from '@/utils/money'
import AppButton from '@/components/ui/AppButton.vue'
import SyncChip from '@/components/ui/SyncChip.vue'

const router = useRouter()
const finance = useFinanceStore()
const sync = useSyncStore()

interface Group {
  key: string
  cfg: EntityConfig
  total: number
  count: number
}

/** One bank/card account row, linked straight to its full statement page. */
interface AccountRow {
  id: string
  name: string
  value: number
  currency: string
}

const groups = computed<Group[]>(() =>
  ACCOUNT_ENTITIES.map((key) => {
    const cfg = entityConfig(key)
    const head = summaryHead(key, finance.summaries[key])
    return { key, cfg, total: head.total, count: head.count }
  })
)

/** Banks and cards are statement accounts — the ledger trail lives behind each id. */
const STATEMENT_KEYS: AccountEntityType[] = ['banks', 'credit-cards']

/** Share of the money total, used for the allocation bar. */
const moneyTotal = computed(() =>
  groups.value
    .filter((g) => g.key !== 'credit-cards')
    .reduce((acc, g) => acc + g.total, 0)
)

const cardTotal = computed(() => groups.value.find((g) => g.key === 'credit-cards')?.total ?? 0)

const allocation = computed(() =>
  groups.value
    .filter((g) => g.key !== 'credit-cards' && g.total > 0)
    .map((g) => ({ label: g.cfg.label, value: g.total, pct: (g.total / moneyTotal.value) * 100, cfg: g.cfg }))
)

const loading = computed(() => ACCOUNT_ENTITIES.some((key) => finance.lists[key].loading))

function open(entity: string) {
  router.push(`/accounts/${entity}`)
}

function accountRows(key: AccountEntityType): AccountRow[] {
  return (finance.rows(key) as FinanceRow[]).map((r) => ({
    id: String(r.id ?? ''),
    name: rowTitle(entityConfig(key), r),
    value: rowValue(entityConfig(key), r),
    currency: rowCurrency(r)
  }))
}

function add() {
  router.push('/accounts/banks/new')
}

onMounted(() => {
  void finance.fetchAll()
})
</script>

<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div>
            <h1 class="page-title">Accounts</h1>
            <p class="page-subtitle">{{ formatMoney(moneyTotal) }} tracked in {{ groups.length }} groups</p>
          </div>
          <SyncChip :status="sync.status" :pending="sync.queueCount" @sync="finance.refresh()" />
        </header>

        <section v-if="allocation.length" class="card">
          <p class="card-label">Allocation</p>
          <div class="alloc-bar" role="img" aria-label="Allocation by group">
            <span
              v-for="a in allocation"
              :key="a.label"
              class="alloc-seg"
              :style="{ width: `${a.pct}%`, background: a.cfg.accent }"
            />
          </div>
          <div class="legend">
            <span v-for="a in allocation" :key="a.label" class="legend-item">
              <span class="legend-dot" :style="{ background: a.cfg.accent }" />
              {{ a.label }} · {{ a.pct.toFixed(0) }}%
            </span>
          </div>
        </section>

        <div class="row-list">
          <template v-for="g in groups" :key="g.key">
            <button class="row-item" type="button" @click="open(g.key)">
              <span class="row-icon"><ion-icon :icon="g.cfg.icon" /></span>
              <span class="row-main">
                <span class="row-title">{{ g.cfg.label }}</span>
                <span class="row-sub">
                  {{ g.count }} {{ g.count === 1 ? 'record' : 'records' }}
                  <template v-if="g.key === 'credit-cards'"> · {{ formatMoney(g.total) }} outstanding</template>
                </span>
              </span>
              <span class="row-value">
                {{ formatMoney(g.total, 'INR', { compact: true }) }}
                <span class="row-extra" v-if="g.key !== 'credit-cards'">
                  {{ moneyTotal > 0 ? `${((g.total / moneyTotal) * 100).toFixed(0)}%` : '—' }}
                </span>
              </span>
              <ion-icon class="text-faint" :icon="chevronForwardOutline" />
            </button>
            <!-- Each bank/card links to its full statement: the trail is per account. -->
            <router-link
              v-for="a in accountRows(g.key as AccountEntityType)"
              :key="`${g.key}-${a.id}`"
              class="row-item row-item-stmt"
              :to="`/accounts/${g.key}/${a.id}`"
            >
              <span class="row-icon row-icon-stmt"><ion-icon :icon="walletOutline" /></span>
              <span class="row-main">
                <span class="row-title clamp-1">{{ a.name }}</span>
                <span class="row-sub">View statement</span>
              </span>
              <span class="row-value">{{ formatMoney(a.value, a.currency, { compact: true }) }}</span>
              <ion-icon class="text-faint" :icon="chevronForwardOutline" />
            </router-link>
          </template>
        </div>

        <section v-if="cardTotal > 0" class="card">
          <p class="card-label">Card utilisation</p>
          <p class="card-value card-value-sm">{{ formatMoney(cardTotal) }}</p>
          <p class="card-foot">Outstanding across {{ groups.find((g) => g.key === 'credit-cards')?.count ?? 0 }} cards</p>
        </section>

        <div v-if="!loading && !groups.some((g) => g.count > 0)" class="empty">
          <ion-icon class="empty-icon" :icon="walletOutline" />
          <p class="empty-title">No accounts yet</p>
          <p class="empty-text">Start with a bank account — you can add assets, deposits, investments and cards later.</p>
        </div>

        <AppButton variant="primary" size="md" block @click="add">
          <ion-icon :icon="addOutline" /> Add a record
        </AppButton>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.alloc-bar {
  display: flex;
  gap: 2px;
  height: 10px;
  margin: var(--density-gap) 0 2px;
  border-radius: 999px;
  overflow: hidden;
  background: var(--surface-3);
}
.alloc-seg {
  height: 100%;
}
/* statement sub-rows sit under their group row, visually nested */
.row-item-stmt {
  padding-left: calc(var(--density-row-pad-x) + 38px);
  border-top: 1px dashed var(--border);
}
.row-icon-stmt {
  width: 26px;
  height: 26px;
  font-size: 14px;
}
</style>