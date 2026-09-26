<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import {

  IonPage,
  IonContent,
  IonHeader,
  IonSpinner,
  IonIcon,
  onIonViewWillEnter
} from '@ionic/vue'
import { trophyOutline, settingsOutline } from 'ionicons/icons'
import ResourceBar from '@/components/game/ResourceBar.vue'
import XpBar from '@/components/game/XpBar.vue'
import GameModal from '@/components/game/GameModal.vue'
import GameButton from '@/components/game/GameButton.vue'
import FloatingNumber from '@/components/game/FloatingNumber.vue'
import SyncChip from '@/components/game/SyncChip.vue'
import { useGameStore } from '@/stores/gameStore'
import type { Building } from '@/types'

const router = useRouter()
const game = useGameStore()

interface Meta {
  icon: string
  label: string
  accent: string
}

const META: Record<string, Meta> = {
  bank: { icon: '⛏️', label: 'Gold Mine', accent: '#f5c542' },
  banks: { icon: '⛏️', label: 'Gold Mine', accent: '#f5c542' },
  asset: { icon: '🏰', label: 'Castle', accent: '#3498db' },
  assets: { icon: '🏰', label: 'Castle', accent: '#3498db' },
  'fixed-deposit': { icon: '🗄️', label: 'Vault', accent: '#4caf50' },
  fixed_deposit: { icon: '🗄️', label: 'Vault', accent: '#4caf50' },
  'fixed-deposits': { icon: '🗄️', label: 'Vault', accent: '#4caf50' },
  investment: { icon: '🔮', label: 'Wizard Tower', accent: '#d55cff' },
  investments: { icon: '🔮', label: 'Wizard Tower', accent: '#d55cff' },
  income: { icon: '🧪', label: 'Elixir Collector', accent: '#8be9fd' },
  incomes: { icon: '🧪', label: 'Elixir Collector', accent: '#8be9fd' },
  expense: { icon: '⚔️', label: 'Barracks', accent: '#e74c3c' },
  expenses: { icon: '⚔️', label: 'Barracks', accent: '#e74c3c' },
  'credit-card': { icon: '🧱', label: 'Wall', accent: '#9b59b6' },
  credit_card: { icon: '🧱', label: 'Wall', accent: '#9b59b6' },
  'credit-cards': { icon: '🧱', label: 'Wall', accent: '#9b59b6' }
}

function metaOf(b: Building): Meta {
  return META[b.type] ?? META[b.type?.toLowerCase?.() ?? ''] ?? { icon: '🏗️', label: b.name ?? 'Building', accent: '#c9a35b' }
}

const selected = ref<Building | null>(null)
const modalOpen = ref(false)
const busyId = ref<number | string | null>(null)
const error = ref('')
const floats = ref<{ id: number; text: string; x: number; y: number; color: string }[]>([])
let floatId = 0

const buildings = computed(() => game.buildings)
const loading = computed(() => game.loading && buildings.value.length === 0)

onIonViewWillEnter(() => {
  void game.fetchVillage()
})

function isReady(b: Building): boolean {
  return !!(b.production_ready || b.can_collect || b.ready)
}

function upgradeCost(b: Building): number {
  return Number(b.upgrade_cost ?? b.upgrade_gold_cost ?? 150 * Number(b.level ?? 1))
}

function openBuilding(b: Building, e?: MouseEvent) {
  selected.value = b
  modalOpen.value = true
  lastTap.value = e
    ? { x: e.clientX ?? (e as unknown as { pageX: number }).pageX, y: e.clientY ?? (e as unknown as { pageY: number }).pageY }
    : { x: window.innerWidth / 2, y: window.innerHeight / 2 }
}

const lastTap = ref({ x: 0, y: 0 })
const selectedUpgradeCost = computed(() => (selected.value ? upgradeCost(selected.value) : 0))

function spawnFloat(text: string, color = '#f5c542') {
  const id = floatId++
  const x = lastTap.value.x || window.innerWidth / 2
  const y = (lastTap.value.y || window.innerHeight / 2) - 20
  floats.value.push({ id, text, x, y, color })
}

function removeFloat(id: number) {
  floats.value = floats.value.filter((f) => f.id !== id)
}

async function collect() {
  if (!selected.value) return
  const b = selected.value
  busyId.value = b.id
  error.value = ''
  try {
    const gained = await game.collect(b.id)
    spawnFloat(`+${gained} 🪙`, '#f5c542')
    const idx = game.buildings.findIndex((x) => x.id === b.id)
    if (idx >= 0) selected.value = { ...game.buildings[idx] }
    modalOpen.value = false
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Collect failed'
  } finally {
    busyId.value = null
  }
}

async function upgrade() {
  if (!selected.value) return
  const b = selected.value
  busyId.value = b.id
  error.value = ''
  try {
    await game.upgrade(b.id)
    spawnFloat('⬆ LEVEL UP!', '#4caf50')
    const idx = game.buildings.findIndex((x) => x.id === b.id)
    if (idx >= 0) selected.value = { ...game.buildings[idx] }
    modalOpen.value = false
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Upgrade failed'
  } finally {
    busyId.value = null
  }
}
</script>

<template>
  <ion-page>
    <ion-header class="hud">
      <div class="hud-top">
        <ResourceBar :resources="game.resources">
          <SyncChip />
        </ResourceBar>
        <div class="hud-actions">
          <button class="hud-btn" type="button" aria-label="Leaderboard" @click="router.push('/leaderboard')">
            <ion-icon :icon="trophyOutline" />
          </button>
          <button class="hud-btn" type="button" aria-label="Settings" @click="router.push('/settings')">
            <ion-icon :icon="settingsOutline" />
          </button>
        </div>
      </div>
      <XpBar :level="game.level" :xp="game.xp" :xp-to-next="game.xpToNext" />
    </ion-header>

    <ion-content :fullscreen="true" class="village-content">
      <div class="village-scroll ff-hide-scrollbar">
        <div class="village-title">
          <h1 class="carved carved-gold">MY VILLAGE</h1>
          <p class="carved subtitle">Tap a building to collect &amp; upgrade</p>
        </div>

        <div v-if="loading" class="village-empty">
          <ion-spinner name="crescent" color="warning" />
          <p class="carved">Summoning village…</p>
        </div>
        <div v-else-if="!buildings.length" class="village-empty">
          <div class="empty-icon">🏚️</div>
          <p class="carved">No buildings yet — sync to summon your fortress!</p>
          <GameButton variant="gold" @click="game.fetchVillage()">RE-SUMMON</GameButton>
        </div>

        <div v-else class="bgrid">
          <button
            v-for="(b, i) in buildings"
            :key="b.id"
            class="btile stone-card"
            :style="{ animationDelay: `${i * 70}ms`, '--accent': metaOf(b).accent }"
            type="button"
            @click="openBuilding(b, $event)"
          >
            <span class="btile-level">Lv {{ b.level ?? 1 }}</span>
            <span v-if="isReady(b)" class="btile-bubble pulse-glow">🪙</span>
            <span class="btile-icon bob" :style="{ animationDelay: `${(i % 5) * 0.3}s` }">
              {{ metaOf(b).icon }}
            </span>
            <span class="btile-name carved clamp-1">{{ b.name || metaOf(b).label }}</span>
            <span class="btile-type carved carved-sm">{{ metaOf(b).label }}</span>
            <span class="btile-base" aria-hidden="true" />
          </button>
        </div>

        <div class="village-footer carved carved-sm">
          Troops: {{ game.troopsTotal }} · Battles: {{ game.battles.length }}
        </div>
      </div>

      <div class="float-layer">
        <FloatingNumber
          v-for="f in floats"
          :key="f.id"
          :text="f.text"
          :x="f.x"
          :y="f.y"
          :color="f.color"
          @done="removeFloat(f.id)"
        />
      </div>

      <GameModal v-model:show="modalOpen" :title="selected ? (selected.name || metaOf(selected).label) : ''">
        <template v-if="selected">
          <div class="binfo">
            <div class="binfo-icon">{{ metaOf(selected).icon }}</div>
            <div class="binfo-grid">
              <div class="binfo-cell">
                <span class="binfo-k carved carved-sm">Level</span>
                <span class="binfo-v carved carved-gold">{{ selected.level ?? 1 }}</span>
              </div>
              <div class="binfo-cell">
                <span class="binfo-k carved carved-sm">Kind</span>
                <span class="binfo-v carved">{{ metaOf(selected).label }}</span>
              </div>
              <div class="binfo-cell">
                <span class="binfo-k carved carved-sm">Status</span>
                <span class="binfo-v carved" :class="{ 'binfo-ready': isReady(selected) }">
                  {{ isReady(selected) ? 'Ready!' : 'Working…' }}
                </span>
              </div>
              <div class="binfo-cell">
                <span class="binfo-k carved carved-sm">Upgrade</span>
                <span class="binfo-v carved">🪙 {{ upgradeCost(selected) }}</span>
              </div>
            </div>
            <p class="binfo-flavor">
              {{ metaOf(selected).label }} generates loot from your
              {{ metaOf(selected).label.toLowerCase() }} records.
            </p>
            <p v-if="error" class="binfo-error carved carved-sm">{{ error }}</p>
          </div>
        </template>
        <template #footer>
          <GameButton variant="wood" size="sm" @click="modalOpen = false">DETAILS</GameButton>
          <GameButton
            variant="blue"
            size="sm"
            :disabled="busyId !== null || game.resources.gold < selectedUpgradeCost"
            @click="upgrade"
          >
            UPGRADE
          </GameButton>
          <GameButton
            v-if="selected && isReady(selected)"
            variant="gold"
            size="sm"
            :disabled="busyId !== null"
            sparkle
            @click="collect"
          >
            COLLECT
          </GameButton>
        </template>
      </GameModal>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.hud {
  position: relative;
  z-index: 20;
  background: linear-gradient(180deg, rgba(26, 15, 0, 0.97) 0%, rgba(42, 26, 10, 0.92) 80%, rgba(42, 26, 10, 0) 100%);
  border-bottom: 3px solid rgba(245, 197, 66, 0.35);
  padding-top: env(safe-area-inset-top);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.55);
}
.hud-top {
  display: flex;
  align-items: center;
}
.hud-actions {
  display: flex;
  gap: 6px;
  padding-right: 10px;
}
.hud-btn {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  border: 3px solid #0a0500;
  background: linear-gradient(180deg, #4a2c12, #2c1808);
  color: var(--ff-gold);
  font-size: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 3px 0 rgba(0, 0, 0, 0.6);
  transition: transform 0.2s var(--ff-bounce);
  padding: 0;
}
.hud-btn:active {
  transform: translateY(3px) scale(1.05);
  box-shadow: none;
}
.village-content {
  --background:
    radial-gradient(circle at 20% 15%, rgba(76, 175, 80, 0.12), transparent 40%),
    radial-gradient(circle at 80% 70%, rgba(245, 197, 66, 0.08), transparent 45%),
    radial-gradient(circle at 50% 120%, #3e2712, #1a0f00 70%);
}
.village-scroll {
  padding: 14px 14px calc(28px + env(safe-area-inset-bottom));
}
.village-title {
  text-align: center;
  margin-bottom: 14px;
}
.village-title h1 {
  margin: 0;
  font-size: 26px;
  letter-spacing: 0.08em;
}
.subtitle {
  margin: 2px 0 0;
  font-size: 12px;
  opacity: 0.7;
  text-transform: uppercase;
  letter-spacing: 0.12em;
}
.village-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 50px 20px;
  text-align: center;
}
.empty-icon {
  font-size: 64px;
  animation: bob 2.5s ease-in-out infinite;
}
.bgrid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 14px;
}
@media (min-width: 560px) {
  .bgrid {
    grid-template-columns: repeat(3, 1fr);
  }
}
.btile {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 30px 10px 16px;
  min-height: 170px;
  background:
    radial-gradient(circle at 50% 0%, color-mix(in srgb, var(--accent) 28%, transparent), transparent 65%),
    linear-gradient(160deg, #4a3218 0%, #33200c 55%, #241405 100%);
  border: 4px solid #120a00;
  border-radius: 20px;
  cursor: pointer;
  box-shadow:
    inset 0 3px 0 rgba(255, 220, 150, 0.25),
    inset 0 -8px 14px rgba(0, 0, 0, 0.55),
    0 7px 0 #0d0700,
    0 12px 22px rgba(0, 0, 0, 0.6);
  transition:
    transform 0.25s var(--ff-bounce),
    box-shadow 0.25s ease;
  animation: bounce-in 0.5s var(--ff-bounce) both;
  overflow: visible;
}
.btile::before {
  content: '';
  position: absolute;
  inset: 6px;
  border: 2px solid color-mix(in srgb, var(--accent) 45%, transparent);
  border-radius: 14px;
  pointer-events: none;
}
.btile:active {
  transform: translateY(6px) scale(0.98);
  box-shadow:
    inset 0 3px 0 rgba(255, 220, 150, 0.2),
    0 2px 0 #0d0700,
    0 4px 10px rgba(0, 0, 0, 0.5);
}
.btile-level {
  position: absolute;
  top: 8px;
  left: 8px;
  background: linear-gradient(180deg, var(--ff-gold), var(--ff-gold-dark));
  color: var(--ff-brown-deep);
  font-family: var(--ff-font);
  font-size: 11px;
  padding: 3px 9px;
  border-radius: 999px;
  border: 2px solid #120a00;
  box-shadow: 0 2px 0 rgba(0, 0, 0, 0.5);
  z-index: 2;
}
.btile-bubble {
  position: absolute;
  top: -12px;
  right: -8px;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: radial-gradient(circle at 35% 30%, #fff6d0, var(--ff-gold) 55%, var(--ff-gold-dark));
  border: 3px solid #120a00;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  z-index: 3;
  box-shadow: 0 4px 0 rgba(0, 0, 0, 0.5);
}
.btile-icon {
  font-size: 54px;
  line-height: 1;
  filter: drop-shadow(0 5px 2px rgba(0, 0, 0, 0.55));
  margin-top: 4px;
}
.btile-name {
  font-family: var(--ff-font);
  font-size: 15px;
  max-width: 100%;
  color: var(--ff-parchment);
  text-shadow: 0 2px 0 #1a0f00;
}
.btile-type {
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  text-shadow: 0 1px 0 #000;
}
.btile-base {
  position: absolute;
  bottom: -10px;
  left: 18%;
  right: 18%;
  height: 12px;
  background: radial-gradient(ellipse, rgba(0, 0, 0, 0.55), transparent 70%);
  border-radius: 50%;
  filter: blur(2px);
}
.float-layer {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 999;
}
.village-footer {
  text-align: center;
  margin-top: 22px;
  opacity: 0.6;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}
.binfo {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 4px 0 8px;
}
.binfo-icon {
  font-size: 72px;
  animation: bob 2.2s ease-in-out infinite;
  filter: drop-shadow(0 6px 3px rgba(0, 0, 0, 0.5));
}
.binfo-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  width: 100%;
}
.binfo-cell {
  background: rgba(0, 0, 0, 0.35);
  border: 3px solid #120a00;
  border-radius: 12px;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.binfo-k {
  opacity: 0.65;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 10px;
}
.binfo-v {
  font-size: 16px;
}
.binfo-ready {
  color: var(--ff-green);
  text-shadow: 0 0 10px rgba(76, 175, 80, 0.7);
}
.binfo-flavor {
  margin: 0;
  text-align: center;
  font-size: 13px;
  opacity: 0.75;
  line-height: 1.45;
}
.binfo-error {
  color: #ff9d94;
  margin: 0;
  text-align: center;
}
</style>
