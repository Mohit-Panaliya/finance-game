/**
 * PWA install-prompt adapter.
 *
 * Chromium fires `beforeinstallprompt`, which we stash so a user gesture can
 * replay it on demand. iOS Safari never fires it — installation there is the
 * manual Share → "Add to Home Screen" flow — so we expose `showIosHint` for a
 * guidance row instead. Once the app is installed (the `appinstalled` event, an
 * accepted prompt, or an already-standalone launch) the entry is dismissed
 * permanently through a localStorage flag.
 */
import { computed, readonly, ref } from 'vue'

interface BeforeInstallPromptEvent extends Event {
  prompt(): Promise<void>
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed'; platform: string }>
}

const DISMISS_KEY = 'fintrack:pwa-install-dismissed'

function readDismissed(): boolean {
  if (typeof localStorage === 'undefined') return false
  try {
    return localStorage.getItem(DISMISS_KEY) === '1'
  } catch {
    return false
  }
}

function detectStandalone(): boolean {
  if (typeof window === 'undefined') return false
  const nav = window.navigator as Navigator & { standalone?: boolean }
  return window.matchMedia?.('(display-mode: standalone)').matches === true || nav.standalone === true
}

function detectIos(): boolean {
  if (typeof navigator === 'undefined') return false
  const ua = navigator.userAgent
  const iOS = /iphone|ipad|ipod/i.test(ua)
  // iPadOS 13+ reports itself as desktop Safari; the touch-points check catches it.
  const iPadOs = navigator.platform === 'MacIntel' && (navigator.maxTouchPoints ?? 0) > 1
  return iOS || iPadOs
}

const canPrompt = ref(false)
const installed = ref(false)
const dismissed = ref(readDismissed())
const standalone = ref(detectStandalone())
const isIOS = ref(detectIos())

let deferred: BeforeInstallPromptEvent | null = null
let listenersBound = false

function persistDismiss(): void {
  dismissed.value = true
  try {
    localStorage.setItem(DISMISS_KEY, '1')
  } catch {
    /* storage blocked: dismissal stays in-memory for this session */
  }
}

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
    persistDismiss()
  })
  const mq = window.matchMedia?.('(display-mode: standalone)')
  mq?.addEventListener?.('change', (e) => {
    standalone.value = e.matches
  })
}

async function promptInstall(): Promise<'accepted' | 'dismissed' | 'unavailable'> {
  const p = deferred
  if (!p) return 'unavailable'
  deferred = null
  canPrompt.value = false
  await p.prompt()
  const { outcome } = await p.userChoice
  if (outcome === 'accepted') {
    installed.value = true
    persistDismiss()
  }
  return outcome
}

export function usePwaInstall() {
  bind()
  return readonly({
    promptInstall,
    dismiss: persistDismiss,
    isInstallable: computed(
      () => canPrompt.value && !installed.value && !dismissed.value && !standalone.value
    ),
    showIosHint: computed(
      () => isIOS.value && !installed.value && !standalone.value && !dismissed.value
    ),
    isIOS: readonly(isIOS),
    isStandalone: readonly(standalone),
    installed: readonly(installed),
    isDismissed: readonly(dismissed)
  })
}
