import type {
  CreateLogEntry,
  CreateMedication,
  UpdateMedication,
  MedicationWithStats,
} from '@/types/medication'
import { apiUrl, authHeaders, handleResponse, today } from './base'

export async function getMedications(): Promise<MedicationWithStats[]> {
  const url = apiUrl('/medications')
  url.searchParams.set('date', today())
  const r = await fetch(url, { headers: authHeaders() })
  return handleResponse(r)
}

export async function createMedication(medication: CreateMedication): Promise<MedicationWithStats> {
  const url = apiUrl('/medications')
  url.searchParams.set('date', today())
  const r = await fetch(url, {
    method: 'POST',
    headers: authHeaders(true),
    body: JSON.stringify(medication),
  })
  return handleResponse(r)
}

export async function getMedicationDetails(id: string): Promise<MedicationWithStats> {
  const url = apiUrl(`/medications/${id}`)
  url.searchParams.set('date', today())
  const r = await fetch(url, { headers: authHeaders() })
  return handleResponse(r)
}

export async function updateMedication(
  id: string,
  medication: UpdateMedication,
): Promise<MedicationWithStats> {
  const url = apiUrl(`/medications/${id}`)
  url.searchParams.set('date', today())
  const r = await fetch(url, {
    method: 'PATCH',
    headers: authHeaders(true),
    body: JSON.stringify(medication),
  })
  return handleResponse(r)
}

export async function updateSnooze(id: string, snoozed: boolean): Promise<MedicationWithStats> {
  const url = apiUrl(`/medications/${id}/snooze`)
  url.searchParams.set('date', today())
  const r = await fetch(url, {
    method: 'PATCH',
    headers: authHeaders(true),
    body: JSON.stringify({snoozed}),
  })
  return handleResponse(r)
}

export async function createLogEntry(
  id: string,
  logEntry: CreateLogEntry,
): Promise<MedicationWithStats> {
  const url = apiUrl(`/medications/${id}/log`)
  const at = today()
  url.searchParams.set('date', at)
  const r = await fetch(url, {
    method: 'POST',
    headers: authHeaders(true),
    body: JSON.stringify({ ...logEntry, date: at }),
  })
  return handleResponse(r)
}

export async function deleteMedication(id: string) {
  const url = apiUrl(`/medications/${id}`)
  const r = await fetch(url, { method: 'DELETE', headers: authHeaders() })
  if (!r.ok) {
    throw new Error(`API Error: ${r.status}`)
  }
}
