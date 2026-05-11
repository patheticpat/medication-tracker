import { computed, ref, watch } from 'vue'

export function useSnooze() {
  const SNOOZE_KEY = 'medication-snoozes'
  const SNOOZE_DURATION_IN_DAYS = 7

  const loadSnoozes = () => {
    try {
      return JSON.parse(localStorage.getItem(SNOOZE_KEY) || '{}')
    } catch {
      return {}
    }
  }
  const snoozes = ref<Record<string, string>>(loadSnoozes())

  watch(snoozes, (newSnoozes) => localStorage.setItem(SNOOZE_KEY, JSON.stringify(newSnoozes)))

  const snooze = (id: string) => {
    const until = new Date()
    until.setDate(until.getDate() + SNOOZE_DURATION_IN_DAYS)

    snoozes.value = { ...snoozes.value, [id]: until.toLocaleDateString('en-CA') }
  }

  const unSnooze = (id: string) => {
    const { [id]: _, ...rest } = snoozes.value
    snoozes.value = rest
  }

  const isSnoozed = computed(() => (id: string) => {
    const until = snoozes.value[id]
    if (until === undefined) return false
    const today = new Date().toLocaleDateString('en-CA')
    if (until <= today) {
      unSnooze(id)
      return false
    }
    return true
  })

  return { isSnoozed, snooze, unSnooze }
}
