import { ref, unref, watch, onBeforeUnmount } from 'vue'
import type { Ref } from 'vue'

export function useCountUp(target: Ref<number> | (() => number), duration = 850): Ref<number> {
  const value = ref(0)
  let raf = 0

  const read = (): number => (typeof target === 'function' ? target() : unref(target))

  function animate() {
    cancelAnimationFrame(raf)
    const to = read()
    const from = value.value
    if (from === to) return
    const start = performance.now()
    const step = (now: number) => {
      const t = Math.min(1, (now - start) / duration)
      const eased = 1 - Math.pow(1 - t, 3)
      value.value = from + (to - from) * eased
      if (t < 1) raf = requestAnimationFrame(step)
      else value.value = to
    }
    raf = requestAnimationFrame(step)
  }

  watch(
    () => read(),
    () => animate(),
    { immediate: true }
  )

  onBeforeUnmount(() => cancelAnimationFrame(raf))
  return value
}
