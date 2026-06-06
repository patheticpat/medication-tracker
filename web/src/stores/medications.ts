import { useMutation, useQuery, useQueryCache } from '@pinia/colada'
import type { CreateLogEntry, MedicationWithStats, UpdateMedication } from '@/types/medication'
import { useToast } from '@/composables/useToast'
import { useApi } from '@/composables/useApi'
import { useI18n } from 'vue-i18n'

export const MEDICATION_KEYS = {
  root: ['medications'] as const,
  byId: (id: string) => [...MEDICATION_KEYS.root, id] as const,
}

export function useMedications() {
  const { getMedications } = useApi()

  return useQuery({
    key: MEDICATION_KEYS.root,
    query: getMedications,
  })
}

export function useCreateLogEntry(id: string) {
  const { createLogEntry } = useApi()
  const cache = useQueryCache()
  return useMutation({
    mutation: (log: CreateLogEntry) => createLogEntry(id, log),
    onSettled: () => {
      cache.invalidateQueries({ key: MEDICATION_KEYS.root })
    },
  })
}

export function useUpdateMedication(id: string) {
  const { updateMedication } = useApi()
  const { addToast } = useToast()
  const { t } = useI18n()
  const cache = useQueryCache()

  return useMutation({
    mutation: (medication: UpdateMedication) => updateMedication(id, medication),
    onError: () => addToast(t('medication.updateFailed'), 'error'),
    onSuccess: () => {
      addToast(t('medication.updated'), 'success')
      cache.invalidateQueries({ key: MEDICATION_KEYS.root })
    },
  })
}

export function useUpdateSnooze() {
  const { updateSnooze } = useApi()
  const cache = useQueryCache()
  const { addToast } = useToast()
  const { t } = useI18n()

  return useMutation({
    mutation: ({ id, snoozed }: { id: string; snoozed: boolean }) => updateSnooze(id, snoozed),
    onMutate: ({ id, snoozed }: { id: string; snoozed: boolean }) => {
      const oldMedicationList = cache.getQueryData<MedicationWithStats[]>(MEDICATION_KEYS.root)
      const oldMedication = cache.getQueryData<MedicationWithStats>(MEDICATION_KEYS.byId(id))

      cache.setQueryData(MEDICATION_KEYS.root, (medications?: MedicationWithStats[]) =>
        medications?.map((m) => (m.id === id ? { ...m, snoozed } : m)),
      )
      cache.setQueryData(MEDICATION_KEYS.byId(id), (medication?: MedicationWithStats) =>
        medication ? { ...medication, snoozed } : medication,
      )

      return { oldMedicationList, oldMedication }
    },
    onError: (_, { id }, { oldMedicationList, oldMedication }) => {
      if (oldMedicationList) cache.setQueryData(MEDICATION_KEYS.root, oldMedicationList)
      if (oldMedication) cache.setQueryData(MEDICATION_KEYS.byId(id), oldMedication)
      addToast(t('medication.snoozeFailed'), 'error')
    },
    onSuccess: (updatedMedication, { id }) => {
      cache.setQueryData(MEDICATION_KEYS.root, (medications?: MedicationWithStats[]) =>
        medications?.map((m) => (m.id === id ? updatedMedication : m)),
      )
      cache.setQueryData(MEDICATION_KEYS.byId(id), () => updatedMedication)
    },
  })
}
