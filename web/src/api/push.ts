import { apiUrl, authHeaders, handleResponse } from './base'

export async function getVapidPublicKey(): Promise<string> {
  const r = await fetch(apiUrl('/push/vapid-public-key'))
  const data = await handleResponse<{ publicKey: string }>(r)
  return data.publicKey
}
export async function subscribePush(subscription: PushSubscriptionJSON): Promise<void> {
  const r = await fetch(apiUrl('/push/subscribe'), {
    method: 'POST',
    headers: authHeaders(true),
    body: JSON.stringify({
      endpoint: subscription.endpoint,
      p256dh: subscription.keys?.p256dh,
      auth: subscription.keys?.auth,
    }),
  })
  await handleResponse<void>(r)
}

export async function unsubscribePush(endpoint: string): Promise<void> {
  const r = await fetch(apiUrl('/push/subscribe'), {
    method: 'DELETE',
    headers: authHeaders(true),
    body: JSON.stringify({ endpoint }),
  })
  await handleResponse<void>(r)
}

export async function updateNotificationSettings(settings: {
  notificationHour: number
  notificationDays: string
}): Promise<void> {
  const r = await fetch(apiUrl('/push/settings'), {
    method: 'PUT',
    headers: authHeaders(true),
    body: JSON.stringify(settings),
  })
  await handleResponse<void>(r)
}
