/**
 * Service-worker update affordance.
 *
 * `vite-plugin-pwa` is configured with `registerType: 'prompt'`, so a newly
 * installed worker waits until the user opts in. `useRegisterSW` flips
 * `needRefresh` when that waiting worker appears; `refresh()` sends the
 * SKIP_WAITING message and reloads, taking over offline-first behaviour intact.
 */
import { useRegisterSW } from 'virtual:pwa-register/vue'

export function usePwaUpdate() {
  const { needRefresh, updateServiceWorker } = useRegisterSW({
    onRegisterError(error) {
      console.warn('[pwa] service worker registration failed', error)
    }
  })

  async function refresh(): Promise<void> {
    await updateServiceWorker(true)
  }

  return { needRefresh, refresh }
}
