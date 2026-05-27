import { useMutation, useQuery, useQueryCache } from '@pinia/colada'
import { createLogEntry, getMedications, updateSnooze } from '@/api/medications'
import type { CreateLogEntry, MedicationWithStats } from '@/types/medication'

export const MEDICATION_KEYS = {
  root: ['medications'] as const,
  byId: (id: string) => [...MEDICATION_KEYS.root, id] as const,
}

export function useMedications() {
  return useQuery({
    key: MEDICATION_KEYS.root,
    query: getMedications,
  })
}

export function useCreateLogEntry(id: string) {
  const cache = useQueryCache()
  return useMutation({
    mutation: (log: CreateLogEntry) => createLogEntry(id, log),
    onSettled: () => {
      cache.invalidateQueries({ key: MEDICATION_KEYS.root })
    },
  })
}

export function useUpdateSnooze() {
  const cache = useQueryCache()
  return useMutation({
    mutation: ({id, snoozed}: {id: string, snoozed: boolean}) => updateSnooze(id, snoozed),
    onMutate: ({id, snoozed}: {id: string, snoozed: boolean}) => {
      cache.setQueryData(MEDICATION_KEYS.root, (medications?: MedicationWithStats[]) => medications?.map((m) => (m.id === id ? {...m, snoozed} : m)))
      cache.setQueryData(MEDICATION_KEYS.byId(id), (medication?: MedicationWithStats) => medication ? {...medication, snoozed} : medication)
    },
    onSuccess: (updatedMedication, {id}) => {
      cache.setQueryData(MEDICATION_KEYS.root, (medications?: MedicationWithStats[]) => medications?.map((m) => (m.id === id ? updatedMedication : m)))
      cache.setQueryData(MEDICATION_KEYS.byId(id), () => updatedMedication)
    },
  })
}
