<script setup lang="ts">
import type { Medication } from '@/types/medication'
import { useMedications } from '@/stores/medications'
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { LoaderCircle } from 'lucide-vue-next'

type MedicationWithAgenda = Medication & { reorderOn: number; reorderAfter: number }

const { data: medications, isLoading, error } = useMedications()

const today = new Date()
const filteredMedications = computed(() =>
  [...(medications.value ?? [])]
    .map((m) => {
      const reorderAfter = Math.max(0, m.daysRemaining - m.warningThreshold)
      const reorderOn = new Date(
        today.getFullYear(),
        today.getMonth(),
        today.getDate() + reorderAfter,
      ).getTime()
      return { ...m, reorderAfter, reorderOn }
    })
    .filter((m) => !m.snoozed)
    .sort((a, b) => {
      if (a.reorderAfter == b.reorderAfter) {
        return a.name.localeCompare(b.name)
      }
      return a.reorderAfter - b.reorderAfter
    }),
)

const groups = computed(() => {
  const groups = new Map<number, MedicationWithAgenda[]>()

  for (const m of filteredMedications.value) {
    const dateKey = m.reorderOn
    if (!groups.has(dateKey)) {
      groups.set(dateKey, [])
    }
    groups.get(dateKey)!.push(m)
  }

  return Array.from(groups.entries()).map(([date, medications]) => ({ date, medications }))
})
</script>

<template>
  <div v-if="isLoading" class="flex justify-center py-12">
    <LoaderCircle class="animate-spin text-gray-400" />
  </div>
  <div v-else-if="error" class="text-center py-12 text-gray-500">
    {{ $t('strings.error') }}
  </div>
  <div v-else-if="groups.length === 0" class="text-center py-12 text-gray-400">
    {{ $t('dashboard.noMedications') }}
  </div>
  <div v-else class="space-y-6">
    <div v-for="group in groups" :key="group.date" class="agenda-group">
      <h2 class="font-medium text-gray-500 mb-3 pb-1 border-b border-gray-200">
        {{ $d(group.date, { dateStyle: 'long' }) }}
      </h2>
      <ul class="mt-2 divide-y divide-gray-100">
        <li v-for="med in group.medications" :key="med.id">
          <RouterLink
            :to="{ name: 'medications-details', params: { id: med.id } }"
            class="flex items-center py-3 px-1 hover:bg-gray-100 rounded-md transition-colors"
          >
            <span class="text-gray-900 font-medium">{{ med.name }}</span>
            <span
              v-if="med.reorderAfter === 0"
              class="text-xs font-medium text-red-600 bg-red-50 ml-2 px-2 py-0.5 rounded-full"
            >
              {{ $t('dashboard.reorderNow') }}
            </span>
          </RouterLink>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped></style>
