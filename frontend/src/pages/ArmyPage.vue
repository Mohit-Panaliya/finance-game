<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { onIonViewWillEnter } from '@ionic/vue'
import { IonPage, IonContent, IonHeader } from '@ionic/vue'
import ResourceBar from '@/components/game/ResourceBar.vue'
import GameButton from '@/components/game/GameButton.vue'
import GameSelect from '@/components/game/GameSelect.vue'
import XpBar from '@/components/game/XpBar.vue'
import SyncChip from '@/components/game/SyncChip.vue'
import { useGameStore } from '@/stores/gameStore'

interface CatalogEntry {
  type: string
  name: string
  icon: string
  cost: number
  attack: number
  hp: number
}

const CATALOG: CatalogEntry[] = [
  { type: 'barbarian', name: 'Barbarian', icon: '🪓', cost: 25, attack: 12, hp: 80 },
  { type: 'archer', name: 'Archer', icon: '🏹', cost: 50, attack: 22, hp: 45 },
  { type: 'giant', name: 'Giant', icon: '🛡️', cost: 150, attack: 30, hp: 320 },
  { type: 'wizard', name: 'Wizard', icon: '🧙', cost: 300, attack: 55, hp: 120 },
  { type: 'dragon', name: 'Dragon', icon: '🐉', cost: 800, attack: 120, hp: 600 },
  { type: 'goblin', name: 'Goblin', icon: '👺', cost: 20, attack: 18, hp: 40 }
]

const game = useGameStore()

const selectedType = ref('barbarian')
const count = ref(3)
const error = ref('')
const training = ref<{ type: string; name: string; count: number; pct: number } | null>(null)
let raf = 0

const typeOptions = CATALOG.map((c) => ({ value: c.type, label: `${c.name} — ${c.cost} ⚗️`, icon: c.icon }))

const selected = computed(() => CATALOG.find((c) => c.type === selectedType.value) ?? CATALOG[0])
const totalCost = computed(() => selected.value.cost * count.value)
const elixir = computed(() => game.resources.elixir)

const roster = computed(() =>
  CATALOG.map((c) => {
    const t = game.troops.find((x) => (x.troop_type ?? x.type) === c.type)
    return {
      ...c,
      count: Number(t?.count ?? 0),
      level: Number(t?.level ?? 1)
    }
  })
)

onIonViewWillEnter(() => {
  void game.fetchVillage()
})

function bump(n: number) {
  count.value = Math.max(1, Math.min(50, count.value + n))
}

function train() {
  if (training.value) return
  error.value = ''
  if (totalCost.value > elixir.value) {
    error.value = 'Not enough elixir — collect more!'
    return
  }
  const entry = selected.value
  training.value = { type: entry.type, name: entry.name, count: count.value, pct: 0 }
  const duration = 1300 + count.value * 220
  const start = performance.now()
  const step = (now: number) => {
    if (!training.value) return
    const t = Math.min(1, (now - start) / duration)
    training.value.pct = Math.round(t * 100)
    if (t < 1) {
      raf = requestAnimationFrame(step)
    } else {
      const tr = training.value
      training.value = null
      void game.trainTroop(tr.type, tr.count).catch((e) => {
        error.value = e instanceof Error ? e.message : 'Training failed'
      })
    }
  }
  raf = requestAnimationFrame(step)
}

function pips(level: number): boolean[] {
  return Array.from({ length: 5 }, (_, i) => i < Math.min(5, level))
}

function fmt(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 10_000) return `${(n / 1000).toFixed(1)}K`
  return String(Math.round(n))
}

onBeforeUnmount(() => cancelAnimationFrame(raf))
</script>

<template>
  <ion-page>
    <ion-header class="hud">
      <ResourceBar :resources="game.resources" compact>
        <SyncChip />
      </ResourceBar>
      <XpBar :level="game.level" :xp="game.xp" :xp-to-next="game.xpToNext" />
    </ion-header>

    <ion-content :fullscreen="true" class="army-content">
      <div class="army-scroll ff-hide-scrollbar">
        <div class="heading">
          <h1 class="carved carved-gold">BARRACKS</h1>
          <p class="carved sub">Train troops from your expense ledger</p>
        </div>

        <!-- ============ TRAIN PANEL ============ -->
        <div class="panel-wood noise train">
          <h2 class="carved carved-gold train-title">TRAINING GROUNDS</h2>

          <GameSelect v-model="selectedType" label="Troop Type" :options="typeOptions" />

          <div class="stepper-row">
            <span class="carved carved-sm stepper-label">Count</span>
            <div class="stepper">
              <button class="step-btn" type="button" @click="bump(-1)">−</button>
              <span class="step-count carved carved-gold">{{ count }}</span>
              <button class="step-btn" type="button" @click="bump(1)">+</button>
            </div>
            <span class="carved carved-sm cost">⚗️ {{ fmt(totalCost) }}</span>
          </div>

          <div v-if="training" class="train-progress">
            <div class="tp-head carved carved-sm">
              {{ training.count }}× {{ training.name }} training…
              <span class="tp-pct">{{ training.pct }}%</span>
            </div>
            <div class="tp-track">
              <div class="tp-fill" :style="{ width: `${training.pct}%` }">
                <div class="tp-shimmer" />
              </div>
            </div>
          </div>

          <p v-if="error" class="army-error carved carved-sm">{{ error }}</p>

          <GameButton
            variant="green"
            size="lg"
            block
            sparkle
            :disabled="!!training || totalCost > elixir"
            @click="train"
          >
            {{ training ? 'TRAINING…' : `TRAIN ${count}× ${selected.name}` }}
          </GameButton>
          <div class="elixir-note carved carved-sm">Elixir on hand: ⚗️ {{ fmt(elixir) }}</div>
        </div>

        <!-- ============ ROSTER ============ -->
        <div class="section-head">
          <h2 class="carved carved-gold">YOUR ARMY</h2>
          <span class="carved carved-sm total">Total: {{ game.troopsTotal }}</span>
        </div>

        <div class="roster">
          <div
            v-for="(t, i) in roster"
            :key="t.type"
            class="troop-card stone-card"
            :class="{ 'troop-empty': t.count === 0 }"
            :style="{ animationDelay: `${i * 60}ms` }"
          >
            <div class="troop-top">
              <span class="troop-icon bob" :style="{ animationDelay: `${(i % 5) * 0.25}s` }">{{ t.icon }}</span>
              <div class="troop-id">
                <span class="carved troop-name">{{ t.name }}</span>
                <span class="pips">
                  <i v-for="(on, pi) in pips(t.level)" :key="pi" class="pip" :class="{ on }" />
                </span>
              </div>
              <span class="troop-count carved carved-gold">×{{ t.count }}</span>
            </div>
            <div class="stat">
              <span class="stat-k carved carved-sm">⚔️ ATK</span>
              <div class="stat-track"><div class="stat-fill atk" :style="{ width: `${Math.min(100, (t.attack / 120) * 100)}%` }" /></div>
              <span class="stat-v carved carved-sm">{{ t.attack }}</span>
            </div>
            <div class="stat">
              <span class="stat-k carved carved-sm">❤️ HP</span>
              <div class="stat-track"><div class="stat-fill hp" :style="{ width: `${Math.min(100, (t.hp / 600) * 100)}%` }" /></div>
              <span class="stat-v carved carved-sm">{{ t.hp }}</span>
            </div>
            <div class="troop-cost carved carved-sm">⚗️ {{ t.cost }} each</div>
          </div>
        </div>
      </div>
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
.army-content {
  --background:
    radial-gradient(circle at 20% 5%, rgba(231, 76, 60, 0.14), transparent 45%),
    radial-gradient(circle at 50% 130%, #4a2c12, #120a00 75%);
}
.army-scroll {
  padding: 14px 14px calc(28px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.heading {
  text-align: center;
}
.heading h1 {
  margin: 0;
  font-size: 25px;
  letter-spacing: 0.08em;
}
.sub {
  margin: 2px 0 0;
  font-size: 12px;
  opacity: 0.7;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}
.train {
  padding: 16px 14px;
  display: flex;
  flex-direction: column;
  gap: 13px;
}
.train-title {
  margin: 0;
  font-size: 17px;
  text-align: center;
  letter-spacing: 0.1em;
}
.stepper-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.stepper-label {
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: #d8b56a;
}
.stepper {
  display: flex;
  align-items: center;
  gap: 8px;
}
.step-btn {
  width: 46px;
  height: 46px;
  border-radius: 14px;
  border: 4px solid #120a00;
  background: linear-gradient(180deg, #ffe27a, var(--ff-gold) 55%, var(--ff-gold-dark));
  color: var(--ff-brown-deep);
  font-size: 26px;
  font-weight: 900;
  line-height: 1;
  cursor: pointer;
  box-shadow: 0 5px 0 #5c3c00;
  transition: transform 0.15s var(--ff-bounce);
  padding: 0;
}
.step-btn:active {
  transform: translateY(5px);
  box-shadow: none;
}
.step-count {
  min-width: 56px;
  text-align: center;
  font-size: 26px;
}
.cost {
  color: var(--ff-elixir);
  font-size: 14px;
}
.train-progress {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.tp-head {
  display: flex;
  justify-content: space-between;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #d8b56a;
  font-size: 11px;
}
.tp-pct {
  color: var(--ff-green);
}
.tp-track {
  position: relative;
  height: 24px;
  border: 3px solid #120a00;
  border-radius: 999px;
  background: linear-gradient(180deg, #1c1004, #0d0700);
  box-shadow: inset 0 3px 8px rgba(0, 0, 0, 0.8);
  overflow: hidden;
}
.tp-fill {
  position: absolute;
  inset: 0 auto 0 0;
  background: linear-gradient(180deg, #8ee08f, var(--ff-green) 55%, var(--ff-green-dark));
  box-shadow: inset 0 2px 0 rgba(255, 255, 255, 0.5);
  border-radius: 999px;
  transition: width 0.15s linear;
  overflow: hidden;
}
.tp-shimmer {
  position: absolute;
  inset: 0;
  background: linear-gradient(100deg, transparent 25%, rgba(255, 255, 255, 0.55) 50%, transparent 75%);
  background-size: 200% 100%;
  animation: shimmer 1.3s linear infinite;
}
.army-error {
  color: #ff9d94;
  text-align: center;
  margin: 0;
}
.elixir-note {
  text-align: center;
  opacity: 0.7;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 11px;
}
.section-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}
.section-head h2 {
  margin: 0;
  font-size: 18px;
  letter-spacing: 0.08em;
}
.total {
  opacity: 0.7;
}
.roster {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}
.troop-card {
  padding: 14px;
  background: linear-gradient(160deg, #4a3218, #241405);
  border: 4px solid #120a00;
  border-radius: 16px;
  box-shadow:
    inset 0 2px 0 rgba(255, 220, 150, 0.22),
    0 6px 0 #0d0700,
    0 10px 18px rgba(0, 0, 0, 0.55);
  display: flex;
  flex-direction: column;
  gap: 9px;
  animation: bounce-in 0.5s var(--ff-bounce) both;
}
.troop-empty {
  opacity: 0.55;
  filter: grayscale(0.4);
}
.troop-top {
  display: flex;
  align-items: center;
  gap: 10px;
}
.troop-icon {
  font-size: 36px;
  filter: drop-shadow(0 3px 1px rgba(0, 0, 0, 0.55));
}
.troop-id {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.troop-name {
  font-size: 16px;
}
.pips {
  display: flex;
  gap: 4px;
}
.pip {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.55);
  border: 2px solid #120a00;
}
.pip.on {
  background: radial-gradient(circle at 35% 30%, #fff6d0, var(--ff-gold));
  box-shadow: 0 0 6px rgba(245, 197, 66, 0.9);
}
.troop-count {
  font-size: 20px;
}
.stat {
  display: flex;
  align-items: center;
  gap: 8px;
}
.stat-k {
  width: 74px;
  font-size: 10px;
  letter-spacing: 0.06em;
  opacity: 0.85;
}
.stat-track {
  flex: 1;
  height: 14px;
  background: rgba(0, 0, 0, 0.55);
  border: 2px solid #120a00;
  border-radius: 999px;
  overflow: hidden;
}
.stat-fill {
  height: 100%;
  border-radius: 999px;
  transition: width 0.8s var(--ff-bounce);
}
.stat-fill.atk {
  background: linear-gradient(90deg, #ff9d94, var(--ff-red));
  box-shadow: 0 0 8px rgba(231, 76, 60, 0.7);
}
.stat-fill.hp {
  background: linear-gradient(90deg, #8ee08f, var(--ff-green));
  box-shadow: 0 0 8px rgba(76, 175, 80, 0.7);
}
.stat-v {
  width: 38px;
  text-align: right;
  font-size: 12px;
}
.troop-cost {
  text-align: right;
  opacity: 0.7;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
</style>
