<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { IonApp, IonIcon, IonRouterOutlet } from '@ionic/vue'
import {
  analyticsOutline,
  homeOutline,
  listOutline,
  settingsOutline,
  walletOutline
} from 'ionicons/icons'
import { useSyncStore } from '@/stores/syncStore'

const route = useRoute()
const sync = useSyncStore()

const TABS = [
  { path: '/dashboard', icon: homeOutline, label: 'Home' },
  { path: '/accounts', icon: walletOutline, label: 'Accounts' },
  { path: '/transactions', icon: listOutline, label: 'Activity' },
  { path: '/analytics', icon: analyticsOutline, label: 'Insights' },
  { path: '/settings', icon: settingsOutline, label: 'Settings' }
]

/** A tab is active on its own path and on any nested path below it. */
function isActive(path: string): boolean {
  return route.path === path || route.path.startsWith(`${path}/`)
}

const showTabs = computed(() => route.path !== '/login' && route.path !== '/register')

onMounted(() => {
  void sync.init()
})
</script>

<template>
  <ion-app>
    <ion-router-outlet />

    <nav v-if="showTabs" class="app-nav" aria-label="Primary">
      <router-link
        v-for="tab in TABS"
        :key="tab.path"
        class="app-nav-link"
        :class="{ 'app-nav-link-active': isActive(tab.path) }"
        :to="tab.path"
      >
        <ion-icon class="app-nav-icon" :icon="tab.icon" />
        <span class="app-nav-label">{{ tab.label }}</span>
      </router-link>
    </nav>
  </ion-app>
</template>

<style scoped>
.app-nav {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 20;
  display: flex;
  align-items: stretch;
  height: calc(58px + env(safe-area-inset-bottom));
  padding-bottom: env(safe-area-inset-bottom);
  background: var(--surface);
  border-top: 1px solid var(--border);
}
.app-nav-link {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  color: var(--text-faint);
  text-decoration: none;
  transition: color 0.15s ease;
  -webkit-tap-highlight-color: transparent;
}
.app-nav-link-active {
  color: var(--accent);
}
.app-nav-icon {
  font-size: 22px;
}
.app-nav-label {
  font-size: 0.64rem;
  font-weight: 600;
  letter-spacing: 0.01em;
}
</style>