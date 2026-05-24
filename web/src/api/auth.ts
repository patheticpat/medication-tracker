import type { AuthRequest, AuthResponse } from '@/types/auth'
import { apiUrl, authHeaders, handleResponse } from './base'
import { useAuthStore } from '@/stores/auth'

const store = useAuthStore()

export const login = async (request: AuthRequest): Promise<AuthResponse> => {
  const url = apiUrl('/auth/login')
  const r = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(request),
  })
  const response = await handleResponse<AuthResponse>(r)
  store.login(response.token)
  return response
}

export const register = async (request: AuthRequest): Promise<AuthResponse> => {
  const url = apiUrl('/auth/register')
  const r = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(request),
  })
  const response = await handleResponse<AuthResponse>(r)
  store.login(response.token)
  return response
}

export const changePassword = async (payload: {
  current_password: string
  new_password: string
}): Promise<void> => {
  // PUT /auth/password mit authHeaders(true) und JSON body
  const r = await fetch(apiUrl('/auth/password'), {
    method: 'PUT',
    headers: authHeaders(true),
    body: JSON.stringify(payload),
  })

  return await handleResponse(r)
}
