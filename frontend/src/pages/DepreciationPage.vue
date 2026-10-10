<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
        <header class="page-head">
          <div class="head-left">
            <ion-back-button default-href="/accounts/assets" />
            <div>
              <h1 class="page-title">Depreciation</h1>
              <p class="page-subtitle">{{ asset?.name || assetId }}</p>
            </div>
          </div>
        </header>

        <p v-if="loading" class="loading-note">Loading schedule…</p>

        <div v-else-if="!depr" class="empty">
          <p class="empty-title">No depreciation data</p>
          <p class="empty-text">This schedule could not be loaded. Check your connection, then rebuild it.</p>
          <AppButton variant="primary" size="sm" @click="rebuild">Rebuild schedule</AppButton>
        </div>

        <template v-else>
          <section class="section">
            <h2 class="section-title">Summary</h2>

            <div class="stat-grid">
              <div class="stat-tile">
                <span class="stat-tile-label">Book value now</span>
                <span class="stat-tile-value">{{ formatMoney(depr.book_value) }}</span>
              </div>
              <div class="stat-tile">
                <span class="stat-tile-label">Accumulated</span>
                <span class="stat-tile-value">{{ formatMoney(depr.accumulated_depreciation) }}</span>
              </div>
            </div>

            <div class="card">
              <div class="meta-list">
                <div class="meta-row">
                  <span class="meta-key">Method</span>
                  <span class="meta-val">{{ methodLabel }}</span>
                </div>
                <div class="meta-row">
                  <span class="meta-key">Cost</span>
                  <span class="meta-val">{{ formatMoney(depr.cost) }}</span>
                </div>
                <div class="meta-row">
                  <span class="meta-key">Salvage value</span>
                  <span class="meta-val">{{ formatMoney(depr.salvage_value) }}</span>
                </div>
                <div class="meta-row">
                  <span class="meta-key">Useful life</span>
                  <span class="meta-val">{{ lifeLabel }}</span>
                </div>
                <div class="meta-row">
                  <span class="meta-key">Schedule start</span>
                  <span class="meta-val">{{ depr.schedule_start || 'Not set' }}</span>
                </div>
                <div class="meta-row">
                  <span class="meta-key">Fully depreciated</span>
                  <span class="meta-val">{{ depr.fully_depreciated_on || 'Not yet' }}</span>
                </div>
              </div>
            </div>

            <div class="summary-action">
              <AppButton variant="primary" size="md" @click="rebuild">Rebuild schedule</AppButton>
              <p v-if="rebuildError" class="form-error">{{ rebuildError }}</p>
            </div>
          </section>

          <section class="section">
            <div class="section-head">
              <h2 class="section-title">Schedule</h2>
              <span class="section-note">
                {{ periodCount }} {{ periodCount === 1 ? 'period' : 'periods' }}
              </span>
            </div>

            <div v-if="depr.problem" class="form-error">{{ depr.problem }}</div>

            <div v-if="periodCount" class="card">
              <div class="table-wrap">
                <table class="depr-table">
                  <thead>
                    <tr>
                      <th>Period</th>
                      <th>Start → End</th>
                      <th class="num">Opening</th>
                      <th class="num">Charge</th>
                      <th class="num">Closing</th>
                      <th class="num">Accumulated</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="p in depr.periods" :key="p.index">
                      <td>{{ p.index }}</td>
                      <td class="range">{{ p.start }} → {{ p.end }}</td>
                      <td class="num">{{ formatNumber(p.opening, 2) }}</td>
                      <td class="num">{{ formatNumber(p.charge, 2) }}</td>
                      <td class="num">{{ formatNumber(p.closing, 2) }}</td>
                      <td class="num">{{ formatNumber(p.accumulated, 2) }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>

            <div v-else class="empty">
              <p class="empty-title">No periods yet</p>
              <p class="empty-text">
                Rebuild the schedule above to generate a period for every month of this asset's life.
              </p>
            </div>
          </section>
        </template>
      </div>
    </ion-content>
  </ion-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { IonBackButton, IonContent, IonPage } from '@ionic/vue'
import AppButton from '@/components/ui/AppButton.vue'
import { request } from '@/api/client'
import { formatMoney, formatNumber } from '@/utils/money'

const route = useRoute()
const assetId = computed(() => route.params.id as string)
const depr = ref<any>(null)
const asset = ref<any>(null)
const loading = ref(true)
const rebuildError = ref('')

/** `straight_line` → `Straight line`, so the summary reads as prose. */
const methodLabel = computed(() => {
  const raw = String(depr.value?.method ?? '')
    .replace(/_/g, ' ')
    .trim()
  return raw ? raw.charAt(0).toUpperCase() + raw.slice(1) : 'Not set'
})

const lifeLabel = computed(() => {
  const months = depr.value?.useful_life_months
  return months ? `${months} months` : 'Not set'
})

const periodCount = computed(() => depr.value?.periods?.length ?? 0)

async function load() {
  try {
    depr.value = await request(`/api/assets/${assetId.value}/depreciation`, { method: 'GET' })
  } catch (e) { /* ignore */ }
  try {
    asset.value = await request(`/api/assets/${assetId.value}`, { method: 'GET' })
  } catch (e) { /* ignore */ }
  loading.value = false
}

async function rebuild() {
  rebuildError.value = ''
  try {
    await request(`/api/assets/${assetId.value}/depreciation`, { method: 'POST' })
  } catch (e) {
    rebuildError.value = e instanceof Error ? e.message : 'Could not rebuild the schedule.'
  }
  await load()
}

onMounted(load)
</script>

<style scoped>
.head-left {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  min-width: 0;
}
.summary-action {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
  align-items: flex-start;
}
.loading-note {
  padding: var(--density-card-pad);
  text-align: center;
  color: var(--text-muted);
  font-size: calc(0.88rem * var(--density-font-scale));
}
.table-wrap {
  overflow-x: auto;
}
.depr-table {
  width: 100%;
  border-collapse: collapse;
  font-size: calc(0.86rem * var(--density-font-scale));
}
.depr-table th,
.depr-table td {
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
}
.depr-table th:first-child,
.depr-table td:first-child {
  padding-left: 0;
}
.depr-table th:last-child,
.depr-table td:last-child {
  padding-right: 0;
}
.depr-table th {
  text-align: left;
  font-size: calc(0.72rem * var(--density-font-scale));
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-faint);
  border-bottom: 1px solid var(--border-strong);
  white-space: nowrap;
}
.depr-table td {
  border-bottom: 1px solid var(--border);
}
.depr-table tbody tr:last-child td {
  border-bottom: none;
}
.depr-table .range {
  white-space: nowrap;
  color: var(--text-muted);
}
.depr-table .num {
  font-variant-numeric: tabular-nums;
  text-align: right;
}
</style>
