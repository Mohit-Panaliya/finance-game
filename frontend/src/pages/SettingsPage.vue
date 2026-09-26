<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { IonPage, IonContent, IonHeader } from '@ionic/vue'
import GameButton from '@/components/game/GameButton.vue'
import SyncChip from '@/components/game/SyncChip.vue'
import ParticleBurst from '@/components/game/ParticleBurst.vue'
import { useAuthStore } from '@/stores/authStore'
import { useSyncStore } from '@/stores/syncStore'
import { useGameStore } from '@/stores/gameStore'
import { getQueue } from '@/services/offlineQueue'

const router = useRouter()
const auth = useAuthStore()
const sync = useSyncStore()
const game = useGameStore()

const queuePreview = ref(0)
const navigatorOnLine = ref(typeof navigator === 'undefined' ? true : navigator.onLine)
const burst = ref(false)
const note = ref('')

const statusLabel = computed(() => {
  switch (sync.status) {
    case 'offline': return 'OFFLINE — queueing changes'
    case 'pushing': return 'PUSHING changes to the keep…'
    case 'synced': return 'SYNCED with the realm'
    case 'error': return `SYNC ERROR — ${sync.message}`
    default: return 'ONLINE — ready to trade'
  }
})

const lastSync = computed(() =>
  sync.lastSyncAt ? new Date(sync.lastSyncAt).toLocaleTimeString() : 'never'
)

onMounted(async () => {
  await refreshQueue()
  window.addEventListener('online', () => (navigatorOnLine.value = true))
  window.addEventListener('offline', () => (navigatorOnLine.value = false))
})

async function refreshQueue() {
  const q = await getQueue()
  queuePreview.value = q.length
  await sync.refreshCount()
}

async function syncNow() {
  note.value = ''
  await sync.syncAll()
  await refreshQueue()
  burst.value = false
  requestAnimationFrame(() => {
    burst.value = true
  })
  window.setTimeout(() => {
    burst.value = false
  }, 1100)
  note.value = sync.message || 'Done'
}

function logout() {
  auth.logout()
  game.$reset()
  void router.replace('/login')
}
</script>

<template>
  <ion-page>
    <ion-header class="hud">
      <div class="hud-row">
        <button class="back-btn" type="button" @click="router.push('/')">‹</button>
        <h1 class="carved carved-gold hud-title">SETTINGS</h1>
        <SyncChip />
      </div>
    </ion-header>

    <ion-content :fullscreen="true" class="set-content">
      <div class="set-scroll ff-hide-scrollbar">
        <!-- profile -->
        <div class="panel-stone noise card">
          <div class="profile">
            <div class="avatar bob">🛡️</div>
            <div class="profile-info">
              <span class="carved carved-gold profile-name">
                {{ auth.user?.name ?? auth.user?.username ?? auth.user?.email ?? 'Champion' }}
              </span>
              <span class="carved carved-sm profile-mail">
                {{ auth.user?.email ?? 'sworn to the fortress' }}
              </span>
              <span class="carved carved-sm profile-lvl">Village Level {{ game.level }}</span>
            </div>
          </div>
        </div>

        <!-- sync -->
        <div class="panel-wood noise card sync-card">
          <div class="sync-head">
            <h2 class="carved carved-gold">REALM SYNC</h2>
            <span class="status-dot" :class="sync.status" />
          </div>
          <p class="carved status-line" :class="`status-${sync.status}`">{{ statusLabel }}</p>
          <div class="sync-grid">
            <div class="sync-cell">
              <span class="k carved carved-sm">Queue</span>
              <span class="v carved carved-gold">{{ sync.queueCount }} ops</span>
            </div>
            <div class="sync-cell">
              <span class="k carved carved-sm">Last sync</span>
              <span class="v carved">{{ lastSync }}</span>
            </div>
            <div class="sync-cell">
              <span class="k carved carved-sm">IndexedDB</span>
              <span class="v carved">{{ queuePreview }} local</span>
            </div>
            <div class="sync-cell">
              <span class="k carved carved-sm">Connection</span>
              <span class="v carved" :class="{ offline: !navigatorOnLine }">
                {{ navigatorOnLine ? '🌐 live' : '📡 none' }}
              </span>
            </div>
          </div>

          <div class="sync-actions">
            <GameButton variant="gold" size="md" sparkle @click="syncNow">SYNC NOW</GameButton>
            <GameButton variant="blue" size="md" @click="sync.pull()">PULL LATEST</GameButton>
          </div>
          <p v-if="note" class="note carved carved-sm">{{ note }}</p>

          <div class="burst-host">
            <ParticleBurst :active="burst" :count="26" origin-x="30%" origin-y="60%" />
          </div>
        </div>

        <!-- about -->
        <div class="panel-stone noise card">
          <h2 class="carved carved-gold card-title">THE KEEP</h2>
          <ul class="about carved carved-sm">
            <li>⚔️ Finance Forge — personal finance, forged as a game</li>
            <li>🏛️ Buildings track banks, assets, vaults &amp; more</li>
            <li>📦 Works offline — mutations queue in IndexedDB</li>
            <li>🔄 Service worker caches assets + API GETs</li>
            <li>📳 PWA — install from your browser menu</li>
          </ul>
        </div>

        <GameButton variant="red" size="lg" block @click="logout">LEAVE FORTRESS</GameButton>

        <p class="version carved carved-sm">Finance Forge v1.0.0 · theme #1a0f00</p>
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
.set-content {
  --background:
    radial-gradient(circle at 30% 0%, rgba(52, 152, 219, 0.12), transparent 45%),
    radial-gradient(circle at 50% 130%, #4a2c12, #120a00 75%);
}
.set-scroll {
  padding: 16px 14px calc(30px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 15px;
}
.card {
  padding: 16px 14px;
  position: relative;
}
.card-title {
  margin: 0 0 10px;
  font-size: 17px;
  letter-spacing: 0.1em;
}
.profile {
  display: flex;
  align-items: center;
  gap: 14px;
}
.avatar {
  width: 68px;
  height: 68px;
  border-radius: 18px;
  background: radial-gradient(circle at 35% 30%, #ffe27a, var(--ff-gold-dark));
  border: 4px solid #120a00;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 34px;
  box-shadow: 0 5px 0 rgba(0, 0, 0, 0.5);
}
.profile-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}
.profile-name {
  font-size: 20px;
}
.profile-mail,
.profile-lvl {
  opacity: 0.7;
  font-size: 12px;
}
.sync-card {
  position: relative;
  overflow: hidden;
}
.sync-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.sync-head h2 {
  margin: 0;
  font-size: 17px;
  letter-spacing: 0.1em;
}
.status-dot {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--ff-green);
  box-shadow: 0 0 10px currentColor;
  animation: pulse-glow 1.3s ease-in-out infinite;
}
.status-dot.offline { background: var(--ff-red); }
.status-dot.pushing { background: var(--ff-blue); }
.status-dot.synced { background: var(--ff-gold); }
.status-dot.error { background: var(--ff-red); }
.status-line {
  margin: 8px 0 12px;
  font-size: 14px;
}
.status-offline { color: #ff9d94; }
.status-pushing { color: #8fd0f7; }
.status-synced { color: var(--ff-gold); }
.status-error { color: #ff9d94; }
.sync-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 9px;
  margin-bottom: 13px;
}
.sync-cell {
  background: rgba(0, 0, 0, 0.35);
  border: 3px solid #120a00;
  border-radius: 12px;
  padding: 8px 11px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.sync-cell .k {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  opacity: 0.7;
}
.sync-cell .v {
  font-size: 15px;
}
.sync-cell .v.offline {
  color: var(--ff-red);
}
.sync-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.note {
  margin: 10px 0 0;
  text-align: center;
  color: var(--ff-green);
}
.burst-host {
  position: absolute;
  inset: 0;
  pointer-events: none;
}
.about {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 7px;
  line-height: 1.4;
  opacity: 0.85;
}
.version {
  text-align: center;
  opacity: 0.5;
  margin: 4px 0 0;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}
</style>
