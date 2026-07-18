<script setup lang="ts">
import { useMedications } from '@/stores/medications'
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { LoaderCircle } from 'lucide-vue-next'

const { data: medications, isLoading, error } = useMedications()

const normalizeDate = (date: Date): string => {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')

  return `${y}-${m}-${d}`
}

const getNextSunday = (date: Date): string => {
  const day = date.getDay()
  const diff = (7 - day) % 7
  date.setDate(date.getDate() + diff)
  return normalizeDate(date)
}

const today = new Date()
today.setHours(0, 0, 0, 0)

const groupedMedicationsByDate = computed(() => {
  const list = [...(medications.value ?? [])]
    .map((m) => {
      const reorderAfter = Math.max(0, m.daysRemaining - m.warningThreshold)
      const reorderOn = new Date(
        today.getFullYear(),
        today.getMonth(),
        today.getDate() + reorderAfter,
      )
      return { ...m, reorderOn: getNextSunday(reorderOn) }
    })
    .filter((m) => !m.snoozed)
    .sort((a, b) => {
      if (a.reorderOn == b.reorderOn) {
        return a.name.localeCompare(b.name)
      }
      return a.reorderOn.localeCompare(b.reorderOn)
    })

  if (list.length === 0) return []

  const groups = list.reduce(
    (acc, item) => {
      const key = item.reorderOn
      if (!acc[key]) acc[key] = []
      acc[key].push(item)
      return acc
    },
    {} as Record<string, (typeof list)[number][]>,
  )

  const dates = Object.keys(groups).sort((a, b) => a.localeCompare(b))

  const minDate = getNextSunday(today)
  const maxDate = dates[dates.length - 1]!
  const result = []

  const addOneWeek = (date: string): string => {
    const d = new Date(date)
    d.setDate(d.getDate() + 7)
    return normalizeDate(d)
  }

  for (let t = minDate; t <= maxDate; t = addOneWeek(t)) {
    result.push({ date: t, items: groups[t] ?? [] })
  }

  return result
})
</script>

<template>
  <div v-if="isLoading" class="flex justify-center py-12">
    <LoaderCircle class="animate-spin text-gray-400" />
  </div>
  <div v-else-if="error" class="text-center py-12 text-gray-500">
    {{ $t('strings.error') }}
  </div>
  <div v-else-if="groupedMedicationsByDate.length === 0" class="text-center py-12 text-gray-400">
    {{ $t('dashboard.noMedications') }}
  </div>
  <div v-else class="space-y-6">
    <div v-for="group in groupedMedicationsByDate" :key="group.date" class="agenda-group">
      <h2 class="font-medium text-gray-500 mb-3 pb-1 border-b border-gray-200">
        {{ $d(new Date(group.date), { dateStyle: 'long' }) }}
      </h2>
      <p v-if="group.items.length === 0" class="text-sm text-gray-400 italic px-1">
        {{ $t('dashboard.noMedicationsThisWeek') }}
      </p>
      <ul v-else class="mt-2 divide-y divide-gray-100">
        <li v-for="med in group.items" :key="med.id">
          <RouterLink
            :to="{ name: 'medications-details', params: { id: med.id } }"
            class="flex items-center py-3 px-1 hover:bg-gray-100 rounded-md transition-colors"
          >
            <span class="text-gray-900 font-medium">{{ med.name }}</span>
          </RouterLink>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped></style>
