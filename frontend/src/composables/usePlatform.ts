import { computed, ref, readonly } from 'vue'
import { isPlatform } from '@ionic/vue'

/**
 * Platform split for the app chrome.
 *
 * Windows/desktop gets a dense, detailed layout (fixed sidebar, wide grids,
 * table-style rows). Android keeps the touch layout it has always had (bottom
 * tab bar, single column) so the PWA feels native on a phone.
 */
function detectWindows(): boolean {
  if (typeof navigator === 'undefined') return false
  const ua = navigator.userAgent.toLowerCase()
  return ua.includes('windows')
}

function detectMac(): boolean {
  if (typeof navigator === 'undefined') return false
  const ua = navigator.userAgent.toLowerCase()
  return ua.includes('mac')
}

// `isPlatform('desktop')` is true for any non-touch laptop, including the
// Windows machines this app is mostly used on.
const desktop = ref(isPlatform('desktop') && typeof window !== 'undefined')
const windows = ref(detectWindows())
const mac = ref(detectMac())
const android = ref(isPlatform('android'))
const ios = ref(isPlatform('ios'))

/**
 * Ionic swaps between its iOS and Material component styles at runtime, so a
 * Windows browser has to be told which one to render.
 */
const ionicMode = computed<'ios' | 'md'>(() => {
  if (desktop.value) return 'md'
  return android.value ? 'md' : 'ios'
})

export interface PlatformInfo {
  /** Any non-touch laptop layout (sidebar, dense tables). */
  isDesktop: boolean
  isWindows: boolean
  isMac: boolean
  isAndroid: boolean
  isIOS: boolean
  /** Touch layout: bottom tabs, single column. */
  isMobile: boolean
  ionicMode: 'ios' | 'md'
  /** Root class applied to <ion-app> so CSS can branch without JS in templates. */
  platformClass: string
}

const info: PlatformInfo = {
  get isDesktop() {
    return desktop.value
  },
  get isWindows() {
    return windows.value
  },
  get isMac() {
    return mac.value
  },
  get isAndroid() {
    return android.value
  },
  get isIOS() {
    return ios.value
  },
  get isMobile() {
    return !desktop.value
  },
  get ionicMode() {
    return ionicMode.value
  },
  get platformClass() {
    if (desktop.value) return 'platform-desktop'
    if (android.value) return 'platform-android'
    return 'platform-ios'
  }
}

export function usePlatform(): Readonly<PlatformInfo> {
  return readonly(info) as Readonly<PlatformInfo>
}

/**
 * Re-read platform signals once the app is running. Ionic reports `desktop`
 * from a media query, so the first paint can land before that settles.
 */
export function watchPlatform(): void {
  if (typeof window === 'undefined') return
  const sync = () => {
    desktop.value = isPlatform('desktop')
    windows.value = detectWindows()
    mac.value = detectMac()
    android.value = isPlatform('android')
    ios.value = isPlatform('ios')
  }
  window.addEventListener('resize', sync, { passive: true })
  window.addEventListener('orientationchange', sync, { passive: true })
  sync()
}
