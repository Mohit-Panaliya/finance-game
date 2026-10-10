<script setup lang="ts">
import { ref } from 'vue'
import { IonIcon } from '@ionic/vue'
import { closeOutline, refreshOutline } from 'ionicons/icons'
import AppButton from '@/components/ui/AppButton.vue'
import { usePwaUpdate } from '@/composables/usePwaUpdate'

const { needRefresh, refresh } = usePwaUpdate()
const hidden = ref(false)

function reload() {
  void refresh()
}
</script>

<template>
  <div
    v-if="needRefresh && !hidden"
    class="update-banner"
    role="status"
    aria-live="polite"
    aria-label="Application update available"
  >
    <ion-icon class="update-banner-icon" :icon="refreshOutline" />
    <span class="update-banner-text">New version available</span>
    <AppButton variant="primary" size="sm" @click="reload">Refresh</AppButton>
    <button
      class="update-banner-close"
      type="button"
      aria-label="Dismiss update"
      @click="hidden = true"
    >
      <ion-icon :icon="closeOutline" />
    </button>
  </div>
</template>

<style scoped>
.update-banner {
  position: fixed;
  left: 50%;
  bottom: calc(72px + env(safe-area-inset-bottom));
  transform: translateX(-50%);
  z-index: 40;
  display: flex;
  align-items: center;
  gap: var(--density-gap);
  width: max-content;
  max-width: min(92vw, 420px);
  padding: 9px 10px 9px 12px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
}
.update-banner-icon {
  flex: none;
  font-size: 18px;
  color: var(--accent);
}
.update-banner-text {
  flex: 1;
  min-width: 0;
  font-size: calc(0.84rem * var(--density-font-scale));
  font-weight: 600;
  color: var(--text);
}
.update-banner-close {
  flex: none;
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  padding: 0;
  background: transparent;
  border: 0;
  border-radius: var(--radius-sm);
  color: var(--text-faint);
  font-size: 18px;
  cursor: pointer;
}
.update-banner-close:hover {
  background: var(--surface-2);
  color: var(--text);
}
@media (prefers-reduced-motion: no-preference) {
  .update-banner {
    animation: update-banner-in 0.22s ease-out;
  }
  @keyframes update-banner-in {
    from {
      opacity: 0;
      transform: translate(-50%, 8px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }
}
</style>
