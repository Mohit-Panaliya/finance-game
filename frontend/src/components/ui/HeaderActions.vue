<script setup lang="ts">
import { useRouter } from 'vue-router'
import { IonIcon } from '@ionic/vue'
import AppButton from '@/components/ui/AppButton.vue'
import SyncChip from '@/components/ui/SyncChip.vue'
import type { SyncStatus } from '@/stores/syncStore'

export interface HeaderAction {
  id: string
  icon: string
  label: string
  /** Push a route (mutually exclusive with `action`). */
  to?: string
  /** Run an imperative handler such as opening a modal or toggling a filter. */
  action?: () => void | Promise<void>
  /** Styles the button as pressed (filters, density toggles). */
  active?: boolean
  /** Hidden action — rendered but inert, matching the AppButton contract. */
  disabled?: boolean
}

const props = withDefaults(
  defineProps<{
    actions?: HeaderAction[]
    status?: SyncStatus
    pending?: number
    lastSyncAt?: number | null
  }>(),
  { actions: () => [], status: 'online', pending: 0, lastSyncAt: null }
)

const emit = defineEmits<{ sync: [] }>()

const router = useRouter()

function run(a: HeaderAction) {
  if (a.disabled) return
  if (a.action) void a.action()
  else if (a.to) void router.push(a.to)
}
</script>

<template>
  <div class="page-actions">
    <AppButton
      v-for="a in actions"
      :key="a.id"
      class="ha-btn"
      :class="{ 'ha-btn-active': a.active }"
      variant="neutral"
      size="sm"
      :disabled="a.disabled"
      :aria-label="a.label"
      :title="a.label"
      @click="run(a)"
    >
      <ion-icon :icon="a.icon" />
    </AppButton>
    <SyncChip :status="status" :pending="pending" :last-sync-at="lastSyncAt" @sync="emit('sync')" />
  </div>
</template>

<style scoped>
.page-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
.ha-btn {
  width: 34px;
  min-width: 34px;
  padding: 0;
}
.ha-btn-active {
  background: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent-ink);
}
</style>