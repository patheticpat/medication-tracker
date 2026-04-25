import { useMutation, useQuery, useQueryCache } from '@pinia/colada'
import { createLogEntry, getMedications } from '@/api/medications'
import type { CreateLogEntry } from '@/types/medication'

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
      cache.invalidateQueries({ key: MEDICATION_KEYS.byId(id) })
    },
  })
}
