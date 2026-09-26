<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { IonApp, IonRouterOutlet, IonTabs, IonTabBar, IonTabButton, IonIcon, IonLabel } from '@ionic/vue'
import {
  homeOutline,
  flashOutline,
  libraryOutline,
  peopleOutline,
  trophyOutline,
  analyticsOutline
} from 'ionicons/icons'
import { useAuthStore } from '@/stores/authStore'
import { useGameStore } from '@/stores/gameStore'
import { useSyncStore } from '@/stores/syncStore'

const route = useRoute()
const auth = useAuthStore()
const game = useGameStore()
const sync = useSyncStore()

const showTabs = computed(
  () => auth.isAuthenticated && !['login', 'register'].includes(String(route.name ?? ''))
)

const tabs = [
  { name: 'village', label: 'Village', icon: homeOutline, path: '/' },
  { name: 'battle', label: 'Battle', icon: flashOutline, path: '/battle' },
  { name: 'records', label: 'Records', icon: libraryOutline, path: '/records' },
  { name: 'army', label: 'Army', icon: peopleOutline, path: '/army' },
  { name: 'achievements', label: 'Awards', icon: trophyOutline, path: '/achievements' },
  { name: 'analysis', label: 'Analysis', icon: analyticsOutline, path: '/analysis' }
]

function isSelected(tab: string): boolean {
  if (route.name === tab) return true
  if (tab === 'records' && String(route.name ?? '').startsWith('record')) return true
  return false
}

function lockLandscape(): void {
  try {
    const orientation = (screen as unknown as {
      orientation?: { lock?: (type: string) => Promise<void> | undefined }
    }).orientation
    const locked = orientation?.lock?.('landscape')
    if (locked) void locked.catch(() => undefined)
  } catch {
    /* orientation lock unsupported (desktop/iOS/unsupported browser) — ignore */
  }
}

onMounted(() => {
  void sync.init()
  if (auth.isAuthenticated) {
    void game.fetchVillage()
  }
  lockLandscape()
})
</script>

<template>
  <ion-app>
    <ion-tabs :class="{ 'game-tab-hidden': !showTabs }">
      <ion-router-outlet />
      <ion-tab-bar slot="bottom" class="game-tab-bar" :class="{ hidden: !showTabs }">
        <ion-tab-button
          v-for="tab in tabs"
          :key="tab.name"
          :tab="tab.name"
          :href="tab.path"
          :class="{ selected: isSelected(tab.name) }"
        >
          <ion-icon :icon="tab.icon" />
          <ion-label>{{ tab.label }}</ion-label>
        </ion-tab-button>
      </ion-tab-bar>
    </ion-tabs>
  </ion-app>
</template>
