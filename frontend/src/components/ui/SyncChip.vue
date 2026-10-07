<script setup lang="ts">
import { computed } from 'vue'
import { IonIcon } from '@ionic/vue'
import { cloudDoneOutline, cloudOfflineOutline, cloudUploadOutline, syncOutline } from 'ionicons/icons'
import type { SyncStatus } from '@/stores/syncStore'

const props = withDefaults(defineProps<{ status?: SyncStatus; pending?: number }>(), {
  status: 'online',
  pending: 0
})

const emit = defineEmits<{ sync: [] }>()

const meta = computed(() => {
  switch (props.status) {
    case 'offline':
      return { label: 'Offline', icon: cloudOfflineOutline, tone: 'offline' }
    case 'pushing':
      return { label: 'Syncing', icon: cloudUploadOutline, tone: 'pushing' }
    case 'synced':
      return { label: 'Synced', icon: cloudDoneOutline, tone: 'synced' }
    default:
      return { label: 'Online', icon: syncOutline, tone: 'online' }
  }
})

const text = computed(() => {
  const n = Math.max(0, props.pending | 0)
  if (n > 0) return `${meta.value.label} · ${n} queued`
  return meta.value.label
})
</script>

<template>
  <button class="schip" :class="`schip-${meta.tone}`" type="button" @click="emit('sync')">
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
  font-family: var(--font-body);
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition:
    background 0.15s ease,
    border-color 0.15s ease;
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
.schip-pushing .schip-icon {
  animation: schip-spin 1.1s linear infinite;
}
.schip-offline {
  color: var(--danger);
  border-color: var(--danger-soft);
  background: var(--danger-soft);
}
@keyframes schip-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>