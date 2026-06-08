<script setup lang="ts">
import { useMedications, useUpdateSnooze } from '@/stores/medications'
import { useUI } from '@/stores/ui'
import { computed } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { ArrowUpDown, Bell, BellOff, AlertTriangle, Plus } from 'lucide-vue-next'
import AddMedicationModal from '@/components/AddMedicationModal.vue'
import { formatUnit } from '@/utils/format'
import type { MedicationWithStats } from '@/types/medication'
import ClipboardButton from '@/components/ClipboardButton.vue'
import SnoozeButton from '@/components/SnoozeButton.vue'
import { storeToRefs } from 'pinia'
import { useToast } from '@/composables/useToast'
import { useI18n } from 'vue-i18n'

const route = useRoute()
const router = useRouter()
const showModal = computed(() => route.query.add === 'true')

const { data, isLoading, error } = useMedications()
const uiStore = useUI()
const { toggleSortOrder } = uiStore
const { sortOrder } = storeToRefs(uiStore)

const { addToast } = useToast()
const { t } = useI18n()

const filteredMedications = computed(() =>
  [...(data.value ?? [])]
    .filter((m) => m.daysRemaining <= m.warningThreshold && !m.snoozed)
    .sort((a, b) => a.name.localeCompare(b.name)),
)

const { mutate: updateSnooze } = useUpdateSnooze()

const sortedMedications = computed(() => {
  if (sortOrder.value === 'alphabetical') {
    return [...(data.value ?? [])].sort((a, b) => a.name.localeCompare(b.name))
  } else {
    const { warned, snoozed, good } = [...(data.value ?? [])]
      .sort((a, b) => a.daysRemaining - b.daysRemaining)
      .reduce(
        (groups, m) => {
          if (m.daysRemaining > m.warningThreshold) groups.good.push(m)
          else if (m.snoozed) groups.snoozed.push(m)
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
  filteredMedications.value.forEach((m) => updateSnooze({ id: m.id, snoozed: true }))
  addToast(t('dashboard.medicationsSnoozed', count), 'info')
}
</script>

<template>
  <div v-if="isLoading">
    <div class="h-4 bg-gray-200 rounded w-16 mb-4 animate-pulse" />
    <ul class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <li v-for="i in 7" :key="i" class="animate-pulse">
        <div
          class="bg-white rounded-lg border border-gray-200 border-t-4 border-t-gray-200 px-5 py-4"
        >
          <div class="flex items-center justify-between mb-3">
            <div class="h-6 bg-gray-200 rounded w-40" />
            <div class="h-6 bg-gray-200 rounded w-12" />
          </div>
          <div class="flex items-end gap-2 mb-4">
            <div class="h-4 bg-gray-200 rounded w-52" />
          </div>
          <div class="h-1.5 bg-gray-200 rounded-full" />
        </div>
      </li>
    </ul>
  </div>
  <div v-else-if="error" class="text-center py-12 text-gray-500">
    {{ $t('strings.error') }}
  </div>
  <div v-else-if="sortedMedications.length === 0" class="text-center py-12 text-gray-400">
    {{ $t('dashboard.noMedications') }}
  </div>
  <div v-else>
    <div
      v-if="filteredMedications.length > 0"
      class="bg-red-50 border border-red-200 rounded-lg px-4 py-2 mb-6"
    >
      <div class="flex items-center justify-start mb-4">
        <AlertTriangle class="text-red-800 me-2" />
        <span class="font-bold text-red-800"> {{ $t('dashboard.runningLow') }}</span>
      </div>
      <p class="text-sm text-red-700 mb-4">
        {{ filteredMedications.map((m) => `${m.name} (${m.daysRemaining}d)`).join(' · ') }}
      </p>
      <div class="flex items-center gap-2 justify-end">
        <SnoozeButton @snooze="snoozeAll" />
        <ClipboardButton :text="clipboardText" />
      </div>
    </div>

    <div class="flex items-center justify-between mb-4">
      <button
        @click="toggleSortOrder"
        class="flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-700 transition-colors"
      >
        <ArrowUpDown class="w-4 h-4" />
        {{
          sortOrder === 'alphabetical'
            ? $t('dashboard.sort.alphabetical')
            : $t('dashboard.sort.urgency')
        }}
      </button>
    </div>

    <ul class="grid grid-cols-1 sm:grid-cols-2 gap-3 items-stretch">
      <li v-for="medication in sortedMedications" :key="medication.id">
        <RouterLink
          :to="{ name: 'medications-details', params: { id: medication.id } }"
          :class="[
            'h-full flex flex-col bg-white rounded-lg border border-gray-200 px-5 py-4 hover:shadow-sm transition-all border-t-4',
            medication.daysRemaining <= medication.warningThreshold
              ? medication.snoozed
                ? 'border-t-amber-500'
                : 'border-t-red-400'
              : 'border-t-green-500',
          ]"
        >
          <div class="flex items-center justify-between">
            <span class="font-medium text-gray-900">{{ medication.name }}</span>
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-500"
                >{{ medication.daysRemaining }}
                {{ $t('strings.day', medication.daysRemaining) }}</span
              >
              <button
                v-if="medication.snoozed && medication.daysRemaining <= medication.warningThreshold"
                @click.prevent.stop="updateSnooze({ id: medication.id, snoozed: false })"
              >
                <BellOff class="w-4 h-4 text-amber-500" />
              </button>
              <button
                v-else-if="medication.daysRemaining <= medication.warningThreshold"
                @click.prevent.stop="updateSnooze({ id: medication.id, snoozed: true })"
              >
                <Bell class="w-4 h-4 text-red-400" />
              </button>
            </div>
          </div>
          <div class="mt-1 mb-3 text-sm text-gray-500">
            <span class="text-lg mr-1">{{ $n(medication.stock, 'decimal') }}</span>
            {{ formatUnit(medication.stock, medication.unit, medication.unitSingular) }}
            {{ $t('dashboard.remaining') }}
          </div>
          <div class="mt-auto h-1.5 bg-gray-200 rounded-full overflow-hidden">
            <div
              class="h-full rounded-full transition-all"
              :class="
                medication.daysRemaining <= medication.warningThreshold
                  ? medication.snoozed
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
    class="fixed bottom-4 right-4 bg-amber-400 text-white rounded-full w-14 h-14 flex items-center justify-center shadow-[0_4px_14px_rgba(0,0,0,0.25)] hover:bg-amber-500 transition-colors"
  >
    <Plus />
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
