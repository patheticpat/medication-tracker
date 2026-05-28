import { ref, computed } from 'vue'
import { getVapidPublicKey, subscribePush, unsubscribePush } from '@/api/push'

function urlBase64ToUint8Array(base64String: string): Uint8Array<ArrayBuffer> {
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4)
  const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/')
  const rawData = atob(base64)
  return new Uint8Array([...rawData].map((c) => c.charCodeAt(0)))
}

export function usePush() {
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const isSupported = computed(() => 'serviceWorker' in navigator && 'PushManager' in window)

  const permission = ref<NotificationPermission>(
    isSupported.value ? Notification.permission : 'denied',
  )

  async function enable() {
    if (!isSupported.value) return
    isLoading.value = true
    error.value = null
    try {
      const result = await Notification.requestPermission()
      permission.value = result
      if (result !== 'granted') return

      const vapid_public_key = await getVapidPublicKey()
      const registration = await navigator.serviceWorker.ready
      const subscription = await registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: urlBase64ToUint8Array(vapid_public_key),
      })
      await subscribePush(subscription.toJSON())
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unbekannter Fehler'
    } finally {
      isLoading.value = false
    }
  }

  async function disable() {
    if (!isSupported.value) return
    isLoading.value = true
    error.value = null
    try {
      const registration = await navigator.serviceWorker.ready
      const subscription = await registration.pushManager.getSubscription()
      if (subscription) {
        await unsubscribePush(subscription.endpoint)
        await subscription.unsubscribe()
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unbekannter Fehler'
    } finally {
      isLoading.value = false
    }
  }

  return { isSupported, isLoading, error, permission, enable, disable }
}
