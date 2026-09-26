<script setup lang="ts">
import { computed } from 'vue'
import { useSyncStore } from '@/stores/syncStore'

const sync = useSyncStore()

const label = computed(() => {
  switch (sync.status) {
    case 'offline':
      return sync.queueCount ? `Offline · ${sync.queueCount}` : 'Offline'
    case 'pushing':
      return 'Pushing…'
    case 'synced':
      return 'Synced'
    case 'error':
      return 'Sync error'
    default:
      return 'Online'
  }
})

const cls = computed(() => ({
  'sync-chip': true,
  [`sync-${sync.status}`]: true,
  'sync-pulse': sync.status === 'pushing' || sync.status === 'offline'
}))
</script>

<template>
  <button :class="cls" type="button" title="Tap to sync" @click="sync.syncAll()">
    <span class="sync-dot" />
    <span class="sync-label">{{ label }}</span>
  </button>
</template>

<style scoped>
.sync-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  border-radius: 999px;
  border: 2px solid #0a0500;
  background: linear-gradient(180deg, #241405, #140b02);
  color: var(--ff-parchment);
  font-family: var(--ff-font);
  font-size: 11px;
  letter-spacing: 0.05em;
  cursor: pointer;
  box-shadow: 0 3px 0 rgba(0, 0, 0, 0.6);
  transition: transform 0.2s var(--ff-bounce);
  z-index: 5;
}
.sync-chip:active {
  transform: translateY(3px);
  box-shadow: none;
}
.sync-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #888;
  box-shadow: 0 0 6px currentColor;
}
.sync-online .sync-dot { background: var(--ff-green); color: var(--ff-green); }
.sync-synced .sync-dot { background: var(--ff-gold); color: var(--ff-gold); }
.sync-offline .sync-dot { background: var(--ff-red); color: var(--ff-red); }
.sync-pushing .sync-dot { background: var(--ff-blue); color: var(--ff-blue); }
.sync-error .sync-dot { background: var(--ff-red); color: var(--ff-red); }
.sync-pulse .sync-dot {
  animation: pulse-glow 1.1s ease-in-out infinite;
}
.sync-label {
  text-shadow: 0 1px 0 #000;
}
</style>
