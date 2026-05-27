<script setup lang="ts">
import { useMedications } from '@/stores/medications'
import { useUI } from '@/stores/ui'
import { computed } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { ArrowUpDown, Bell, BellOff } from 'lucide-vue-next'
import AddMedicationModal from '@/components/AddMedicationModal.vue'
import { formatAmount } from '@/api/base'
import type { MedicationWithStats } from '@/types/medication'
import ClipboardButton from '@/components/ClipboardButton.vue'
import { useSnooze } from '@/composables/useSnooze'
import SnoozeButton from '@/components/SnoozeButton.vue'
import { storeToRefs } from 'pinia'
import { useToast } from '@/composables/useToast'

const route = useRoute()
const router = useRouter()
const showModal = computed(() => route.query.add === 'true')

const { data, isLoading, error } = useMedications()
const { snooze, unSnooze, isSnoozed } = useSnooze()
const uiStore = useUI()
const { toggleSortOrder } = uiStore
const { sortOrder } = storeToRefs(uiStore)

const { addToast } = useToast()

const filteredMedications = computed(() =>
  [...(data.value ?? [])]
    .filter((m) => m.daysRemaining <= m.warningThreshold && !isSnoozed.value(m.id))
    .sort((a, b) => a.name.localeCompare(b.name)),
)

const sortedMedications = computed(() => {
  if (sortOrder.value === 'alphabetical') {
    return [...(data.value ?? [])].sort((a, b) => a.name.localeCompare(b.name))
  } else {
    const { warned, snoozed, good } = [...(data.value ?? [])]
      .sort((a, b) => a.daysRemaining - b.daysRemaining)
      .reduce(
        (groups, m) => {
          if (m.daysRemaining > m.warningThreshold) groups.good.push(m)
          else if (isSnoozed.value(m.id)) groups.snoozed.push(m)
          else groups.warned.push(m)
          return groups
        },
        { warned: [], snoozed: [], good: [] } as Record<
          'warned' | 'snoozed' | 'good',
          MedicationWithStats[]
        >,
      )
    return [...warned, ...snoozed, ...good]
  }
})

const clipboardText = computed(() =>
  filteredMedications.value.length === 1
    ? filteredMedications.value[0]!.name
    : filteredMedications.value.map((m) => `- ${m.name}`).join('\n'),
)

const snoozeAll = () => {
  const count = filteredMedications.value.length
  filteredMedications.value.forEach((m) => snooze(m.id))
  addToast(`${count} medications snoozed`, 'info')
}
</script>

<template>
  <div v-if="isLoading" class="flex justify-center py-12">
    <div class="w-8 h-8 border-4 border-gray-200 border-t-blue-500 rounded-full animate-spin" />
  </div>
  <div v-else-if="error" class="text-center py-12 text-gray-500">
    Something went wrong. Please try again later.
  </div>
  <div v-else-if="sortedMedications.length === 0" class="text-center py-12 text-gray-400">
    No medications yet. Add one with the + button.
  </div>
  <div v-else>
    <div
      v-if="filteredMedications.length > 0"
      class="bg-red-50 border border-red-200 rounded-lg px-5 py-4 mb-6"
    >
      <div class="flex items-center justify-between mb-3">
        <span class="font-medium text-red-800">Running low</span>
        <div class="flex items-center gap-2">
          <SnoozeButton @snooze="snoozeAll" />
          <ClipboardButton :text="clipboardText" />
        </div>
      </div>
      <p class="text-sm text-red-700">
        {{ filteredMedications.map((m) => `${m.name} (${m.daysRemaining}d)`).join(' · ') }}
      </p>
    </div>

    <div class="flex items-center justify-between mb-4">
      <button
        @click="toggleSortOrder"
        class="flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-700 transition-colors"
      >
        <ArrowUpDown class="w-4 h-4" />
        {{ sortOrder === 'alphabetical' ? 'A–Z' : 'By urgency' }}
      </button>
    </div>

    <ul class="grid grid-cols-1 sm:grid-cols-2 gap-3 items-stretch">
      <li v-for="medication in sortedMedications" :key="medication.id">
        <RouterLink
          :to="{ name: 'medications-details', params: { id: medication.id } }"
          :class="[
            'h-full flex flex-col bg-white rounded-lg border border-gray-200 px-5 py-4 hover:shadow-sm transition-all border-t-4',
            medication.daysRemaining <= medication.warningThreshold
              ? isSnoozed(medication.id)
                ? 'border-t-amber-500'
                : 'border-t-red-400'
              : 'border-t-green-500',
          ]"
        >
          <div class="flex items-center justify-between">
            <span class="font-medium text-gray-900">{{ medication.name }}</span>
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-500">{{ medication.daysRemaining }} days</span>
              <button
                v-if="
                  isSnoozed(medication.id) &&
                  medication.daysRemaining <= medication.warningThreshold
                "
                @click.prevent.stop="unSnooze(medication.id)"
              >
                <BellOff class="w-4 h-4 text-amber-500" />
              </button>
              <button
                v-else-if="medication.daysRemaining <= medication.warningThreshold"
                @click.prevent.stop="snooze(medication.id)"
              >
                <Bell class="w-4 h-4 text-red-400" />
              </button>
            </div>
          </div>
          <div class="mt-1 mb-3 text-sm text-gray-500">
            <span class="text-lg mr-1">{{ formatAmount(medication.stock) }}</span>
            {{ medication.unit }} remaining
          </div>
          <div class="mt-auto h-1.5 bg-gray-200 rounded-full overflow-hidden">
            <div
              class="h-full rounded-full transition-all"
              :class="
                medication.daysRemaining <= medication.warningThreshold
                  ? isSnoozed(medication.id)
                    ? 'bg-amber-500'
                    : 'bg-red-400'
                  : 'bg-green-500'
              "
              :style="{
                width: `${Math.min(medication.daysRemaining / (medication.warningThreshold * 4), 1) * 100}%`,
              }"
            />
          </div>
        </RouterLink>
      </li>
    </ul>
  </div>
  <RouterLink
    v-if="!isLoading && !error"
    :to="{ name: 'dashboard', query: { add: 'true' } }"
    class="fixed bottom-6 right-6 bg-amber-400 text-white rounded-full w-14 h-14 flex items-center justify-center shadow-lg text-2xl hover:bg-amber-500 transition-colors"
  >
    +
  </RouterLink>
  <Teleport to="body">
    <div
      v-if="showModal"
      class="fixed inset-0 bg-black/50 flex items-end sm:items-center justify-center z-50"
      @click.self="router.back()"
    >
      <div class="bg-white w-full sm:max-w-md sm:rounded-xl rounded-t-xl p-6">
        <AddMedicationModal />
      </div>
    </div>
  </Teleport>
</template>

<style scoped></style>
