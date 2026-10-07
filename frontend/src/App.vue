<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { IonApp, IonIcon, IonRouterOutlet } from '@ionic/vue'
import {
  homeOutline, peopleOutline,
  listOutline,
  settingsOutline,
  walletOutline,
  analyticsOutline,
  cloudOfflineOutline,
  syncOutline,
  albumsOutline,
  cartOutline
} from 'ionicons/icons'
import { useSyncStore } from '@/stores/syncStore'
import { usePlatform } from '@/composables/usePlatform'

const route = useRoute()
const sync = useSyncStore()
const platform = usePlatform()

interface NavEntry {
  path: string
  icon: string
  label: string
  hint: string
}

const PRIMARY: NavEntry[] = [
  { path: '/dashboard', icon: homeOutline, label: 'Dashboard', hint: 'Net worth and totals' },
  { path: '/accounts', icon: walletOutline, label: 'Accounts', hint: 'Banks, cards, deposits' },
  { path: '/transactions', icon: listOutline, label: 'Activity', hint: 'Income and expenses' },
  { path: '/analytics', icon: analyticsOutline, label: 'Insights', hint: 'Trends and allocation' },
  { path: '/debts', icon: peopleOutline, label: 'Debts', hint: 'Who owes whom' },
  { path: '/notes', icon: albumsOutline, label: 'Notes', hint: 'Keep notes and lists' },
  { path: '/buy-list', icon: cartOutline, label: 'Buy List', hint: 'Wishlist and purchases' },


  { path: '/settings', icon: settingsOutline, label: 'Settings', hint: 'Sync and account' }
]

/** A tab is active on its own path and on any nested path below it. */
function isActive(path: string): boolean {
  return route.path === path || route.path.startsWith(`${path}/`)
}

const showTabs = computed(() => route.path !== '/login' && route.path !== '/register')

// Same signal SettingsPage uses, so the sidebar and settings agree.
const online = ref(typeof navigator !== 'undefined' ? navigator.onLine : true)
const setOnline = () => (online.value = true)
const setOffline = () => (online.value = false)

onMounted(() => {
  window.addEventListener('online', setOnline)
  window.addEventListener('offline', setOffline)
  void sync.init()
})

onUnmounted(() => {
  window.removeEventListener('online', setOnline)
  window.removeEventListener('offline', setOffline)
})
</script>

<template>
  <ion-app :class="platform.platformClass">
    <ion-router-outlet />

    <template v-if="showTabs">
      <!-- Windows/desktop: fixed sidebar with descriptive nav, no bottom bar. -->
      <aside v-if="platform.isDesktop" class="side-nav" aria-label="Primary">
        <div class="side-brand">
          <span class="side-logo" aria-hidden="true">F</span>
          <span class="side-brand-text">
            <strong>Fintrack</strong>
            <small>Personal finance</small>
          </span>
        </div>

        <nav class="side-links">
          <router-link
            v-for="item in PRIMARY"
            :key="item.path"
            class="side-link"
            :class="{ 'side-link-active': isActive(item.path) }"
            :to="item.path"
            :aria-current="isActive(item.path) ? 'page' : undefined"
          >
            <ion-icon class="side-icon" :icon="item.icon" />
            <span class="side-link-text">
              <span class="side-link-label">{{ item.label }}</span>
              <span class="side-link-hint">{{ item.hint }}</span>
            </span>
          </router-link>
        </nav>

        <div class="side-foot">
          <div class="side-status" :class="{ 'side-status-off': !online }">
            <ion-icon :icon="online ? syncOutline : cloudOfflineOutline" />
            <span>
              <strong>{{ online ? 'Synced' : 'Offline' }}</strong>
              <small v-if="sync.queueCount">{{ sync.queueCount }} pending</small>
              <small v-else>Up to date</small>
            </span>
          </div>
        </div>
      </aside>

      <!-- Android/iOS: unchanged bottom tab bar. -->
      <nav v-else class="app-nav" aria-label="Primary">
        <router-link
          v-for="tab in PRIMARY"
          :key="tab.path"
          class="app-nav-link"
          :class="{ 'app-nav-link-active': isActive(tab.path) }"
          :to="tab.path"
        >
          <ion-icon class="app-nav-icon" :icon="tab.icon" />
          <span class="app-nav-label">{{ tab.label }}</span>
        </router-link>
      </nav>
    </template>
  </ion-app>
</template>

<style scoped>
/* ---- desktop sidebar ---- */
.side-nav {
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  z-index: 30;
  width: 264px;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border-right: 1px solid var(--border);
}

.side-brand {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 20px 18px 18px;
  border-bottom: 1px solid var(--border);
}

.side-logo {
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  border-radius: 10px;
  background: var(--accent);
  color: #fff;
  font-weight: 700;
  font-size: 1.05rem;
}

.side-brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.25;
}
.side-brand-text strong {
  font-size: 0.98rem;
}
.side-brand-text small {
  color: var(--text-faint);
  font-size: 0.72rem;
}

.side-links {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 14px 10px;
  overflow-y: auto;
}

.side-link {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 9px 11px;
  border-radius: var(--radius-sm);
  color: var(--text);
  text-decoration: none;
  transition: background 0.14s ease, color 0.14s ease;
}
.side-link:hover {
  background: var(--surface-2);
}
.side-link-active {
  background: var(--accent-soft);
  color: var(--accent);
}
.side-icon {
  font-size: 19px;
  flex: none;
}
.side-link-text {
  display: flex;
  flex-direction: column;
  line-height: 1.3;
  min-width: 0;
}
.side-link-label {
  font-size: 0.88rem;
  font-weight: 600;
}
.side-link-hint {
  color: var(--text-faint);
  font-size: 0.7rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.side-foot {
  padding: 12px;
  border-top: 1px solid var(--border);
}
.side-status {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 11px;
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--success);
  font-size: 0.8rem;
}
.side-status-off {
  color: var(--warning);
}
.side-status span {
  display: flex;
  flex-direction: column;
  line-height: 1.25;
}
.side-status small {
  color: var(--text-faint);
  font-size: 0.7rem;
}

/* ---- mobile tab bar ---- */
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
