import { useQueryCache } from '@pinia/colada'
import { onUnmounted } from 'vue'
import { MEDICATION_KEYS } from '@/stores/medications'

export function useMidnightRefresh() {
  const cache = useQueryCache()

  function msUntilMidnight() {
    const now = new Date()
    const midnight = new Date(now)
    midnight.setHours(24, 0, 0, 0)
    return midnight.getTime() - now.getTime()
  }

  let timeout: ReturnType<typeof setTimeout>

  function scheduleRefresh() {
    timeout = setTimeout(() => {
      cache.invalidateQueries({ key: MEDICATION_KEYS.root })
      scheduleRefresh()
    }, msUntilMidnight())
  }

  scheduleRefresh()
  onUnmounted(() => clearTimeout(timeout))
}
