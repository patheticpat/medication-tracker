import router from '@/router'
import { useAuthStore } from '@/stores/auth'

export const BASE_URL = import.meta.env.VITE_API_BASE_URL

export function today(): string {
  return new Date().toLocaleDateString('en-CA')
}

export const formatAmount = (amount: number) => Number(amount.toFixed(1)).toString()

export function apiUrl(path: string): URL {
  return new URL(`${BASE_URL}${path.replace(/^\//, '')}`, window.location.origin)
}

export async function handleResponse<T>(r: Response): Promise<T> {
  if (r.status === 401) {
    const authStore = useAuthStore()
    authStore.logout()
    router.replace({ name: 'login' })
    throw new Error('Unauthorized')
  }
  if (!r.ok) {
    throw new Error(`API Error: ${r.status}`)
  }
  if (r.status == 204) return undefined as T
  return r.json()
}

export function authHeaders(withContentType = false): Record<string, string> {
  const token = localStorage.getItem('token')
  const headers: Record<string, string> = {}
  if (token) headers['Authorization'] = `Bearer ${token}`
  if (withContentType) headers['Content-Type'] = 'application/json'
  return headers
}
