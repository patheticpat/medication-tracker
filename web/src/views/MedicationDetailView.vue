<script setup lang="ts">
import { MEDICATION_KEYS, useUpdateSnooze } from '@/stores/medications'
import type { CreateLogEntry, MedicationWithStats } from '@/types/medication'
import { useMutation, useQuery, useQueryCache } from '@pinia/colada'
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Trash2, Pencil, Bell, BellOff } from 'lucide-vue-next'
import EditMedicationModal from '@/components/EditMedicationModal.vue'
import { formatUnit, parseDate } from '@/utils/format'
import ClipboardButton from '@/components/ClipboardButton.vue'
import { useToast } from '@/composables/useToast'
import { useApi } from '@/composables/useApi'
import { useI18n } from 'vue-i18n'

const { n, t } = useI18n()

const router = useRouter()
const route = useRoute()
const cache = useQueryCache()
const { addToast } = useToast()
const showEditModal = computed(() => route.query.edit === 'true')
const { getMedicationDetails, createLogEntry, deleteMedication } = useApi()
const {
  data: medication,
  isLoading,
  error,
} = useQuery({
  key: () => MEDICATION_KEYS.byId(route.params.id as string),
  query: () => getMedicationDetails(route.params.id as string),
})

const { mutateAsync: deleteAsync } = useMutation({
  mutation: () => deleteMedication(route.params.id as string),
})

const { mutateAsync: createLogAsync } = useMutation({
  mutation: (log: CreateLogEntry) => createLogEntry(route.params.id as string, log),
  onSuccess: (updatedMedication, log) => {
    addToast(log.kind === 'refill' ? 'Refill logged' : 'Stock updated', 'success')
    cache.setQueryData(MEDICATION_KEYS.byId(updatedMedication.id), updatedMedication)
    const { logs: _, ...medicationWithoutLogs } = updatedMedication
    cache.setQueryData(MEDICATION_KEYS.root, (medications?: MedicationWithStats[]) =>
      medications?.map((m) => (m.id === medicationWithoutLogs.id ? medicationWithoutLogs : m)),
    )
  },
})

const { mutate: updateSnooze } = useUpdateSnooze()

const confirmDelete = ref(false)

const handleDelete = async () => {
  if (!confirmDelete.value) {
    confirmDelete.value = true
    setTimeout(() => (confirmDelete.value = false), 3000)
    return
  }
  try {
    await deleteAsync()
    addToast(`${medication.value!.name} deleted`, 'success')
    await router.replace({ name: 'dashboard' })
    cache.invalidateQueries({ key: MEDICATION_KEYS.root })
  } catch {
    addToast(`Failed to delete ${medication.value!.name}`, 'error')
  }
}

const handleBaseline = async () => {
  await createLogAsync({
    kind: 'baseline',
    amount: baselineAmount.value,
    note: baselineNote.value || undefined,
  })
  baselineAmount.value = 0
  baselineNote.value = ''
}

const handleRefill = async () => {
  await createLogAsync({
    kind: 'refill',
    amount: refillAmount.value,
    note: refillNote.value || undefined,
  })
  refillAmount.value = 0
  refillNote.value = ''
}

const refillAmount = ref(0)
const refillNote = ref('')
const baselineAmount = ref(0)
const baselineNote = ref('')

const schedule = computed(() => {
  if (!medication.value) return ''
  const m = medication.value
  const amount = n(m.schedule.amount, 'dosage')
  const dosage = `${amount} ${formatUnit(m.schedule.amount, m.unit, m.unitSingular)}`
  if (m.schedule.kind === 'daily') {
    return t('medication.dailyDose', { dosage })
  } else {
    const weekdays = ['sunday', 'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday']
    const day = t(`strings.weekdays.on.${weekdays[m.schedule.dayOfWeek]}`)
    return t('medication.weeklyDose', { dosage, day })
  }
})

const logEntries = computed(() => {
  if (!medication.value || !medication.value.logs) return []
  return [...medication.value.logs].reverse()
})
</script>

<template>
  <div v-if="isLoading" class="flex justify-center py-12">
    <div class="w-8 h-8 border-4 border-gray-200 border-t-blue-500 rounded-full animate-spin" />
  </div>
  <div v-else-if="error" class="text-center py-12 text-gray-500">
    Something went wrong. Please try again later.
  </div>

  <div v-else-if="medication">
    <!-- Header -->
    <div class="mb-6">
      <div>
        <div class="flex items-center justify-between gap-2">
          <h1 class="text-2xl font-bold text-gray-900">{{ medication.name }}</h1>
          <ClipboardButton :text="medication.name" />
        </div>
        <p class="text-gray-500 mt-1">{{ schedule }}</p>
      </div>
    </div>

    <!-- Stock Info -->
    <div class="bg-white rounded-lg border border-gray-200 px-5 py-4 mb-6">
      <div class="flex justify-between items-center">
        <span class="text-gray-500 text-sm">{{ $t('medication.stock') }}</span>
        <span class="font-medium text-gray-900"
          >{{ $n(medication.stock, 'decimal') }}
          {{ formatUnit(medication.stock, medication.unit, medication.unitSingular) }}</span
        >
      </div>
      <div class="flex justify-between items-center mt-2">
        <span class="text-gray-500 text-sm">{{ $t('medication.daysRemaining') }}</span>
        <span
          class="font-medium"
          :class="
            medication.daysRemaining <= medication.warningThreshold
              ? medication.snoozed
                ? 'text-amber-500'
                : 'text-red-500'
              : 'text-green-600'
          "
        >
          {{ medication.daysRemaining }} {{ $t('strings.day', medication.daysRemaining) }}
        </span>
      </div>
      <div class="flex justify-between items-center mt-2">
        <span class="text-gray-500 text-sm">{{ $t('medication.threshold') }}</span>
        <span class="font-medium text-gray-900"
          >{{ $n(medication.warningThreshold, 'decimal') }}
          {{ $t('strings.day', medication.warningThreshold) }}</span
        >
      </div>
    </div>
    <div class="grid grid-cols-2 gap-3 mb-2">
      <!-- Refill -->
      <div class="bg-white rounded-lg border border-gray-200 px-5 py-4">
        <h2 class="font-medium text-gray-900 mb-3">{{ $t('medication.refill') }}</h2>
        <form @submit.prevent="handleRefill" class="flex flex-col gap-2">
          <input
            type="number"
            step="any"
            min="0"
            v-model.number="refillAmount"
            class="border border-gray-200 rounded-md px-3 py-1.5 text-sm w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            type="text"
            v-model.trim="refillNote"
            :placeholder="t('medication.note')"
            class="border border-gray-200 rounded-md px-3 py-1.5 text-sm w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            type="submit"
            class="bg-emerald-600 text-white text-sm rounded-md px-3 py-1.5 hover:bg-emerald-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            :disabled="refillAmount <= 0"
          >
            {{ $t('strings.save') }}
          </button>
        </form>
      </div>

      <!-- Recount -->
      <div class="bg-white rounded-lg border border-gray-200 px-5 py-4">
        <h2 class="font-medium text-gray-900 mb-3">{{ $t('medication.recount') }}</h2>
        <form @submit.prevent="handleBaseline" class="flex flex-col gap-2">
          <input
            type="number"
            step="any"
            min="0"
            v-model.number="baselineAmount"
            class="border border-gray-200 rounded-md px-3 py-1.5 text-sm w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            type="text"
            v-model.trim="baselineNote"
            :placeholder="t('medication.note')"
            class="border border-gray-200 rounded-md px-3 py-1.5 text-sm w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            type="submit"
            class="bg-amber-500 text-white text-sm rounded-md px-3 py-1.5 hover:bg-amber-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            :disabled="baselineAmount < 0"
          >
            {{ $t('strings.save') }}
          </button>
        </form>
      </div>
    </div>
    <!-- Aktionen -->
    <div class="mt-2 mb-6 pt-6 flex items-center justify-between">
      <RouterLink
        :to="{
          name: 'medications-details',
          params: { id: medication.id },
          query: { edit: 'true' },
        }"
        class="flex items-center gap-1.5 text-sm text-gray-600 hover:bg-gray-100 border border-gray-200 rounded-md px-3 py-1.5 transition-colors"
      >
        <Pencil class="w-4 h-4" />
        {{ $t('medication.editButton') }}
      </RouterLink>

      <button
        v-if="medication.snoozed && medication.daysRemaining <= medication.warningThreshold"
        @click="updateSnooze({ id: medication.id, snoozed: false })"
        class="flex items-center gap-1.5 text-sm text-amber-500 border border-amber-200 hover:bg-amber-50 rounded-md px-3 py-1.5 transition-colors"
      >
        <Bell class="w-4 h-4" />
        {{ $t('medication.unsnooze') }}
      </button>

      <button
        v-if="!medication.snoozed && medication.daysRemaining <= medication.warningThreshold"
        @click="updateSnooze({ id: medication.id, snoozed: true })"
        class="flex items-center gap-1.5 text-sm text-amber-500 border border-amber-200 hover:bg-amber-50 rounded-md px-3 py-1.5 transition-colors"
      >
        <BellOff class="w-4 h-4" />
        {{ $t('medication.snooze') }}
      </button>

      <button
        @click="handleDelete"
        class="flex items-center gap-1.5 text-sm transition-colors px-3 py-1.5 rounded-md border"
        :class="
          confirmDelete
            ? 'bg-red-600 text-white border-red-600'
            : 'text-red-500 border-red-200 hover:bg-red-50'
        "
      >
        <Trash2 class="w-4 h-4" />
        {{ confirmDelete ? $t('medication.confirmDelete') : $t('medication.delete') }}
      </button>
    </div>
    <h2 class="font-medium text-gray-900 mb-3">History</h2>
    <div class="bg-white rounded-lg border border-gray-200 divide-y divide-gray-100">
      <div v-for="l in logEntries" :key="l.id" class="px-5 py-3 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <span
            class="text-xs font-medium px-2 py-0.5 rounded-full w-16 text-center"
            :class="
              l.kind === 'refill'
                ? 'bg-emerald-100 text-emerald-700'
                : 'bg-amber-100 text-amber-700'
            "
          >
            {{ $t(`medication.action.${l.kind}`) }}
          </span>
          <div class="flex flex-col">
            <span class="text-sm text-gray-500">{{
              $d(parseDate(l.date), { dateStyle: 'long' })
            }}</span>
            <span v-if="l.note" class="text-xs text-gray-400">{{ l.note }}</span>
          </div>
        </div>
        <span class="text-sm font-medium text-gray-900"
          >{{ l.kind === 'refill' ? '+ ' : '' }}{{ $n(l.amount, 'decimal') }}
          {{ formatUnit(l.amount, medication.unit, medication.unitSingular) }}</span
        >
      </div>
    </div>
  </div>

  <Teleport to="body" v-if="medication">
    <div
      v-if="showEditModal"
      class="fixed inset-0 bg-black/50 flex items-end sm:items-center justify-center z-50"
      @click.self="router.back()"
    >
      <div class="bg-white w-full sm:max-w-md sm:rounded-xl rounded-t-xl p-6">
        <EditMedicationModal :medication="medication" />
      </div>
    </div>
  </Teleport>
</template>

<style scoped></style>
