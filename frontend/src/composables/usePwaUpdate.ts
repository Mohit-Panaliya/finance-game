/**
 * Service-worker update affordance.
 *
 * `vite-plugin-pwa` is configured with `registerType: 'prompt'`, so a newly
 * installed worker waits until the user opts in. `useRegisterSW` flips
 * `needRefresh` when that waiting worker appears; `refresh()` sends the
 * SKIP_WAITING message and reloads, taking over offline-first behaviour intact.
 *
 * The plugin reloads itself only when it can prove the worker is an update; a
 * freshly claimed client can miss that check, so we also watch `controllerchange`
 * — gated on an explicit user Refresh — and keep a short fallback timer.
 */
import { onMounted, onUnmounted } from 'vue'
import { useRegisterSW } from 'virtual:pwa-register/vue'

export function usePwaUpdate() {
  const { needRefresh, updateServiceWorker } = useRegisterSW({
    onRegisterError(error) {
      console.warn('[pwa] service worker registration failed', error)
    }
  })

  let reloadOnControllerChange = false

  function onControllerChange() {
    if (reloadOnControllerChange) window.location.reload()
  }

  onMounted(() => {
    if (typeof navigator !== 'undefined' && 'serviceWorker' in navigator) {
      navigator.serviceWorker.addEventListener('controllerchange', onControllerChange)
    }
  })

  onUnmounted(() => {
    if (typeof navigator !== 'undefined' && 'serviceWorker' in navigator) {
      navigator.serviceWorker.removeEventListener('controllerchange', onControllerChange)
    }
  })

  async function refresh(): Promise<void> {
    reloadOnControllerChange = true
    await updateServiceWorker(true)
    // Safety net: activate-without-controllerchange would otherwise strand the
    // banner on screen.
    window.setTimeout(() => {
      if (reloadOnControllerChange) window.location.reload()
    }, 3000)
  }

  return { needRefresh, refresh }
}
