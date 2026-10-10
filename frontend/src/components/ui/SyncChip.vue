<script setup lang="ts">
import { computed } from 'vue'
import { IonIcon } from '@ionic/vue'
import { cloudDoneOutline, cloudOfflineOutline, refreshOutline, syncOutline } from 'ionicons/icons'
import { useSyncStore } from '@/stores/syncStore'
import type { SyncStatus } from '@/stores/syncStore'
import { relativeTime } from '@/utils/date'

const props = withDefaults(
  defineProps<{ status?: SyncStatus; pending?: number; lastSyncAt?: number | null }>(),
  { status: 'online', pending: 0, lastSyncAt: null }
)

const emit = defineEmits<{ sync: [] }>()

const store = useSyncStore()

const meta = computed(() => {
  switch (props.status) {
    case 'offline':
      return { label: 'Offline', icon: cloudOfflineOutline, tone: 'offline' }
    case 'pushing':
      return { label: 'Syncing', icon: syncOutline, tone: 'pushing' }
    case 'error':
      return { label: 'Sync error', icon: refreshOutline, tone: 'error' }
    case 'synced':
      return { label: 'Synced', icon: cloudDoneOutline, tone: 'synced' }
    default:
      return { label: 'Online', icon: syncOutline, tone: 'online' }
  }
})

const queued = computed(() => Math.max(0, props.pending | 0))

const text = computed(() => {
  if (props.status === 'offline') return queued.value > 0 ? `Offline · ${queued.value} queued` : 'Offline'
  if (props.status === 'error') return queued.value > 0 ? `Error · ${queued.value} queued` : 'Sync error'
  if (props.status === 'pushing') return queued.value > 0 ? `Syncing · ${queued.value}` : 'Syncing…'
  return meta.value.label
})

const title = computed(() => {
  const n = queued.value
  const rel = props.lastSyncAt ? `Last synced ${relativeTime(props.lastSyncAt)}` : 'Not synced yet'
  switch (props.status) {
    case 'offline':
      return `Offline — ${n} change${n === 1 ? '' : 's'} queued. Tap to retry.`
    case 'error':
      return `Sync failed${n > 0 ? `, ${n} queued` : ''}. Tap to retry.`
    case 'pushing':
      return `Pushing ${n} change${n === 1 ? '' : 's'}…`
    default:
      return `${meta.value.label} — ${rel}. Tap to sync.`
  }
})

function onTap() {
  // Tap always retries the sync itself (per the badge contract) while still
  // forwarding the emit so current call sites keep refreshing their lists.
  void store.syncAll()
  emit('sync')
}
</script>

<template>
  <button class="schip" :class="`schip-${meta.tone}`" type="button" :title="title" @click="onTap">
    <ion-icon class="schip-icon" :icon="meta.icon" />
    <span class="schip-text">{{ text }}</span>
  </button>
</template>

<style scoped>
.schip {
  display: inline-flex;
  align-items: center;
  gap: var(--density-gap);
  min-height: 34px;
  padding: 0 var(--density-row-pad-x);
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--surface-2);
  color: var(--text);
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition:
    background 0.15s ease,
    border-color 0.15s ease,
    color 0.15s ease;
}
.schip-icon {
  font-size: 15px;
}
.schip-online {
  color: var(--success);
  border-color: var(--success-soft);
  background: var(--success-soft);
}
.schip-synced {
  color: var(--success);
  border-color: var(--success-soft);
  background: var(--success-soft);
}
.schip-pushing {
  color: var(--accent);
  border-color: var(--accent-soft);
  background: var(--accent-soft);
}
@keyframes schip-spin {
  to {
    transform: rotate(360deg);
  }
}
@keyframes schip-pulse {
  0%,
  100% {
    box-shadow: 0 0 0 0 var(--accent-soft);
  }
  50% {
    box-shadow: 0 0 0 5px transparent;
  }
}
.schip-offline {
  color: var(--warning);
  border-color: var(--warning-soft);
  background: var(--warning-soft);
}
.schip-error {
  color: var(--danger);
  border-color: var(--danger-soft);
  background: var(--danger-soft);
}
@media (prefers-reduced-motion: no-preference) {
  .schip-pushing .schip-icon {
    animation: schip-spin 1.1s linear infinite;
  }
  .schip-pushing {
    animation: schip-pulse 1.5s ease-in-out infinite;
  }
}
@media (prefers-reduced-motion: reduce) {
  .schip-pushing .schip-icon {
    animation: none;
  }
  .schip-pushing {
    animation: none;
  }
}
</style>