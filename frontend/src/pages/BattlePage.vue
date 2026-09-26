<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { useRouter } from 'vue-router'
import {
 IonPage, IonContent, IonHeader,
  onIonViewWillEnter
} from '@ionic/vue'
import ResourceBar from '@/components/game/ResourceBar.vue'
import GameButton from '@/components/game/GameButton.vue'
import GameModal from '@/components/game/GameModal.vue'
import GameSelect from '@/components/game/GameSelect.vue'
import XpBar from '@/components/game/XpBar.vue'
import ParticleBurst from '@/components/game/ParticleBurst.vue'
import { useGameStore } from '@/stores/gameStore'
import type { Troop } from '@/types'

const router = useRouter()
const game = useGameStore()

const battleType = ref('raid')
const battleTypeOptions = [
  { value: 'raid', label: 'Raid — loot gold', icon: '🏴‍☠️' },
  { value: 'war', label: 'War — crush base', icon: '⚔️' },
  { value: 'farming', label: 'Farming — easy prey', icon: '🌾' }
]

const squad = ref<Record<string, number>>({})
const phase = ref<'idle' | 'fighting' | 'result'>('idle')
const stageShake = ref<boolean[]>([false, false, false, false])
const stageGone = ref<boolean[]>([false, false, false, false])
const starsShown = ref(0)
const displayLoot = ref(0)
const result = ref<{ result: 'victory' | 'defeat'; stars: number; loot: number; simulated?: boolean } | null>(null)
const burstActive = ref(false)
const battleError = ref('')
const timers: number[] = []

const enemyBase = [
  { icon: '🏰', name: 'Keep' },
  { icon: '🏦', name: 'Treasury' },
  { icon: '🔮', name: 'Tower' },
  { icon: '🗄️', name: 'Vault' }
]

const availableTroops = computed<Troop[]>(() => game.troops)
const squadCount = computed(() => Object.values(squad.value).reduce((a, b) => a + b, 0))
const squadIds = computed(() => {
  const ids: Array<number | string> = []
  for (const t of availableTroops.value) {
    const type = t.troop_type ?? t.type ?? ''
    const n = squad.value[type] ?? 0
    for (let i = 0; i < n; i++) ids.push(t.id ?? type)
  }
  return ids
})

onIonViewWillEnter(() => {
  void game.fetchVillage()
  void game.fetchBattles()
})

function troopIcon(type: string): string {
  const map: Record<string, string> = {
    barbarian: '🪓',
    archer: '🏹',
    giant: '🛡️',
    wizard: '🧙',
    dragon: '🐉',
    goblin: '👺'
  }
  return map[type] ?? '⚔️'
}

function troopCount(t: Troop): number {
  return Number(t.count ?? 0)
}

function selectedOf(t: Troop): number {
  return squad.value[t.troop_type ?? t.type ?? ''] ?? 0
}

function bump(t: Troop, delta: number) {
  const type = t.troop_type ?? t.type ?? ''
  const max = troopCount(t)
  const next = Math.max(0, Math.min(max, (squad.value[type] ?? 0) + delta))
  if (next === 0) delete squad.value[type]
  else squad.value[type] = next
}

function clearTimers() {
  while (timers.length) window.clearTimeout(timers.pop())
}

function later(fn: () => void, ms: number) {
  timers.push(window.setTimeout(fn, ms))
}

function countUp(to: number, ms: number) {
  const start = performance.now()
  const step = (now: number) => {
    const t = Math.min(1, (now - start) / ms)
    displayLoot.value = Math.round(to * (1 - Math.pow(1 - t, 3)))
    if (t < 1) requestAnimationFrame(step)
  }
  requestAnimationFrame(step)
}

async function attack() {
  if (phase.value === 'fighting') return
  if (squadCount.value === 0) {
    battleError.value = 'Pick troops for your squad first!'
    return
  }
  battleError.value = ''
  phase.value = 'fighting'
  stageShake.value = [false, false, false, false]
  stageGone.value = [false, false, false, false]
  starsShown.value = 0
  displayLoot.value = 0
  result.value = null

  let outcome: { result: 'victory' | 'defeat'; stars: number; loot: number; simulated?: boolean } | null = null
  try {
    outcome = await game.attack(battleType.value, squadIds.value)
  } catch (e) {
    battleError.value = e instanceof Error ? e.message : 'Battle failed'
    phase.value = 'idle'
    return
  }

  const stars = outcome?.stars ?? 0
  const loot = outcome?.loot ?? 0

  const doomed = Math.min(4, stars + (stars > 0 ? 1 : 0))
  for (let i = 0; i < doomed; i++) {
    later(() => {
      stageShake.value[i] = true
    }, 700 + i * 520)
    later(() => {
      stageGone.value[i] = true
      stageShake.value[i] = false
    }, 1150 + i * 520)
  }

  for (let s = 1; s <= stars; s++) {
    later(() => {
      starsShown.value = s
      if (s === stars) {
        burstActive.value = false
        requestAnimationFrame(() => {
          burstActive.value = true
        })
      }
    }, 1300 + s * 750)
  }

  later(() => countUp(loot, 1600), 900)

  later(() => {
    result.value = outcome
      ? { result: outcome.result, stars, loot, simulated: outcome.simulated }
      : { result: stars > 0 ? 'victory' : 'defeat', stars, loot, simulated: false }
    phase.value = 'result'
    burstActive.value = false
  }, 3600 + stars * 400)
}

function resetStage() {
  clearTimers()
  phase.value = 'idle'
  result.value = null
  stageShake.value = [false, false, false, false]
  stageGone.value = [false, false, false, false]
  starsShown.value = 0
  displayLoot.value = 0
  squad.value = {}
}

function starCount(b: { stars?: number }): number {
  return Math.max(0, Math.min(3, Number(b.stars ?? 0)))
}

onBeforeUnmount(clearTimers)
</script>

<template>
  <ion-page>
    <ion-header class="hud">
      <ResourceBar :resources="game.resources" compact />
      <XpBar :level="game.level" :xp="game.xp" :xp-to-next="game.xpToNext" />
    </ion-header>

    <ion-content :fullscreen="true" class="battle-content">
      <div class="battle-scroll ff-hide-scrollbar">
        <div class="heading">
          <h1 class="carved carved-gold">RAID THE LEDGER</h1>
          <p class="carved sub">Assemble a squad, smash the enemy vaults</p>
        </div>

        <div class="panel-stone noise setup">
          <div class="setup-row">
            <GameSelect
              v-model="battleType"
              label="Battle Type"
              :options="battleTypeOptions"
            />
          </div>

          <div class="squad-head carved carved-sm">YOUR SQUAD — tap to add</div>

          <div v-if="!availableTroops.length" class="squad-empty carved">
            No troops trained yet.
            <button class="inline-link" type="button" @click="router.push('/army')">Visit Barracks →</button>
          </div>

          <div v-else class="squad-grid">
            <button
              v-for="t in availableTroops"
              :key="String(t.id ?? t.troop_type ?? t.type)"
              class="squad-card"
              :class="{ 'squad-on': selectedOf(t) > 0 }"
              type="button"
              :disabled="troopCount(t) === 0"
              @click="bump(t, 1)"
              @contextmenu.prevent="bump(t, -1)"
            >
              <span class="squad-icon">{{ troopIcon(t.troop_type ?? t.type ?? '') }}</span>
              <span class="squad-name carved">{{ t.name ?? t.troop_type ?? t.type }}</span>
              <span class="squad-count carved carved-sm">×{{ troopCount(t) }}</span>
              <span v-if="selectedOf(t) > 0" class="squad-sel">✦ {{ selectedOf(t) }}</span>
              <span class="squad-minus" @click.stop="bump(t, -1)">−</span>
            </button>
          </div>

          <p v-if="battleError" class="b-error carved carved-sm">{{ battleError }}</p>

          <GameButton
            variant="red"
            size="lg"
            block
            sparkle
            :disabled="phase === 'fighting' || squadCount === 0"
            @click="attack"
          >
            {{ phase === 'fighting' ? 'BATTLE RAGES…' : `ATTACK! (${squadCount} troops)` }}
          </GameButton>
        </div>

        <!-- ===================== BATTLE STAGE ===================== -->
        <div class="stage panel-wood noise" :class="{ 'stage-active': phase !== 'idle' }">
          <div class="stage-sky" aria-hidden="true">
            <span class="cloud c1">☁️</span>
            <span class="cloud c2">☁️</span>
          </div>

          <div class="stars-row">
            <span
              v-for="n in 3"
              :key="n"
              class="bstar"
              :class="{ on: n <= starsShown }"
              :style="{ animationDelay: `${(n - 1) * 0.12}s` }"
            >★</span>
            <ParticleBurst :active="burstActive" :count="26" origin-x="50%" origin-y="40%" />
          </div>

          <div class="loot-hud carved carved-gold">🪙 {{ displayLoot.toLocaleString() }}</div>

          <div class="enemy-row">
            <div
              v-for="(b, i) in enemyBase"
              :key="b.name"
              class="enemy-tile"
              :class="{ shaking: stageShake[i], gone: stageGone[i] }"
              :style="{ animationDelay: `${i * 0.1}s` }"
            >
              <span class="enemy-icon">{{ b.icon }}</span>
              <span class="enemy-name carved carved-sm">{{ b.name }}</span>
              <span v-if="stageGone[i]" class="enemy-boom">💥</span>
            </div>
          </div>

          <div class="ground" aria-hidden="true" />

          <div v-if="phase === 'fighting'" class="march">
            <span
              v-for="n in Math.min(8, Math.max(1, squadCount))"
              :key="n"
              class="trooper"
              :style="{ bottom: `${18 + (n % 3) * 16}px`, animationDelay: `${n * 0.18}s`, '--walk-x': `${52 + n * 7}%` }"
            >⚔️</span>
          </div>

          <div v-if="phase === 'idle'" class="stage-hint carved carved-sm">
            Troops appear here during a raid
          </div>
        </div>

        <!-- ===================== HISTORY ===================== -->
        <div class="panel-stone noise history">
          <h2 class="carved carved-gold section-title">BATTLE LOG</h2>
          <div v-if="!game.battles.length" class="hist-empty carved carved-sm">
            No raids recorded yet — first blood awaits!
          </div>
          <div
            v-for="(b, i) in game.battles.slice(0, 12)"
            :key="b.id"
            class="hist-row"
            :style="{ animationDelay: `${i * 50}ms` }"
          >
            <span class="hist-result" :class="String(b.result ?? 'victory').includes('defeat') ? 'loss' : 'win'">
              {{ String(b.result ?? 'victory').includes('defeat') ? '💀' : '🏆' }}
            </span>
            <div class="hist-mid">
              <span class="carved clamp-1">{{ b.battle_type ?? 'raid' }}</span>
              <span class="carved carved-sm hist-loot">+{{ Number(b.loot ?? b.gold_looted ?? 0) }} 🪙</span>
            </div>
            <span class="hist-stars">
              <span v-for="n in 3" :key="n" class="hist-star" :class="{ on: n <= starCount(b) }">★</span>
            </span>
          </div>
        </div>
      </div>

      <!-- ===================== RESULT CHEST ===================== -->
      <GameModal
        :show="phase === 'result' && !!result"
        :title="result?.result === 'victory' ? 'VICTORY!' : 'DEFEAT'"
        @update:show="(v) => { if (!v) resetStage() }"
      >
        <div class="result">
          <div class="result-chest" :class="result?.result">
            <span class="result-chest-icon">{{ result?.result === 'victory' ? '🎁' : '🪦' }}</span>
          </div>
          <div class="result-stars">
            <span
              v-for="n in 3"
              :key="n"
              class="result-star"
              :class="{ on: n <= (result?.stars ?? 0) }"
              :style="{ animationDelay: `${n * 0.28}s` }"
            >★</span>
          </div>
          <div class="result-loot carved carved-gold">
            +{{ (result?.loot ?? 0).toLocaleString() }} 🪙 looted
          </div>
          <div class="result-sub carved carved-sm">
            {{ result?.simulated ? 'offline raid (queued for sync)' : 'server-confirmed raid' }}
          </div>
        </div>
        <template #footer>
          <GameButton variant="gold" size="md" block sparkle @click="resetStage">CLAIM &amp; RESET</GameButton>
        </template>
      </GameModal>
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
.battle-content {
  --background:
    radial-gradient(circle at 80% 10%, rgba(231, 76, 60, 0.14), transparent 45%),
    radial-gradient(circle at 50% 120%, #4a2c12, #120a00 75%);
}
.battle-scroll {
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
  font-size: 24px;
  letter-spacing: 0.06em;
}
.sub {
  margin: 2px 0 0;
  font-size: 12px;
  opacity: 0.7;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}
.setup {
  padding: 16px 14px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.squad-head {
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: #d8b56a;
}
.squad-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 10px;
}
.squad-card {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 12px 8px 14px;
  background: linear-gradient(180deg, #4a3218, #241405);
  border: 3px solid #120a00;
  border-radius: 14px;
  cursor: pointer;
  box-shadow:
    inset 0 2px 0 rgba(255, 220, 150, 0.2),
    0 4px 0 #0d0700;
  transition:
    transform 0.2s var(--ff-bounce),
    border-color 0.2s ease,
    box-shadow 0.2s ease;
  color: var(--ff-parchment);
  font-family: var(--ff-font-body);
}
.squad-card:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.squad-card:active {
  transform: translateY(4px);
  box-shadow: 0 0 0 #0d0700;
}
.squad-on {
  border-color: var(--ff-gold);
  box-shadow:
    inset 0 0 16px rgba(245, 197, 66, 0.3),
    0 0 16px rgba(245, 197, 66, 0.5),
    0 4px 0 #0d0700;
}
.squad-icon {
  font-size: 32px;
}
.squad-name {
  font-size: 13px;
}
.squad-count {
  opacity: 0.7;
}
.squad-sel {
  position: absolute;
  top: -8px;
  right: -6px;
  background: linear-gradient(180deg, var(--ff-gold), var(--ff-gold-dark));
  color: var(--ff-brown-deep);
  font-family: var(--ff-font);
  font-size: 11px;
  border: 2px solid #120a00;
  border-radius: 999px;
  padding: 2px 7px;
  animation: bounce-in 0.35s var(--ff-bounce) both;
}
.squad-minus {
  position: absolute;
  bottom: -6px;
  right: -6px;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: linear-gradient(180deg, #ff9d94, var(--ff-red));
  border: 2px solid #120a00;
  color: #fff;
  font-weight: 900;
  font-size: 16px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 3px 0 var(--ff-red-dark);
}
.squad-empty {
  text-align: center;
  opacity: 0.8;
  padding: 8px;
}
.inline-link {
  background: none;
  border: none;
  color: var(--ff-gold);
  font-family: var(--ff-font);
  font-size: 15px;
  cursor: pointer;
  text-decoration: underline;
}
.b-error {
  color: #ff9d94;
  text-align: center;
  margin: 0;
}
.stage {
  position: relative;
  min-height: 300px;
  overflow: hidden;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.stage-active {
  border-color: var(--ff-gold-dark);
}
.stage-sky {
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, #1a2a4a 0%, #2a1a0a 75%);
  opacity: 0.55;
  pointer-events: none;
}
.cloud {
  position: absolute;
  font-size: 34px;
  opacity: 0.5;
  animation: drift 22s linear infinite;
}
.c1 { top: 8%; left: -10%; }
.c2 { top: 22%; left: 40%; animation-duration: 30s; animation-delay: -8s; }
@keyframes drift {
  from { transform: translateX(0); }
  to { transform: translateX(460px); }
}
.stars-row {
  position: relative;
  display: flex;
  justify-content: center;
  gap: 14px;
  z-index: 3;
  min-height: 44px;
}
.bstar {
  font-size: 40px;
  color: rgba(255, 255, 255, 0.15);
  text-shadow: 0 3px 0 rgba(0, 0, 0, 0.5);
  transform: scale(0.8);
  transition: all 0.35s var(--ff-bounce);
}
.bstar.on {
  color: var(--ff-gold);
  text-shadow:
    0 2px 0 #8a5a00,
    0 0 22px rgba(245, 197, 66, 0.95);
  animation: pop-star 0.55s var(--ff-bounce) both;
}
.loot-hud {
  position: absolute;
  top: 62px;
  right: 14px;
  z-index: 3;
  font-size: 22px;
}
.enemy-row {
  position: relative;
  z-index: 2;
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  margin-top: 40px;
}
.enemy-tile {
  position: relative;
  background:
    radial-gradient(circle at 50% 20%, rgba(245, 197, 66, 0.15), transparent 60%),
    linear-gradient(180deg, #4a3218, #241405);
  border: 3px solid #120a00;
  border-radius: 12px;
  padding: 12px 6px 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  box-shadow: 0 4px 0 #0d0700;
  transform-origin: bottom center;
}
.enemy-icon {
  font-size: 34px;
}
.enemy-name {
  font-size: 11px;
  opacity: 0.85;
}
.enemy-tile.shaking {
  animation: shake 0.45s ease-in-out infinite;
}
.enemy-tile.gone {
  animation: explode 0.6s ease-out forwards;
}
.enemy-boom {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 44px;
  z-index: 4;
  animation: bounce-in 0.4s var(--ff-bounce) both;
}
.ground {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 56px;
  background:
    repeating-linear-gradient(90deg, rgba(0, 0, 0, 0.18) 0 4px, transparent 4px 40px),
    linear-gradient(180deg, #5c8a3a, #33551f 60%, #244016);
  border-top: 4px solid #120a00;
  z-index: 1;
}
.march {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 70px;
  z-index: 4;
  overflow: hidden;
}
.trooper {
  position: absolute;
  left: 0;
  font-size: 26px;
  filter: drop-shadow(0 3px 1px rgba(0, 0, 0, 0.6));
  animation: walk-right 2.6s cubic-bezier(0.4, 0, 0.6, 1) forwards;
}
.stage-hint {
  position: absolute;
  bottom: 20px;
  left: 0;
  right: 0;
  text-align: center;
  z-index: 2;
  opacity: 0.65;
}
.history {
  padding: 16px 14px;
}
.section-title {
  margin: 0 0 12px;
  font-size: 17px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
.hist-empty {
  text-align: center;
  opacity: 0.65;
  padding: 10px;
}
.hist-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), transparent);
  border: 3px solid #120a00;
  border-radius: 12px;
  margin-bottom: 8px;
  box-shadow: 0 3px 0 rgba(0, 0, 0, 0.5);
  animation: bounce-in 0.4s var(--ff-bounce) both;
}
.hist-result {
  font-size: 24px;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  border: 2px solid #120a00;
  background: rgba(0, 0, 0, 0.35);
}
.hist-mid {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  font-family: var(--ff-font);
  font-size: 14px;
  color: var(--ff-parchment);
}
.hist-loot {
  color: var(--ff-gold);
}
.hist-stars {
  display: flex;
  gap: 2px;
}
.hist-star {
  font-size: 18px;
  color: rgba(255, 255, 255, 0.15);
}
.hist-star.on {
  color: var(--ff-gold);
  text-shadow: 0 0 8px rgba(245, 197, 66, 0.8);
}
.result {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 8px 0 4px;
}
.result-chest-icon {
  font-size: 76px;
  display: inline-block;
  animation: bounce-in 0.6s var(--ff-bounce) both;
  filter: drop-shadow(0 6px 3px rgba(0, 0, 0, 0.5));
}
.result-chest.victory .result-chest-icon {
  animation: bob 2s ease-in-out infinite, bounce-in 0.6s var(--ff-bounce) both;
}
.result-stars {
  display: flex;
  gap: 12px;
}
.result-star {
  font-size: 44px;
  color: rgba(255, 255, 255, 0.14);
  transform: scale(0.4);
  opacity: 0;
}
.result-star.on {
  color: var(--ff-gold);
  text-shadow:
    0 2px 0 #8a5a00,
    0 0 24px rgba(245, 197, 66, 0.95);
  animation: pop-star 0.55s var(--ff-bounce) both;
}
.result-loot {
  font-size: 24px;
}
.result-sub {
  opacity: 0.6;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 10px;
}
</style>
