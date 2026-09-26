<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { IonPage, IonContent, IonHeader } from '@ionic/vue'
import SyncChip from '@/components/game/SyncChip.vue'
import { useGameStore } from '@/stores/gameStore'
import { useAuthStore } from '@/stores/authStore'
import type { LeaderboardRow } from '@/types'

const router = useRouter()
const game = useGameStore()
const auth = useAuthStore()

const loaded = ref(false)

onMounted(async () => {
  await game.fetchLeaderboard()
  loaded.value = true
})

const rows = computed<LeaderboardRow[]>(() => {
  if (game.leaderboard.length) return game.leaderboard
  return [
    { rank: 1, name: 'Morgath the Rich', trophies: 2400, level: 14 },
    { rank: 2, name: 'Coinessa', trophies: 2150, level: 12 },
    { rank: 3, name: 'Vaultor', trophies: 1980, level: 11 },
    { rank: 4, name: auth.user?.name ?? 'You', trophies: Math.max(100, game.resources.trophies), level: game.level, is_me: true },
    { rank: 5, name: 'LedgerLord', trophies: 890, level: 6 },
    { rank: 6, name: 'Pennywise Pete', trophies: 720, level: 5 }
  ]
})

const podium = computed(() => rows.value.slice(0, 3))
const rest = computed(() => rows.value.slice(3))
const order = computed(() => {
  const p = podium.value
  if (p.length < 3) return p
  return [p[1], p[0], p[2]] // 2nd, 1st, 3rd
})

function isMe(r: LeaderboardRow): boolean {
  if (r.is_me) return true
  const me = auth.user
  if (!me) return false
  if (r.user_id !== undefined && me.id !== undefined && String(r.user_id) === String(me.id)) return true
  const myName = me.name ?? me.username
  return !!myName && r.name === myName
}

function nameOf(r: LeaderboardRow): string {
  return r.name ?? r.username ?? 'Champion'
}

function rankOf(r: LeaderboardRow, i: number): number {
  return Number(r.rank ?? r.position ?? i + 1)
}

function trophiesOf(r: LeaderboardRow): number {
  return Number(r.trophies ?? 0)
}

const podiumMeta = [
  { cls: 'second', label: '2nd', delay: '0.15s' },
  { cls: 'first', label: '1st', delay: '0s' },
  { cls: 'third', label: '3rd', delay: '0.3s' }
]
</script>

<template>
  <ion-page>
    <ion-header class="hud">
      <div class="hud-row">
        <button class="back-btn" type="button" @click="router.push('/')">‹</button>
        <h1 class="carved carved-gold hud-title">LEADERBOARD</h1>
        <SyncChip />
      </div>
    </ion-header>

    <ion-content :fullscreen="true" class="lb-content">
      <div class="lb-scroll ff-hide-scrollbar">
        <div class="podium">
          <div
            v-for="(r, i) in order"
            :key="`p-${i}`"
            class="podium-col"
            :class="podiumMeta[i]?.cls ?? 'second'"
            :style="{ animationDelay: podiumMeta[i]?.delay ?? '0s' }"
          >
            <div class="podium-avatar">
              <span class="podium-emoji">{{ podiumMeta[i]?.cls === 'first' ? '👑' : '🛡️' }}</span>
            </div>
            <span class="carved podium-name clamp-1" :class="{ 'podium-me': isMe(r) }">
              {{ nameOf(r) }}
            </span>
            <span class="carved carved-sm podium-trophies">🏆 {{ trophiesOf(r) }}</span>
            <div class="podium-block" :class="podiumMeta[i]?.cls">
              <span class="podium-rank carved carved-gold">
                {{ podiumMeta[i]?.label }}
              </span>
              <span class="podium-lvl carved carved-sm">Lv {{ r.level ?? 1 }}</span>
            </div>
          </div>
        </div>

        <div class="panel-stone noise list-panel">
          <div
            v-for="(r, i) in rest"
            :key="`r-${i}`"
            class="lb-row"
            :class="{ 'lb-me': isMe(r) }"
            :style="{ animationDelay: `${i * 50}ms` }"
          >
            <span class="lb-rank carved carved-gold">#{{ rankOf(r, i) }}</span>
            <span class="lb-avatar">{{ isMe(r) ? '🙋' : '🤖' }}</span>
            <div class="lb-mid">
              <span class="carved lb-name clamp-1">{{ nameOf(r) }}</span>
              <span class="carved carved-sm lb-lvl">Level {{ r.level ?? 1 }}</span>
            </div>
            <span class="carved lb-trophies">🏆 {{ trophiesOf(r) }}</span>
          </div>
          <div v-if="!rest.length && !loaded" class="lb-empty carved carved-sm">Summoning ranks…</div>
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
  gap: 10px;
  padding: 12px 14px;
}
.back-btn {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  border: 3px solid #0a0500;
  background: linear-gradient(180deg, #ffe27a, var(--ff-gold) 55%, var(--ff-gold-dark));
  color: var(--ff-brown-deep);
  font-size: 26px;
  font-weight: 900;
  line-height: 1;
  cursor: pointer;
  box-shadow: 0 4px 0 #5c3c00;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.back-btn:active {
  transform: translateY(4px);
  box-shadow: none;
}
.hud-title {
  margin: 0;
  flex: 1;
  font-size: 21px;
  letter-spacing: 0.08em;
}
.lb-content {
  --background:
    radial-gradient(circle at 50% -10%, rgba(245, 197, 66, 0.2), transparent 50%),
    radial-gradient(circle at 50% 130%, #4a2c12, #120a00 75%);
}
.lb-scroll {
  padding: 20px 14px calc(28px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.podium {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 10px;
  min-height: 240px;
}
.podium-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  flex: 1;
  max-width: 130px;
  animation: bounce-in 0.7s var(--ff-bounce) both;
}
.podium-avatar {
  width: 58px;
  height: 58px;
  border-radius: 50%;
  background: radial-gradient(circle at 35% 30%, #ffe27a, var(--ff-gold-dark));
  border: 4px solid #120a00;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 5px 0 rgba(0, 0, 0, 0.5);
}
.podium-col.first .podium-avatar {
  width: 72px;
  height: 72px;
  animation: bob 2.2s ease-in-out infinite;
  box-shadow: 0 0 24px rgba(245, 197, 66, 0.8), 0 5px 0 rgba(0, 0, 0, 0.5);
}
.podium-emoji {
  font-size: 30px;
}
.podium-col.first .podium-emoji {
  font-size: 38px;
}
.podium-name {
  font-size: 13px;
  text-align: center;
  max-width: 100%;
  color: var(--ff-parchment);
}
.podium-me {
  color: var(--ff-gold);
  text-shadow: 0 0 10px rgba(245, 197, 66, 0.8);
}
.podium-trophies {
  color: #ffd76a;
  font-size: 12px;
}
.podium-block {
  width: 100%;
  border: 4px solid #120a00;
  border-radius: 12px 12px 0 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  padding: 8px 4px;
  box-shadow: inset 0 3px 0 rgba(255, 255, 255, 0.3);
}
.podium-block.first {
  height: 120px;
  background: linear-gradient(180deg, #ffe27a, var(--ff-gold) 40%, var(--ff-gold-dark));
}
.podium-block.second {
  height: 86px;
  background: linear-gradient(180deg, #e8e8e8, #b8b8b8 40%, #7d7d7d);
}
.podium-block.third {
  height: 66px;
  background: linear-gradient(180deg, #f0b27a, #cd7f32 40%, #8a5a00);
}
.podium-block .podium-rank {
  font-size: 20px;
  color: var(--ff-brown-deep);
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.4);
}
.podium-block .podium-lvl {
  color: rgba(0, 0, 0, 0.7);
  text-shadow: none;
}
.list-panel {
  padding: 14px 12px;
  display: flex;
  flex-direction: column;
  gap: 9px;
}
.lb-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), transparent);
  border: 3px solid #120a00;
  border-radius: 12px;
  box-shadow: 0 3px 0 rgba(0, 0, 0, 0.5);
  animation: bounce-in 0.4s var(--ff-bounce) both;
}
.lb-me {
  border-color: var(--ff-gold);
  background: linear-gradient(180deg, rgba(245, 197, 66, 0.22), rgba(245, 197, 66, 0.05));
  box-shadow: 0 0 16px rgba(245, 197, 66, 0.45), 0 3px 0 rgba(0, 0, 0, 0.5);
}
.lb-rank {
  font-size: 15px;
  width: 44px;
}
.lb-avatar {
  font-size: 24px;
}
.lb-mid {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.lb-name {
  font-size: 15px;
  color: var(--ff-parchment);
}
.lb-lvl {
  opacity: 0.6;
  font-size: 11px;
  text-transform: uppercase;
}
.lb-trophies {
  color: #ffd76a;
  font-size: 14px;
}
.lb-empty {
  text-align: center;
  opacity: 0.6;
  padding: 10px;
}
</style>
