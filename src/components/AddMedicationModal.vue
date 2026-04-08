<script setup lang="ts">
import { createMedication } from '@/api/medications'
import { MEDICATION_KEYS } from '@/stores/medications'
import type { Schedule } from '@/types/medication'
import { useMutation, useQueryCache } from '@pinia/colada'
import { computed, nextTick, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { X } from 'lucide-vue-next'

const nameInput = ref<HTMLInputElement | null>(null)

onMounted(() => {
  nextTick(() => nameInput.value?.focus())
})

const router = useRouter()
const cache = useQueryCache()

const { mutateAsync, isLoading } = useMutation({
  mutation: createMedication,
  onSettled: () => cache.invalidateQueries({ key: MEDICATION_KEYS.root }),
})

const name = ref('')
const unit = ref('pills')
const warningThreshold = ref(21)
const initialStock = ref(0)
const scheduleAmount = ref(0)
const scheduleKind = ref<'daily' | 'weekly'>('daily')
const scheduleDayOfWeek = ref(1)

const isValid = computed(() => {
  return (
    name.value.length >= 3 &&
    unit.value.length > 0 &&
    initialStock.value >= 0 &&
    warningThreshold.value >= 0 &&
    scheduleAmount.value > 0
  )
})

const handleSubmit = async () => {
  const schedule: Schedule =
    scheduleKind.value === 'daily'
      ? { kind: 'daily', amount: scheduleAmount.value }
      : { kind: 'weekly', amount: scheduleAmount.value, dayOfWeek: scheduleDayOfWeek.value }
  await mutateAsync({
    name: name.value,
    unit: unit.value,
    warningThreshold: warningThreshold.value,
    initialStock: initialStock.value,
    schedule,
  })
  router.back()
}
</script>

<template>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-lg font-semibold text-gray-900">Add Medication</h2>
    <button @click="router.back()" class="text-gray-400 hover:text-gray-600 transition-colors">
      <X class="w-5 h-5" />
    </button>
  </div>

  <form @submit.prevent="handleSubmit" class="flex flex-col gap-4">
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1">
        <label class="text-sm text-gray-600">Name</label>
        <input
          type="text"
          ref="nameInput"
          v-model.trim="name"
          class="border border-gray-200 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm text-gray-600">Unit</label>
        <input
          type="text"
          v-model.trim="unit"
          class="border border-gray-200 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
    </div>

    <div class="flex flex-col gap-1">
      <label class="text-sm text-gray-600">Schedule</label>
      <div class="flex gap-2">
        <select
          v-model="scheduleKind"
          class="border border-gray-200 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="daily">Daily</option>
          <option value="weekly">Weekly</option>
        </select>
        <select
          v-if="scheduleKind === 'weekly'"
          v-model.number="scheduleDayOfWeek"
          class="border border-gray-200 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="1">Monday</option>
          <option value="2">Tuesday</option>
          <option value="3">Wednesday</option>
          <option value="4">Thursday</option>
          <option value="5">Friday</option>
          <option value="6">Saturday</option>
          <option value="0">Sunday</option>
        </select>
        <input
          type="number"
          step="any"
          min="0"
          v-model.number="scheduleAmount"
          class="border border-gray-200 rounded-md px-3 py-1.5 text-sm w-24 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1">
        <label class="text-sm text-gray-600">Initial stock</label>
        <input
          type="number"
          step="any"
          min="0"
          v-model.number="initialStock"
          class="border border-gray-200 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm text-gray-600">Warning threshold (days)</label>
        <input
          type="number"
          min="0"
          v-model.number="warningThreshold"
          class="border border-gray-200 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
    </div>

    <button
      type="submit"
      :disabled="isLoading || !isValid"
      class="bg-emerald-600 text-white text-sm rounded-md px-4 py-2 hover:bg-emerald-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed mt-2"
    >
      Add Medication
    </button>
  </form>
</template>

<style scoped></style>
