import {
  startRegistration,
  startAuthentication,
} from '@simplewebauthn/browser'
import { apiUrl, authHeaders, handleResponse } from './base'
import type { PublicKeyCredentialCreationOptionsJSON, PublicKeyCredentialRequestOptionsJSON } from '@simplewebauthn/browser'
import type { AuthResponse } from '@/types/auth'

export async function registerPasskey() {
  // 1. Begin vom Backend holen
  const r = await fetch(apiUrl('/auth/passkey/register/begin'), {
    method: 'POST',
    headers: authHeaders(),
  })
  const options = await handleResponse<{publicKey: PublicKeyCredentialCreationOptionsJSON}>(r)

  // 2. Browser zeigt Authenticator-Dialog
  const credential = await startRegistration({ optionsJSON: options.publicKey })

  // 3. Ergebnis ans Backend schicken
  const r2 = await fetch(apiUrl('/auth/passkey/register/complete'), {
    method: 'POST',
    headers: authHeaders(true),
    body: JSON.stringify(credential),
  })
  await handleResponse<void>(r2)
}

export async function loginWithPasskey(username: string) {
  // 1. Begin vom Backend holen
  const r = await fetch(apiUrl('/auth/passkey/login/begin'), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username }),
  })
  const options = await handleResponse<{publicKey: PublicKeyCredentialRequestOptionsJSON}>(r)

  // 2. Browser zeigt Authenticator-Dialog
  const credential = await startAuthentication({ optionsJSON: options.publicKey })

  // 3. Ergebnis ans Backend schicken
  const r2 = await fetch(apiUrl('/auth/passkey/login/complete'), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(credential),
  })
  return handleResponse<AuthResponse>(r2)
}
