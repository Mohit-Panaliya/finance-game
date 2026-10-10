/**
 * PWA install-prompt adapter.
 *
 * The More sheet in App.vue references this through optional chaining and only
 * renders the "Install app" row when `isInstallable` is true, so a build that
 * drops this composable keeps working — the entry simply never appears.
 */
import { computed, readonly, ref } from 'vue'

interface BeforeInstallPromptEvent extends Event {
  prompt(): Promise<void>
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed'; platform: string }>
}

const canPrompt = ref(false)
const installed = ref(false)
let deferred: BeforeInstallPromptEvent | null = null
let listenersBound = false

function bind(): void {
  if (listenersBound || typeof window === 'undefined') return
  listenersBound = true
  window.addEventListener('beforeinstallprompt', (e) => {
    e.preventDefault()
    deferred = e as BeforeInstallPromptEvent
    canPrompt.value = true
  })
  window.addEventListener('appinstalled', () => {
    installed.value = true
    canPrompt.value = false
    deferred = null
  })
}

async function install(): Promise<boolean> {
  const p = deferred
  if (!p) return false
  deferred = null
  canPrompt.value = false
  await p.prompt()
  return true
}

export function usePwaInstall() {
  bind()
  return readonly({
    install,
    installed: readonly(installed),
    isInstallable: computed(() => canPrompt.value && !installed.value)
  })
}