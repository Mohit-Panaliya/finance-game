<script setup lang="ts">
import { onBeforeUnmount, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    show: boolean
    title?: string
    dismissable?: boolean
    maxWidth?: string
  }>(),
  { title: '', dismissable: true, maxWidth: '440px' }
)

const emit = defineEmits<{ 'update:show': [value: boolean]; close: [] }>()

function close() {
  if (!props.dismissable) return
  emit('update:show', false)
  emit('close')
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}

watch(
  () => props.show,
  (v) => {
    if (v) document.addEventListener('keydown', onKey)
    else document.removeEventListener('keydown', onKey)
    document.body.style.overflow = v ? 'hidden' : ''
  },
  { immediate: true }
)

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKey)
  document.body.style.overflow = ''
})
</script>

<template>
  <teleport to="body">
    <transition name="gm-fade">
      <div v-if="show" class="gm-overlay" @click.self="close">
        <transition name="gm-pop" appear>
          <div
            class="gm-panel panel-stone noise"
            :style="{ maxWidth }"
            role="dialog"
            aria-modal="true"
          >
            <slot name="header">
              <div class="gm-banner">
                <span class="gm-banner-rope gm-banner-rope-l" />
                <span class="gm-banner-rope gm-banner-rope-r" />
                <div class="gm-banner-cloth">
                  <h2 class="gm-title carved carved-gold">{{ title }}</h2>
                </div>
                <button class="gm-close" type="button" aria-label="Close" @click="close">
                  <span class="gm-close-x">×</span>
                </button>
              </div>
            </slot>
            <button
              v-if="!$slots.header"
              class="gm-close gm-close-float"
              type="button"
              aria-label="Close"
              @click="close"
            >
              <span class="gm-close-x">×</span>
            </button>

            <div class="gm-body">
              <slot />
            </div>

            <div v-if="$slots.footer" class="gm-footer">
              <slot name="footer" />
            </div>
          </div>
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<style scoped>
.gm-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: radial-gradient(circle at 50% 30%, rgba(60, 35, 10, 0.75), rgba(8, 4, 0, 0.92));
  backdrop-filter: blur(7px);
  -webkit-backdrop-filter: blur(7px);
}
.gm-panel {
  position: relative;
  width: 100%;
  max-height: min(86vh, 720px);
  display: flex;
  flex-direction: column;
  padding: 16px 16px 18px;
  border: 4px solid var(--ff-gold-dark);
  box-shadow:
    inset 0 2px 0 rgba(255, 220, 150, 0.3),
    inset 0 -8px 16px rgba(0, 0, 0, 0.6),
    0 0 0 3px rgba(245, 197, 66, 0.35),
    0 16px 40px rgba(0, 0, 0, 0.75);
}
.gm-banner {
  position: relative;
  display: flex;
  justify-content: center;
  margin: -4px 0 14px;
  min-height: 56px;
  align-items: center;
}
.gm-banner-cloth {
  position: relative;
  background: linear-gradient(180deg, #a33 0%, #7a1f1f 60%, #5c1515 100%);
  border: 3px solid var(--ff-brown-deep);
  border-radius: 12px;
  padding: 8px 46px 10px;
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.3),
    0 5px 0 #3a0d0d,
    0 8px 16px rgba(0, 0, 0, 0.55);
  text-align: center;
  min-width: 60%;
}
.gm-title {
  margin: 0;
  font-size: 20px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.gm-banner-rope {
  position: absolute;
  top: -10px;
  width: 4px;
  height: 18px;
  background: var(--ff-gold-dark);
  border-radius: 2px;
}
.gm-banner-rope-l { left: 30%; }
.gm-banner-rope-r { right: 30%; }
.gm-close {
  position: absolute;
  top: -8px;
  right: -6px;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  border: 3px solid var(--ff-brown-deep);
  background: radial-gradient(circle at 35% 30%, #ff8d84, var(--ff-red) 55%, var(--ff-red-dark));
  color: #fff;
  font-size: 26px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow:
    inset 0 2px 0 rgba(255, 255, 255, 0.5),
    0 4px 0 var(--ff-red-dark),
    0 6px 12px rgba(0, 0, 0, 0.5);
  transition: transform 0.35s var(--ff-bounce);
  z-index: 3;
  padding: 0;
}
.gm-close:hover,
.gm-close:active {
  transform: rotate(180deg) scale(1.08);
}
.gm-close-x {
  font-family: var(--ff-font);
  transform: translateY(-2px);
  text-shadow: 0 2px 0 rgba(0, 0, 0, 0.45);
}
.gm-close-float {
  top: 10px;
  right: 10px;
}
.gm-body {
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  color: var(--ff-parchment);
  font-family: var(--ff-font-body);
  padding: 2px 4px;
}
.gm-footer {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  flex-wrap: wrap;
  margin-top: 16px;
  padding-top: 12px;
  border-top: 3px dashed rgba(245, 197, 66, 0.35);
}
.gm-fade-enter-active,
.gm-fade-leave-active {
  transition: opacity 0.25s ease;
}
.gm-fade-enter-from,
.gm-fade-leave-to {
  opacity: 0;
}
.gm-pop-enter-active {
  animation: gm-pop-in 0.5s var(--ff-bounce) both;
}
.gm-pop-leave-active {
  animation: gm-pop-in 0.22s ease reverse both;
}
@keyframes gm-pop-in {
  0% {
    transform: scale(0.55) translateY(60px);
    opacity: 0;
  }
  70% {
    transform: scale(1.05) translateY(-8px);
    opacity: 1;
  }
  100% {
    transform: scale(1) translateY(0);
    opacity: 1;
  }
}
</style>
