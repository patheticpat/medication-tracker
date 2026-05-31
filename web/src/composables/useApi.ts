// src/composables/useApi.ts
import { useAuth0 } from '@auth0/auth0-vue'
import { ApiError } from '@/api/base'
import type {
  CreateLogEntry,
  CreateMedication,
  MedicationWithStats,
  UpdateMedication,
} from '@/types/medication'
import type { NotificationSettings } from '@/types/push'

const BASE_URL = import.meta.env.VITE_API_BASE_URL

export function useApi() {
  const { getAccessTokenSilently, logout } = useAuth0()

  const handleResponse = async <T>(r: Response): Promise<T> => {
    if (r.status === 401) {
      logout({ logoutParams: { returnTo: window.location.origin } })
      throw new ApiError(401, 'Unauthorized')
    }
    if (!r.ok) {
      const body = await r.text().catch(() => '')
      throw new ApiError(r.status, body || `API Error: ${r.status}`)
    }
    if (r.status == 204) return undefined as T
    return r.json()
  }

  const fetchWithToken = async (input: string, init?: RequestInit) => {
    const token = await getAccessTokenSilently()
    return fetch(`${BASE_URL}${input}`, {
      ...init,
      headers: {
        ...init?.headers,
        Authorization: `Bearer ${token}`,
        'X-Timezone': Intl.DateTimeFormat().resolvedOptions().timeZone,
      },
    })
  }

  const getVersion = async (): Promise<{ status: string; version: string }> => {
    const r = await fetchWithToken('/health')
    return handleResponse(r)
  }

  const getMedications = async (): Promise<MedicationWithStats[]> => {
    const r = await fetchWithToken('/medications')
    return handleResponse(r)
  }

  const getMedicationDetails = async (id: string): Promise<MedicationWithStats> => {
    const r = await fetchWithToken(`/medications/${id}`)
    return handleResponse(r)
  }

  const createMedication = async (medication: CreateMedication): Promise<MedicationWithStats> => {
    const r = await fetchWithToken('/medications', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(medication),
    })
    return handleResponse(r)
  }

  const updateSnooze = async (id: string, snoozed: boolean): Promise<MedicationWithStats> => {
    const r = await fetchWithToken(`/medications/${id}/snooze`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ snoozed }),
    })
    return handleResponse(r)
  }

  const createLogEntry = async (
    id: string,
    logEntry: CreateLogEntry,
  ): Promise<MedicationWithStats> => {
    const r = await fetchWithToken(`/medications/${id}/log`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(logEntry),
    })
    return handleResponse(r)
  }

  const updateMedication = async (
    id: string,
    medication: UpdateMedication,
  ): Promise<MedicationWithStats> => {
    const r = await fetchWithToken(`/medications/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(medication),
    })
    return handleResponse(r)
  }

  const deleteMedication = async (id: string): Promise<void> => {
    const r = await fetchWithToken(`/medications/${id}`, {
      method: 'DELETE',
    })

    await handleResponse<void>(r)
  }

  const getVapidPublicKey = async (): Promise<string> => {
    const r = await fetchWithToken('/push/vapid-public-key')
    const data = await handleResponse<{ publicKey: string }>(r)
    return data.publicKey
  }

  const subscribePush = async (subscription: PushSubscriptionJSON): Promise<void> => {
    const r = await fetchWithToken('/push/subscribe', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        endpoint: subscription.endpoint,
        p256dh: subscription.keys?.p256dh,
        auth: subscription.keys?.auth,
      }),
    })
    return handleResponse<void>(r)
  }

  const unsubscribePush = async (endpoint: string): Promise<void> => {
    const r = await fetchWithToken('/push/subscribe', {
      method: 'DELETE',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ endpoint }),
    })
    return handleResponse<void>(r)
  }

  const updateNotificationSettings = async (settings: NotificationSettings): Promise<void> => {
    const r = await fetchWithToken('/push/settings', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(settings),
    })
    return handleResponse<void>(r)
  }

  const getNotificationSettings = async (): Promise<NotificationSettings> => {
    const r = await fetchWithToken('/push/settings')
    return await handleResponse(r)
  }

  const testPush = async (endpoint: string): Promise<void> => {
    const r = await fetchWithToken('/push/test', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ endpoint }),
    })
    return handleResponse<void>(r)
  }

  return {
    getVersion,
    getMedications,
    getMedicationDetails,
    createMedication,
    deleteMedication,
    createLogEntry,
    updateSnooze,
    updateMedication,
    testPush,
    getNotificationSettings,
    updateNotificationSettings,
    getVapidPublicKey,
    subscribePush,
    unsubscribePush,
  }
}
