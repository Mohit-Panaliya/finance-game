<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from 'vue'
import { IonIcon } from '@ionic/vue'
import { closeOutline } from 'ionicons/icons'

const props = withDefaults(
  defineProps<{
    open: boolean
    title?: string
    subtitle?: string
    /** Sheet docks to the bottom of the screen; otherwise it is a centred dialog. */
    sheet?: boolean
  }>(),
  { open: false, title: '', subtitle: '', sheet: true }
)

const emit = defineEmits<{ close: [] }>()

const busy = ref(false)

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.open) emit('close')
}

watch(
  () => props.open,
  (isOpen) => {
    document.addEventListener('keydown', onKey)
    document.body.style.overflow = isOpen ? 'hidden' : ''
  }
)

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKey)
  document.body.style.overflow = ''
})
</script>

<template>
  <teleport to="body">
    <transition name="amd-fade">
      <div v-if="open" class="amd-overlay" @click.self="emit('close')">
        <transition :name="sheet ? 'amd-sheet' : 'amd-pop'" appear>
          <section class="amd-panel" :class="{ 'amd-panel-sheet': sheet }" role="dialog" aria-modal="true">
            <header class="amd-head">
              <div class="amd-head-text">
                <h2 class="amd-title clamp-1">{{ title }}</h2>
                <p v-if="subtitle" class="amd-subtitle clamp-1">{{ subtitle }}</p>
              </div>
              <button class="amd-x" type="button" aria-label="Close" :disabled="busy" @click="emit('close')">
                <ion-icon :icon="closeOutline" />
              </button>
            </header>

            <div class="amd-body ff-hide-scrollbar">
              <slot :busy="busy" :setBusy="(v: boolean) => (busy = v)" />
            </div>

            <footer v-if="$slots.footer" class="amd-foot">
              <slot name="footer" />
            </footer>
          </section>
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<style scoped>
.amd-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: var(--scrim);
  backdrop-filter: blur(3px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--density-page-pad);
}
.amd-panel-sheet {
  align-self: flex-end;
  width: min(560px, 100%);
  border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  padding-bottom: calc(14px + env(safe-area-inset-bottom));
  /* Cancels the overlay gutter so the sheet still docks flush to the edge. */
  margin-bottom: calc(-1 * var(--density-page-pad) - env(safe-area-inset-bottom));
}
.amd-panel {
  width: min(560px, 100%);
  max-height: 86vh;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}
.amd-head {
  display: flex;
  align-items: flex-start;
  gap: var(--density-gap);
  padding: var(--density-card-pad) var(--density-card-pad) var(--density-gap);
  border-bottom: 1px solid var(--border);
}
.amd-head-text {
  flex: 1;
  min-width: 0;
}
.amd-title {
  margin: 0;
  font-size: 1.08rem;
  font-weight: 650;
}
.amd-subtitle {
  margin: 3px 0 0;
  font-size: 0.8rem;
  color: var(--text-muted);
}
.amd-x {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 50%;
  color: var(--text-muted);
  font-size: 18px;
  cursor: pointer;
}
.amd-body {
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  padding: var(--density-card-pad);
}
.amd-foot {
  display: flex;
  gap: var(--density-gap);
  padding: var(--density-gap) var(--density-card-pad) 0;
  border-top: 1px solid var(--border);
}
.amd-fade-enter-active,
.amd-fade-leave-active {
  transition: opacity 0.18s ease;
}
.amd-fade-enter-from,
.amd-fade-leave-to {
  opacity: 0;
}
.amd-pop-enter-active,
.amd-pop-leave-active,
.amd-sheet-enter-active,
.amd-sheet-leave-active {
  transition: transform 0.22s ease;
}
.amd-pop-enter-from,
.amd-pop-leave-to {
  transform: scale(0.95);
}
.amd-sheet-enter-from,
.amd-sheet-leave-to {
  transform: translateY(100%);
}
</style>