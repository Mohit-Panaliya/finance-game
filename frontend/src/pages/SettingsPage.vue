<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import {
  checkmarkCircleOutline,
  chevronDownOutline,
  chevronUpOutline,
  cloudDownloadOutline,
  cloudUploadOutline,
  downloadOutline,
  logOutOutline,
  refreshOutline,
  shareOutline
} from 'ionicons/icons'
import { useAuthStore } from '@/stores/authStore'
import { useFinanceStore } from '@/stores/financeStore'
import { useSyncStore } from '@/stores/syncStore'
import { usePwaInstall } from '@/composables/usePwaInstall'
import { getQueue } from '@/services/offlineQueue'
import { formatNumber } from '@/utils/money'
import { formatDateTime } from '@/utils/date'
import {
  ACCENT_PALETTES,
  DEFAULT_STYLE,
  DENSITIES,
  FONT_FAMILIES,
  THEME_FAMILIES,
  THEME_MODES,
  useTheme
} from '@/composables/useTheme'
import { ENTITY_LIST } from '@/entityConfig'
import AppButton from '@/components/ui/AppButton.vue'
import HeaderActions from '@/components/ui/HeaderActions.vue'

const auth = useAuthStore()
const finance = useFinanceStore()
const sync = useSyncStore()
const router = useRouter()
const theme = useTheme()
const pwa = usePwaInstall()

/** Offer the install prompt, or point iOS users at the manual flow. */
async function installApp() {
  const outcome = await pwa.promptInstall()
  if (outcome === 'accepted') notify('Fintrack installed')
  else if (outcome === 'unavailable') notify('Install from your browser menu', 'warning')
  else notify('Install prompt dismissed', 'warning')
}
function hideInstall() {
  pwa.dismiss()
  notify('Install entry hidden')
}

/** Family groups for the picker; the default option carries its own caption. */
const styleGroups = computed(() =>
  THEME_FAMILIES.map((family) => ({
    label: family.label,
    themes: family.themes.map((t) =>
      t.value === DEFAULT_STYLE ? { ...t, label: 'Flexoki (default)' } : t
    )
  }))
)

const appearanceNote = computed(() =>
  theme.mode === 'system'
    ? `Following your device (${theme.systemTheme})`
    : `Set to ${theme.resolvedTheme}`
)

function resetAppearance() {
  theme.reset()
  notify('Appearance reset to defaults')
}

/** Transient status line. @ionic/vue v8 exposes no useIonToast, so this is plain state. */
const notice = ref('')
let noticeTimer: ReturnType<typeof setTimeout> | undefined
function notify(message: string, tone: 'success' | 'warning' = 'success'): void {
  notice.value = message
  noticeTone.value = tone
  if (noticeTimer) clearTimeout(noticeTimer)
  noticeTimer = setTimeout(() => {
    notice.value = ''
  }, 2600)
}
const noticeTone = ref<'success' | 'warning'>('success')

/** Rarely-used sync tools sit behind a local disclosure so the section stays quiet. */
const showSyncTools = ref(false)

/** Reactive connectivity flag — `navigator` is not available on the component instance. */
const online = ref(typeof navigator !== 'undefined' ? navigator.onLine : true)
onMounted(() => {
  const up = () => (online.value = true)
  const down = () => (online.value = false)
  window.addEventListener('online', up)
  window.addEventListener('offline', down)
  void auth.fetchMe()
  return () => {
    window.removeEventListener('online', up)
    window.removeEventListener('offline', down)
    if (noticeTimer) clearTimeout(noticeTimer)
  }
})

const user = computed(() => auth.user)
const email = computed(() => user.value?.email ?? user.value?.username ?? '—')
const displayName = computed(() => user.value?.name ?? user.value?.username ?? '—')

const lastSync = computed(() => (sync.lastSyncAt ? formatDateTime(sync.lastSyncAt) : 'Never'))

const recordCount = computed(() =>
  ENTITY_LIST.reduce((acc, e) => acc + (finance.lists[e].total || finance.lists[e].items.length), 0)
)

const statusTone = computed(() => {
  switch (sync.status) {
    case 'offline':
      return 'badge-danger'
    case 'pushing':
      return 'badge-accent'
    case 'error':
      return 'badge-danger'
    case 'synced':
      return 'badge-success'
    default:
      return 'badge-success'
  }
})

async function syncNow() {
  await sync.syncAll()
  await finance.refresh()
  notify(sync.message || 'Sync complete', sync.status === 'error' ? 'warning' : 'success')
}

async function pullLatest() {
  await sync.pull()
  await finance.refresh()
  notify(sync.message || 'Latest changes pulled', sync.status === 'error' ? 'warning' : 'success')
}

async function flushQueue() {
  const ok = await sync.flush()
  notify(ok ? 'Queue flushed' : 'Queue still waiting', ok ? 'success' : 'warning')
}

async function refreshAll() {
  await finance.refresh()
  notify('Data refreshed')
}

async function logout() {
  auth.logout()
  finance.reset()
  sync.initialized = false
  sync.status = navigator.onLine ? 'online' : 'offline'
  sync.queueCount = 0
  sync.message = ''
  await router.replace('/login')
}

async function queueSize() {
  const q = await getQueue()
  notify(`${q.length} change${q.length === 1 ? '' : 's'} queued`)
}

</script>

<template>
  <ion-page>
    <ion-content class="app-content">
      <div class="page">
      <p v-if="notice" class="notice" :class="`notice-${noticeTone}`" role="status">{{ notice }}</p>
        <header class="page-head">
          <div class="page-head-text">
            <h1 class="page-title">Settings</h1>
            <p class="page-subtitle">Sync, appearance and account</p>
          </div>
          <HeaderActions
            :status="sync.status"
            :pending="sync.queueCount"
            :last-sync-at="sync.lastSyncAt"
            @sync="syncNow"
          />
        </header>

        <section class="card">
          <p class="card-label">Signed in as</p>
          <p class="card-value card-value-sm">{{ displayName }}</p>
          <p class="card-foot">{{ email }}</p>
        </section>

        <section class="section">
          <div class="section-head">
            <h2 class="section-title">Appearance</h2>
            <span class="section-note">{{ appearanceNote }}</span>
          </div>

          <div class="card">
            <p class="card-label">Theme</p>
            <div class="style-families" role="group" aria-label="Theme style">
              <div v-for="family in styleGroups" :key="family.label" class="style-family">
                <p class="style-family-label">{{ family.label }}</p>
                <div class="style-chips">
                  <button
                    v-for="s in family.themes"
                    :key="s.value"
                    type="button"
                    class="chip"
                    :class="{ 'chip-active': theme.style === s.value }"
                    :aria-pressed="theme.style === s.value"
                    @click="theme.setStyle(s.value)"
                  >
                    {{ s.label }}
                  </button>
                </div>
              </div>
            </div>

            <p class="card-label spaced">Mode</p>
            <div class="chips" role="group" aria-label="Theme mode">
              <button
                v-for="m in THEME_MODES"
                :key="m.value"
                type="button"
                class="chip theme-mode"
                :class="[`theme-mode-${m.value}`, { 'chip-active': theme.mode === m.value }]"
                :aria-pressed="theme.mode === m.value"
                @click="theme.setMode(m.value)"
              >
                {{ m.label }}
              </button>
            </div>

            <p class="card-label spaced">Accent</p>
            <div class="swatch-grid" role="group" aria-label="Accent palette">
              <button
                v-for="p in ACCENT_PALETTES"
                :key="p.name"
                type="button"
                class="accent-swatch"
                :class="{ 'accent-swatch-active': theme.accent === p.name }"
                :data-palette="p.name"
                :aria-pressed="theme.accent === p.name"
                @click="theme.setAccent(p.name)"
              >
                <span class="swatch-dot" :style="{ background: p.swatch }" />
                <span class="swatch-label">{{ p.label }}</span>
                <ion-icon
                  v-if="theme.accent === p.name"
                  class="swatch-check"
                  :icon="checkmarkCircleOutline"
                />
              </button>
            </div>

            <p class="card-label spaced">Density</p>
            <div class="chips" role="group" aria-label="Layout density">
              <button
                v-for="d in DENSITIES"
                :key="d.value"
                type="button"
                class="chip density-option"
                :class="[`density-${d.value}`, { 'chip-active': theme.density === d.value }]"
                :aria-pressed="theme.density === d.value"
                @click="theme.setDensity(d.value)"
              >
                {{ d.label }}
              </button>
            </div>

            <p class="card-label spaced">Font</p>
            <p class="hint-text">Sets the body/UI type. Each theme keeps its own display face.</p>
            <div class="style-families" role="group" aria-label="UI font">
              <div v-for="family in FONT_FAMILIES" :key="family.label" class="style-family">
                <p class="style-family-label">{{ family.label }}</p>
                <div class="style-chips">
                  <button
                    v-for="f in family.fonts"
                    :key="f.value"
                    type="button"
                    class="chip font-option"
                    :class="{ 'chip-active': theme.font === f.value }"
                    :aria-pressed="theme.font === f.value"
                    :style="{ fontFamily: f.stack }"
                    @click="theme.setFont(f.value)"
                  >
                    {{ f.label }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <AppButton variant="neutral" size="md" @click="resetAppearance">
            <ion-icon :icon="refreshOutline" /> Reset to defaults
          </AppButton>
        </section>

        <section class="section">
          <h2 class="section-title">Sync</h2>
          <div class="card">
            <div class="sync-head">
              <span class="badge" :class="statusTone">{{ sync.status }}</span>
              <span class="text-sm text-muted">{{ sync.message || 'No pending changes' }}</span>
            </div>
            <div class="meta-list sync-meta">
              <div class="meta-row">
                <span class="meta-key">Connection</span>
                <span class="meta-val">{{ online ? 'Online' : 'Offline' }}</span>
              </div>
              <div class="meta-row">
                <span class="meta-key">Queued changes</span>
                <span class="meta-val">{{ formatNumber(sync.queueCount) }}</span>
              </div>
              <div class="meta-row">
                <span class="meta-key">Last synced</span>
                <span class="meta-val">{{ lastSync }}</span>
              </div>
              <div class="meta-row">
                <span class="meta-key">Records loaded</span>
                <span class="meta-val">{{ formatNumber(recordCount) }}</span>
              </div>
            </div>
          </div>

          <div class="action-grid">
            <AppButton variant="primary" size="md" :disabled="sync.status === 'pushing'" @click="syncNow">
              <ion-icon :icon="cloudUploadOutline" /> Sync now
            </AppButton>
            <AppButton variant="neutral" size="md" @click="pullLatest">
              <ion-icon :icon="cloudDownloadOutline" /> Pull latest
            </AppButton>
          </div>

          <button
            class="link-row"
            type="button"
            :aria-expanded="showSyncTools"
            @click="showSyncTools = !showSyncTools"
          >
            <span class="text-sm text-muted">More sync tools</span>
            <ion-icon class="text-faint" :icon="showSyncTools ? chevronUpOutline : chevronDownOutline" />
          </button>

          <div v-show="showSyncTools" class="sync-tools">
            <AppButton variant="ghost" size="sm" :disabled="!sync.queueCount" @click="flushQueue">
              Push queue
            </AppButton>
            <AppButton variant="ghost" size="sm" @click="refreshAll">
              <ion-icon :icon="refreshOutline" /> Reload data
            </AppButton>
            <AppButton variant="ghost" size="sm" @click="queueSize">Inspect offline queue</AppButton>
          </div>
        </section>

        <section
          v-if="pwa.isStandalone || ((pwa.isInstallable || pwa.showIosHint) && !pwa.isDismissed)"
          class="section"
        >
          <h2 class="section-title">App</h2>
          <div class="card">
            <p class="card-label">Install</p>
            <template v-if="pwa.isStandalone || pwa.installed">
              <p class="text-sm text-muted">
                Fintrack is installed — launch it from your home screen or app list and it works offline.
              </p>
            </template>
            <template v-else-if="pwa.isInstallable">
              <p class="text-sm text-muted">
                Install Fintrack for a full-screen, offline-capable experience.
              </p>
              <div class="action-grid install-actions">
                <AppButton variant="primary" size="md" @click="installApp">
                  <ion-icon :icon="downloadOutline" /> Install app
                </AppButton>
                <AppButton variant="ghost" size="md" @click="hideInstall">Not now</AppButton>
              </div>
            </template>
            <template v-else>
              <p class="text-sm text-muted">
                On iPhone or iPad, tap
                <ion-icon class="inline-icon" :icon="shareOutline" />
                <strong>Share</strong>, then choose <strong>Add to Home Screen</strong>.
              </p>
              <div class="install-actions">
                <AppButton variant="ghost" size="sm" @click="hideInstall">Got it</AppButton>
              </div>
            </template>
          </div>
        </section>

        <section class="section">
          <h2 class="section-title">About</h2>
          <div class="card">
            <p class="text-sm text-muted">Fintrack 1.0</p>
          </div>
        </section>

        <AppButton variant="danger" size="lg" block @click="logout">
          <ion-icon :icon="logOutOutline" /> Sign out
        </AppButton>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.sync-head {
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  flex-wrap: wrap;
}
.action-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--density-gap);
}
.link-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: var(--density-row-pad-y) var(--density-row-pad-x);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
}
.sync-meta {
  margin-top: var(--density-gap);
}
.install-actions {
  margin-top: var(--density-gap);
}
.inline-icon {
  vertical-align: -2px;
  font-size: 1em;
  color: var(--accent);
}
.sync-tools {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--density-gap);
}
.spaced {
  margin-top: var(--density-gap);
}
.hint-text {
  margin: -4px 0 8px;
  font-size: 0.78rem;
  color: var(--text-muted);
}
.font-option {
  font-variant-numeric: tabular-nums;
}
.style-families {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
}
.style-family {
  display: flex;
  flex-direction: column;
  gap: var(--density-gap);
}
.style-family-label {
  font-size: calc(0.74rem * var(--density-font-scale));
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-faint);
}
.style-chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--density-gap);
}
.swatch-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--density-gap);
}
.accent-swatch {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 7px;
  padding: 11px 6px 9px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    background 0.14s ease,
    border-color 0.14s ease;
}
.accent-swatch-active {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.swatch-dot {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 1px solid var(--border-strong);
}
.swatch-label {
  font-size: 0.74rem;
  font-weight: 600;
  color: var(--text-muted);
}
.accent-swatch-active .swatch-label {
  color: var(--accent-ink);
}
.swatch-check {
  position: absolute;
  top: 4px;
  right: 5px;
  font-size: 15px;
  color: var(--accent-ink);
}
@media (max-width: 380px) {
  .action-grid {
    grid-template-columns: 1fr;
  }
  .swatch-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>