<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  IonPage,
  IonContent,
  IonHeader,
  IonFab,
  IonFabButton,
  onIonViewWillEnter
} from '@ionic/vue'
import ResourceBar from '@/components/game/ResourceBar.vue'
import SyncChip from '@/components/game/SyncChip.vue'
import { useGameStore } from '@/stores/gameStore'
import { useFinanceStore } from '@/stores/financeStore'
import { ENTITY_CONFIGS, ENTITY_LIST, formatGold } from '@/entityConfig'
import { useCountUp } from '@/composables/useCountUp'
import type { EntityType } from '@/types'

const router = useRouter()
const game = useGameStore()
const finance = useFinanceStore()

function sumEntity(entity: EntityType, keys: string[]): number {
  const list = finance.lists[entity]
  if (!list) return 0
  return list.items.reduce((s, row) => {
    for (const k of keys) {
      if (row[k] !== undefined && row[k] !== null && row[k] !== '') {
        const n = Number(row[k])
        if (Number.isFinite(n)) return s + n
      }
    }
    return s
  }, 0)
}

const yearlyIncome = computed(() => {
  const list = finance.lists.incomes
  if (!list) return 0
  return list.items.reduce((s, row) => {
    const amount = Number(row.amount ?? 0) || 0
    const freq = String(row.frequency ?? 'monthly')
    if (freq === 'yearly') return s + amount
    if (freq === 'weekly') return s + amount * 52
    if (freq === 'once') return s + amount
    return s + amount * 12
  }, 0)
})

const monthlyFixed = computed(() => {
  const list = finance.lists.expenses
  if (!list) return 0
  return list.items.reduce((s, row) => {
    const amount = Number(row.amount ?? 0) || 0
    const freq = String(row.frequency ?? 'once')
    if (freq === 'yearly') return s + amount / 12
    if (freq === 'weekly') return s + (amount * 52) / 12
    if (freq === 'monthly') return s + amount
    return s
  }, 0)
})

const netWorth = computed(
  () =>
    sumEntity('assets', ['value', 'amount']) +
    sumEntity('banks', ['balance']) +
    sumEntity('investments', ['amount', 'value']) +
    sumEntity('fixed-deposits', ['principal']) -
    sumEntity('credit-cards', ['outstanding'])
)

const roiPct = computed(() => {
  const list = finance.lists.investments
  if (!list || !list.items.length) return 0
  const invested = list.items.reduce((s, r) => s + (Number(r.amount ?? r.value ?? 0) || 0), 0)
  const ret = list.items.reduce((s, r) => s + (Number(r.returns ?? 0) || 0), 0)
  if (invested <= 0) return 0
  return (ret / invested) * 100
})

const donutPct = computed(() => Math.max(0, Math.min(100, roiPct.value)))
const DONUT_C = 2 * Math.PI * 40
const donutOffset = computed(() => DONUT_C * (1 - donutPct.value / 100))

const animYearly = useCountUp(yearlyIncome)
const animMonthly = useCountUp(monthlyFixed)
const animNetWorth = useCountUp(netWorth)
const animRoi = useCountUp(roiPct, 1100)

function entityTotal(entity: EntityType): number {
  return sumEntity(entity, ENTITY_CONFIGS[entity].valueKeys)
}

const cardTotals = computed(() => {
  const out: Record<string, number> = {}
  for (const e of ENTITY_LIST) out[e] = entityTotal(e)
  return out
})
const animCards: Record<string, ReturnType<typeof useCountUp>> = {}
for (const e of ENTITY_LIST) {
  animCards[e] = useCountUp(() => cardTotals.value[e] ?? 0)
}

async function load() {
  await Promise.all([
    ...ENTITY_LIST.map((e) => finance.fetchList(e, { reset: true, perPage: 200 })),
    finance.fetchSummary()
  ])
}

onIonViewWillEnter(() => {
  void load()
  void game.fetchVillage()
})
onMounted(() => void load())

function fmt(n: number): string {
  return formatGold(n)
}
</script>

<template>
  <ion-page>
    <ion-header class="hud">
      <div class="hud-row">
        <h1 class="carved carved-gold hud-title">RECORDS</h1>
        <SyncChip />
      </div>
    </ion-header>

    <ion-content :fullscreen="true" class="rec-content">
      <div class="rec-scroll ff-hide-scrollbar">
        <!-- ============ SUMMARY ============ -->
        <div class="summary panel-stone noise">
          <div class="sum-main">
            <span class="sum-label carved carved-sm">Yearly Income</span>
            <span class="sum-big carved carved-gold shimmer-text">🪙 {{ fmt(animYearly) }}</span>
          </div>

          <div class="sum-donut-wrap">
            <svg class="donut" viewBox="0 0 100 100" aria-label="ROI donut">
              <circle class="donut-track" cx="50" cy="50" r="40" />
              <circle
                class="donut-value"
                cx="50"
                cy="50"
                r="40"
                :stroke-dasharray="DONUT_C"
                :stroke-dashoffset="donutOffset"
              />
            </svg>
            <div class="donut-center">
              <span class="donut-num carved carved-gold">{{ animRoi.toFixed(1) }}%</span>
              <span class="donut-cap carved carved-sm">ROI</span>
            </div>
          </div>

          <div class="sum-cells">
            <div class="sum-cell">
              <span class="sum-k carved carved-sm">Monthly Fixed</span>
              <span class="sum-v carved">{{ fmt(animMonthly) }}</span>
            </div>
            <div class="sum-cell">
              <span class="sum-k carved carved-sm">Net Worth</span>
              <span class="sum-v carved net">{{ fmt(animNetWorth) }}</span>
            </div>
          </div>
        </div>

        <!-- ============ CATEGORY CHESTS ============ -->
        <div class="section-head">
          <h2 class="carved carved-gold">TREASURY ROOMS</h2>
          <p class="carved carved-sm muted">Open a chest to manage records</p>
        </div>

        <div class="cats">
          <button
            v-for="(e, i) in ENTITY_LIST"
            :key="e"
            class="cat stone-card"
            :style="{ animationDelay: `${i * 60}ms`, '--accent': ENTITY_CONFIGS[e].accent }"
            type="button"
            @click="router.push(`/records/${e}`)"
          >
            <span class="cat-hinge cat-hinge-l" aria-hidden="true" />
            <span class="cat-hinge cat-hinge-r" aria-hidden="true" />
            <span class="cat-icon bob" :style="{ animationDelay: `${(i % 4) * 0.35}s` }">
              {{ ENTITY_CONFIGS[e].icon }}
            </span>
            <span class="cat-label carved">{{ ENTITY_CONFIGS[e].label }}</span>
            <span class="cat-total carved carved-gold">🪙 {{ fmt(animCards[e].value) }}</span>
            <span class="cat-arrow">→</span>
          </button>
        </div>
      </div>

      <ion-fab slot="fixed" vertical="bottom" horizontal="end" class="rec-fab">
        <ion-fab-button @click="router.push('/records/incomes/new')">
          <span class="fab-glyph">+</span>
        </ion-fab-button>
      </ion-fab>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.hud {
  position: relative;
  z-index: 20;
  background: linear-gradient(180deg, rgba(26, 15, 0, 0.97), rgba(42, 26, 10, 0.9));
  border-bottom: 3px solid rgba(245, 197, 66, 0.35);
  padding-top: env(safe-area-inset-top);
}
.hud-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
}
.hud-title {
  margin: 0;
  font-size: 22px;
  letter-spacing: 0.1em;
}
.rec-content {
  --background:
    radial-gradient(circle at 15% 10%, rgba(155, 89, 182, 0.14), transparent 45%),
    radial-gradient(circle at 50% 130%, #4a2c12, #120a00 75%);
}
.rec-scroll {
  padding: 14px 14px calc(110px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.summary {
  padding: 18px 16px 16px;
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 12px;
  align-items: center;
}
.sum-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.sum-label {
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: #d8b56a;
}
.sum-big {
  font-size: 34px;
  line-height: 1.1;
}
.sum-donut-wrap {
  position: relative;
  width: 110px;
  height: 110px;
}
.donut {
  width: 110px;
  height: 110px;
  transform: rotate(-90deg);
}
.donut-track {
  fill: none;
  stroke: rgba(0, 0, 0, 0.55);
  stroke-width: 11;
}
.donut-value {
  fill: none;
  stroke: url(#none);
  stroke: var(--ff-purple);
  stroke-width: 11;
  stroke-linecap: round;
  stroke-dasharray: 251.33;
  transition: stroke-dashoffset 1.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  filter: drop-shadow(0 0 6px rgba(155, 89, 182, 0.8));
}
.donut-center {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}
.donut-num {
  font-size: 17px;
}
.donut-cap {
  font-size: 9px;
  letter-spacing: 0.16em;
  opacity: 0.7;
}
.sum-cells {
  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.sum-cell {
  background: rgba(0, 0, 0, 0.35);
  border: 3px solid #120a00;
  border-radius: 12px;
  padding: 9px 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.sum-k {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  opacity: 0.7;
}
.sum-v {
  font-size: 19px;
  color: var(--ff-parchment);
}
.sum-v.net {
  color: var(--ff-green);
  text-shadow: 0 0 10px rgba(76, 175, 80, 0.5);
}
.section-head h2 {
  margin: 4px 0 2px;
  font-size: 18px;
  letter-spacing: 0.08em;
}
.muted {
  margin: 0;
  opacity: 0.6;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}
.cats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 13px;
}
@media (min-width: 560px) {
  .cats {
    grid-template-columns: repeat(3, 1fr);
  }
}
.cat {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 20px 10px 16px;
  min-height: 150px;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--accent) 22%, transparent), transparent 55%),
    linear-gradient(160deg, #5c3817 0%, #3a220d 60%, #241405 100%);
  border: 4px solid #120a00;
  border-radius: 18px;
  cursor: pointer;
  box-shadow:
    inset 0 3px 0 rgba(255, 220, 150, 0.3),
    inset 0 -8px 12px rgba(0, 0, 0, 0.5),
    0 7px 0 #0d0700,
    0 11px 20px rgba(0, 0, 0, 0.55);
  transition: transform 0.25s var(--ff-bounce);
  animation: bounce-in 0.5s var(--ff-bounce) both;
  color: var(--ff-parchment);
}
.cat::before {
  content: '';
  position: absolute;
  top: 0;
  left: 10%;
  right: 10%;
  height: 8px;
  background: linear-gradient(180deg, var(--accent), transparent);
  opacity: 0.7;
  border-radius: 0 0 8px 8px;
}
.cat:active {
  transform: translateY(6px);
  box-shadow:
    inset 0 3px 0 rgba(255, 220, 150, 0.2),
    0 1px 0 #0d0700;
}
.cat-hinge {
  position: absolute;
  top: 40%;
  width: 10px;
  height: 26px;
  background: linear-gradient(180deg, #c9a35b, #6b431f);
  border: 2px solid #120a00;
  border-radius: 3px;
}
.cat-hinge-l { left: -6px; }
.cat-hinge-r { right: -6px; }
.cat-icon {
  font-size: 44px;
  filter: drop-shadow(0 4px 2px rgba(0, 0, 0, 0.55));
}
.cat-label {
  font-size: 15px;
  text-shadow: 0 2px 0 #1a0f00;
}
.cat-total {
  font-size: 14px;
}
.cat-arrow {
  position: absolute;
  bottom: 8px;
  right: 12px;
  color: var(--accent);
  font-size: 16px;
  opacity: 0.9;
}
.rec-fab {
  margin-bottom: calc(76px + env(safe-area-inset-bottom));
}
.fab-glyph {
  font-size: 34px;
  font-family: var(--ff-font);
  line-height: 1;
}
</style>
