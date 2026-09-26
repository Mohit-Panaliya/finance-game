<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { onIonViewWillEnter, IonPage, IonContent, IonHeader } from '@ionic/vue'
import GameButton from '@/components/game/GameButton.vue'
import ParticleBurst from '@/components/game/ParticleBurst.vue'
import SyncChip from '@/components/game/SyncChip.vue'
import { useGameStore } from '@/stores/gameStore'
import type { Achievement } from '@/types'

const game = useGameStore()

const fallback: Achievement[] = [
  { id: 'a1', title: 'First Coin', description: 'Collect from your first building', icon: '🪙', unlocked: true, claimed: false, progress: 1, target: 1, gold_reward: 100, xp_reward: 20 },
  { id: 'a2', title: 'Vault Keeper', description: 'Hold 10,000 gold', icon: '🗄️', unlocked: true, claimed: false, progress: 7400, target: 10000, gold_reward: 250, xp_reward: 40 },
  { id: 'a3', title: 'Warlord', description: 'Win 5 battles', icon: '⚔️', unlocked: false, claimed: false, progress: 2, target: 5, gold_reward: 500, xp_reward: 80 },
  { id: 'a4', title: 'Archmage', description: 'Upgrade the Wizard Tower to lvl 5', icon: '🔮', unlocked: false, claimed: false, progress: 3, target: 5, gems_reward: 5, xp_reward: 100 },
  { id: 'a5', title: 'Tycoon', description: 'Reach 100,000 net worth', icon: '💰', unlocked: false, claimed: false, progress: 42000, target: 100000, gold_reward: 1000, xp_reward: 150 },
  { id: 'a6', title: 'Grand Marshal', description: 'Reach level 10', icon: '👑', unlocked: false, claimed: false, progress: 4, target: 10, gems_reward: 20, xp_reward: 300 }
]

const achievements = computed<Achievement[]>(() =>
  game.achievements.length ? game.achievements : fallback
)

const unlockedCount = computed(
  () => achievements.value.filter((a) => a.unlocked || a.claimed).length
)

const bursts = reactive<Record<string, boolean>>({})
const claiming = ref<string | null>(null)
const err = ref('')

onIonViewWillEnter(() => {
  void game.fetchAchievements()
})

function progressPct(a: Achievement): number {
  const target = Number(a.target ?? a.max_progress ?? 100)
  const cur = Number(a.progress ?? 0)
  if (target <= 0) return a.unlocked ? 100 : 0
  return Math.max(0, Math.min(100, (cur / target) * 100))
}

const RING_C = 2 * Math.PI * 26

function ringOffset(a: Achievement): number {
  return RING_C * (1 - progressPct(a) / 100)
}

function isUnlocked(a: Achievement): boolean {
  return !!(a.unlocked || a.claimed)
}

async function claim(a: Achievement) {
  if (claiming.value) return
  err.value = ''
  claiming.value = String(a.id)
  try {
    const ok = await game.claimAchievement(a.id)
    if (ok) {
      bursts[String(a.id)] = false
      requestAnimationFrame(() => {
        bursts[String(a.id)] = true
      })
      window.setTimeout(() => {
        bursts[String(a.id)] = false
      }, 1200)
    }
  } catch (e) {
    err.value = e instanceof Error ? e.message : 'Claim failed'
  } finally {
    claiming.value = null
  }
}
</script>

<template>
  <ion-page>
    <ion-header class="hud">
      <div class="hud-row">
        <div>
          <h1 class="carved carved-gold hud-title">TROPHY ROOM</h1>
          <p class="carved carved-sm hud-sub">
            {{ unlockedCount }} / {{ achievements.length }} unlocked
          </p>
        </div>
        <SyncChip />
      </div>
    </ion-header>

    <ion-content :fullscreen="true" class="ach-content">
      <div class="ach-scroll ff-hide-scrollbar">
        <p v-if="err" class="ach-err carved carved-sm">{{ err }}</p>

        <div class="grid">
          <div
            v-for="(a, i) in achievements"
            :key="String(a.id)"
            class="ach-card"
            :class="{ locked: !isUnlocked(a), claimed: !!a.claimed }"
            :style="{ animationDelay: `${i * 70}ms` }"
          >
            <div class="ach-ring-wrap">
              <svg class="ach-ring" viewBox="0 0 60 60">
                <circle class="ring-track" cx="30" cy="30" r="26" />
                <circle
                  class="ring-val"
                  cx="30"
                  cy="30"
                  r="26"
                  :stroke-dasharray="RING_C"
                  :stroke-dashoffset="ringOffset(a)"
                />
              </svg>
              <span class="ach-icon" :class="{ bob: isUnlocked(a) }">{{ isUnlocked(a) ? a.icon ?? '🏆' : '🔒' }}</span>
              <ParticleBurst :active="!!bursts[String(a.id)]" :count="24" />
            </div>

            <div class="ach-body">
              <span class="carved ach-title" :class="{ 'carved-gold': isUnlocked(a) }">
                {{ a.title ?? a.name ?? 'Achievement' }}
              </span>
              <span class="carved carved-sm ach-desc">{{ a.description }}</span>
              <span class="carved carved-sm ach-prog">
                {{ Math.round(Number(a.progress ?? 0)) }} / {{ Number(a.target ?? a.max_progress ?? 100) }}
                · {{ Math.round(progressPct(a)) }}%
              </span>
              <span class="reward carved carved-sm">
                🎁 {{ a.gold_reward ? `${a.gold_reward}🪙` : '' }}
                {{ a.gems_reward ? `${a.gems_reward}💎` : '' }}
                {{ a.xp_reward ? `${a.xp_reward}XP` : '' }}
              </span>
            </div>

            <div class="ach-action">
              <GameButton
                v-if="isUnlocked(a) && !a.claimed"
                variant="gold"
                size="sm"
                sparkle
                :disabled="claiming === String(a.id)"
                @click="claim(a)"
              >
                {{ claiming === String(a.id) ? '…' : 'CLAIM' }}
              </GameButton>
              <span v-else-if="a.claimed" class="claimed-tag carved carved-sm">✓ CLAIMED</span>
              <span v-else class="locked-tag carved carved-sm">LOCKED</span>
            </div>
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
.hud-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
}
.hud-title {
  margin: 0;
  font-size: 22px;
  letter-spacing: 0.08em;
}
.hud-sub {
  margin: 2px 0 0;
  opacity: 0.7;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}
.ach-content {
  --background:
    radial-gradient(circle at 75% 5%, rgba(245, 197, 66, 0.16), transparent 45%),
    radial-gradient(circle at 50% 130%, #4a2c12, #120a00 75%);
}
.ach-scroll {
  padding: 14px 14px calc(28px + env(safe-area-inset-bottom));
}
.ach-err {
  color: #ff9d94;
  text-align: center;
}
.grid {
  display: flex;
  flex-direction: column;
  gap: 13px;
}
.ach-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: 13px;
  padding: 14px;
  background: linear-gradient(160deg, #4a3218, #241405);
  border: 4px solid #120a00;
  border-radius: 18px;
  box-shadow:
    inset 0 2px 0 rgba(255, 220, 150, 0.25),
    0 6px 0 #0d0700,
    0 10px 20px rgba(0, 0, 0, 0.55);
  animation: bounce-in 0.5s var(--ff-bounce) both;
  overflow: visible;
}
.ach-card.locked {
  background: linear-gradient(160deg, #3a3a3a, #222 60%, #161616);
  filter: grayscale(0.9);
  opacity: 0.85;
}
.ach-card.locked::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 14px;
  background: repeating-linear-gradient(
    45deg,
    rgba(255, 255, 255, 0.03) 0 6px,
    transparent 6px 14px
  );
  pointer-events: none;
}
.ach-card.claimed {
  border-color: var(--ff-gold-dark);
}
.ach-card:not(.locked) {
  box-shadow:
    inset 0 2px 0 rgba(255, 220, 150, 0.3),
    0 6px 0 #0d0700,
    0 0 22px rgba(245, 197, 66, 0.28),
    0 10px 20px rgba(0, 0, 0, 0.55);
}
.ach-ring-wrap {
  position: relative;
  width: 78px;
  height: 78px;
  flex-shrink: 0;
}
.ach-ring {
  width: 78px;
  height: 78px;
  transform: rotate(-90deg);
}
.ring-track {
  fill: rgba(0, 0, 0, 0.5);
  stroke: rgba(255, 255, 255, 0.1);
  stroke-width: 7;
}
.ring-val {
  fill: none;
  stroke: var(--ff-gold);
  stroke-width: 7;
  stroke-linecap: round;
  stroke-dasharray: 163.36;
  transition: stroke-dashoffset 1s cubic-bezier(0.34, 1.56, 0.64, 1);
  filter: drop-shadow(0 0 5px rgba(245, 197, 66, 0.8));
}
.locked .ring-val {
  stroke: #777;
  filter: none;
}
.ach-icon {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 30px;
}
.ach-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.ach-title {
  font-size: 16px;
}
.ach-desc {
  opacity: 0.75;
  font-size: 12px;
}
.ach-prog {
  color: var(--ff-gold);
  font-size: 11px;
}
.reward {
  opacity: 0.85;
  font-size: 11px;
}
.ach-action {
  flex-shrink: 0;
}
.claimed-tag {
  color: var(--ff-green);
  text-shadow: 0 0 8px rgba(76, 175, 80, 0.7);
}
.locked-tag {
  opacity: 0.55;
  background: rgba(0, 0, 0, 0.4);
  border: 2px solid #120a00;
  padding: 6px 10px;
  border-radius: 999px;
  font-size: 10px;
  letter-spacing: 0.1em;
}
</style>
