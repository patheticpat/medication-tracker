import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useApi } from './useApi'
import { useToast } from './useToast'

function urlBase64ToUint8Array(base64String: string): Uint8Array<ArrayBuffer> {
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4)
  const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/')
  const rawData = atob(base64)
  return new Uint8Array([...rawData].map((c) => c.charCodeAt(0)))
}

export function usePush() {
  const { getVapidPublicKey, subscribePush, testPush, unsubscribePush } = useApi()
  const { addToast } = useToast()
  const { t } = useI18n()
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const isSubscribed = ref(false)

  const isSupported = computed(() => 'serviceWorker' in navigator && 'PushManager' in window)

  const permission = ref<NotificationPermission>(
    isSupported.value ? Notification.permission : 'denied',
  )

  async function checkSubscription() {
    if (!isSupported.value) return
    const registration = await navigator.serviceWorker.ready
    const existing = await registration.pushManager.getSubscription()
    isSubscribed.value = existing !== null
  }

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
      isSubscribed.value = true
      addToast(t('settings.notificationsEnabled'), 'success')
    } catch (e) {
      error.value = e instanceof Error ? e.message : t('strings.unknownError')
      addToast(t('settings.notificationsEnableFailed'), 'error')
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
        isSubscribed.value = false
        addToast(t('settings.notificationsDisabled'), 'success')
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : t('strings.unknownError')
      addToast(t('settings.notificationsDisableFailed'), 'error')
    } finally {
      isLoading.value = false
    }
  }

  async function sendTestPush() {
    if (!isSupported.value) return
    isLoading.value = true
    error.value = null
    try {
      const registration = await navigator.serviceWorker.ready
      const subscription = await registration.pushManager.getSubscription()
      if (!subscription) return
      await testPush(subscription.endpoint)
      addToast(t('settings.testNotificationSent'), 'success')
    } catch (e) {
      error.value = e instanceof Error ? e.message : t('strings.unknownError')
      addToast(t('settings.testNotificationFailed'), 'error')
    } finally {
      isLoading.value = false
    }
  }

  onMounted(checkSubscription)

  return { isSupported, isLoading, error, permission, isSubscribed, enable, disable, sendTestPush }
}
