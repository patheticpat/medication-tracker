<script setup lang="ts">
import { createLogEntry, deleteMedication, getMedicationDetails } from '@/api/medications'
import { MEDICATION_KEYS } from '@/stores/medications'
import type { CreateLogEntry } from '@/types/medication'
import { useMutation, useQuery, useQueryCache } from '@pinia/colada'
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Trash2, Pencil } from 'lucide-vue-next'
import EditMedicationModal from '@/components/EditMedicationModal.vue'
import { formatAmount } from '@/api/base'
import ClipboardButton from '@/components/ClipboardButton.vue'
import { useSnooze } from '@/composables/useSnooze'

const router = useRouter()
const route = useRoute()
const cache = useQueryCache()
const showEditModal = computed(() => route.query.edit === 'true')
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
  onSettled: () => cache.invalidateQueries({ key: MEDICATION_KEYS.root }),
})

const { isSnoozed } = useSnooze()

const confirmDelete = ref(false)

const handleDelete = async () => {
  if (!confirmDelete.value) {
    confirmDelete.value = true
    setTimeout(() => (confirmDelete.value = false), 3000)
    return
  }
  await deleteAsync()
  await router.replace({ name: 'dashboard' })
  cache.invalidateQueries({ key: MEDICATION_KEYS.root })
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
  const dosage = `${m.schedule.amount} ${m.unit}`
  if (m.schedule.kind === 'weekly') {
    const weekdays = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']
    const day = weekdays[m.schedule.dayOfWeek]
    return `${dosage} per week (every ${day})`
  } else {
    return `${dosage} per day`
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
    <div class="flex items-start justify-between mb-6">
      <div>
        <div class="flex items-center gap-2">
          <h1 class="text-2xl font-bold text-gray-900">{{ medication.name }}</h1>
          <RouterLink
            :to="{
              name: 'medications-details',
              params: { id: medication.id },
              query: { edit: 'true' },
            }"
            class="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <Pencil class="w-4 h-4" />
          </RouterLink>
          <ClipboardButton :text="medication.name" />
        </div>
        <p class="text-gray-500 mt-1">{{ schedule }}</p>
      </div>
      <button
        @click="handleDelete"
        class="flex items-center gap-1.5 transition-colors p-1"
        :class="confirmDelete ? 'text-red-600' : 'text-red-400 hover:text-red-600'"
      >
        <Trash2 class="w-5 h-5" />
        <span v-if="confirmDelete" class="text-sm font-medium">Confirm?</span>
      </button>
    </div>

    <!-- Stock Info -->
    <div class="bg-white rounded-lg border border-gray-200 px-5 py-4 mb-6">
      <div class="flex justify-between items-center">
        <span class="text-gray-500 text-sm">Current stock</span>
        <span class="font-medium text-gray-900"
          >{{ formatAmount(medication.stock) }} {{ medication.unit }}</span
        >
      </div>
      <div class="flex justify-between items-center mt-2">
        <span class="text-gray-500 text-sm">Days remaining</span>
        <span
          class="font-medium"
          :class="
            medication.daysRemaining <= medication.warningThreshold
              ? isSnoozed(medication.id)
                ? 'text-amber-500'
                : 'text-red-500'
              : 'text-green-600'
          "
        >
          {{ medication.daysRemaining }} days
        </span>
      </div>
      <div class="flex justify-between items-center mt-2">
        <span class="text-gray-500 text-sm">Warning threshold</span>
        <span class="font-medium text-gray-900"
          >{{ formatAmount(medication.warningThreshold) }} days</span
        >
      </div>
    </div>
    <div class="grid grid-cols-2 gap-3 mb-6">
      <!-- Refill -->
      <div class="bg-white rounded-lg border border-gray-200 px-5 py-4">
        <h2 class="font-medium text-gray-900 mb-3">Refill</h2>
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
            placeholder="Note (optional)"
            class="border border-gray-200 rounded-md px-3 py-1.5 text-sm w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            type="submit"
            class="bg-emerald-600 text-white text-sm rounded-md px-3 py-1.5 hover:bg-emerald-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            :disabled="refillAmount <= 0"
          >
            Add refill
          </button>
        </form>
      </div>

      <!-- Recount -->
      <div class="bg-white rounded-lg border border-gray-200 px-5 py-4">
        <h2 class="font-medium text-gray-900 mb-3">Recount</h2>
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
            placeholder="Note (optional)"
            class="border border-gray-200 rounded-md px-3 py-1.5 text-sm w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            type="submit"
            class="bg-amber-500 text-white text-sm rounded-md px-3 py-1.5 hover:bg-amber-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            :disabled="baselineAmount < 0"
          >
            Set baseline
          </button>
        </form>
      </div>
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
            {{ l.kind }}
          </span>
          <div class="flex flex-col">
            <span class="text-sm text-gray-500">{{ l.date }}</span>
            <span v-if="l.note" class="text-xs text-gray-400">{{ l.note }}</span>
          </div>
        </div>
        <span class="text-sm font-medium text-gray-900"
          >{{ l.kind === 'refill' ? '+ ' : '' }}{{ formatAmount(l.amount) }}
          {{ medication.unit }}</span
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
