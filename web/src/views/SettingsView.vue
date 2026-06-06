<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useQuery } from '@pinia/colada'
import { useToast } from '@/composables/useToast'
import { Bell, BellOff } from 'lucide-vue-next'
import { usePush } from '@/composables/usePush'
import { useApi } from '@/composables/useApi'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const { updateNotificationSettings, getNotificationSettings, getVersion } = useApi()

declare const __GIT_SHA__: string

const backendSHA = ref<string>('unknown')
const frontendSHA = __GIT_SHA__

const { addToast } = useToast()

const notificationHour = ref(8)
const notificationDays = ref([0, 1, 2, 3, 4, 5, 6])

const { data: notificationSettings } = useQuery({
  key: () => ['NOTIFICATION_SETTINGS'],
  query: () => getNotificationSettings(),
})

watch(
  notificationSettings,
  (settings) => {
    if (settings) {
      notificationHour.value = settings.notificationHour
      notificationDays.value =
        settings.notificationDays.length == 0
          ? []
          : settings.notificationDays.split(',').map(Number)
    }
  },
  { immediate: true },
)

const {
  isSupported,
  isSubscribed,
  isLoading: pushLoading,
  permission,
  enable,
  disable,
  sendTestPush,
  error: pushError,
} = usePush()

watch(pushError, (error) => {
  if (error && error.length > 0) {
    addToast(error, 'error')
  }
})

const DAYS = [
  { label: t('strings.weekdays.short.monday'), value: 1 },
  { label: t('strings.weekdays.short.tuesday'), value: 2 },
  { label: t('strings.weekdays.short.wednesday'), value: 3 },
  { label: t('strings.weekdays.short.thursday'), value: 4 },
  { label: t('strings.weekdays.short.friday'), value: 5 },
  { label: t('strings.weekdays.short.saturday'), value: 6 },
  { label: t('strings.weekdays.short.sunday'), value: 0 },
]

const toggleDay = (value: number) => {
  if (notificationDays.value.includes(value)) {
    notificationDays.value = notificationDays.value.filter((d) => d !== value)
  } else {
    notificationDays.value = [...notificationDays.value, value]
  }
}

const handleSaveSettings = async () => {
  try {
    await updateNotificationSettings({
      notificationHour: notificationHour.value,
      notificationDays: notificationDays.value.sort().join(','),
    })
    addToast(t('settings.saved'), 'success')
  } catch {
    addToast(t('settings.saveFailed'), 'error')
  }
}

onMounted(async () => {
  try {
    const data = await getVersion()
    backendSHA.value = data.version ?? 'unknown'
  } catch {
    backendSHA.value = 'unknown'
  }
})

const short = (sha: string | null) =>
  sha && sha.length > 20 ? sha.slice(0, 7) : t('strings.unknown')
</script>

<template>
  <h1 class="text-2xl font-bold text-gray-900 mb-6">{{ $t('navigation.settings') }}</h1>

  <div class="bg-white rounded-lg border border-gray-200 divide-y divide-gray-100">
    <div class="px-5 py-4">
      <h2 class="font-medium text-gray-900 mb-1">{{ $t('settings.notifications') }}</h2>
      <p v-if="isSupported" class="text-sm text-gray-500 mb-4">
        {{ $t('settings.notificationsExplanation') }}
      </p>

      <div v-if="!isSupported" class="text-sm text-gray-400">
        {{ $t('settings.notificationsUnsupported') }}
      </div>

      <div v-else class="flex flex-col gap-4">
        <div class="flex items-center gap-3">
          <button
            v-if="!isSubscribed"
            @click="enable"
            :disabled="pushLoading || permission === 'denied'"
            class="flex items-center gap-2 bg-blue-600 text-white text-sm rounded-md px-4 py-2 hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Bell class="w-4 h-4" />
            {{ $t('settings.enableNotifications') }}
          </button>
          <button
            v-else
            @click="disable"
            :disabled="pushLoading"
            class="flex items-center gap-2 bg-gray-100 text-gray-700 text-sm rounded-md px-4 py-2 hover:bg-gray-200 transition-colors disabled:opacity-50"
          >
            <BellOff class="w-4 h-4" />
            {{ $t('settings.disableNotifications') }}
          </button>
          <span v-if="permission === 'denied'" class="text-xs text-red-500">
            {{ $t('settings.notificationsBlocked') }}
          </span>
        </div>

        <div v-if="isSubscribed" class="flex flex-col gap-3">
          <div class="flex flex-col gap-1">
            <label class="text-xs text-gray-500">{{ $t('settings.time') }}</label>
            <input
              v-model.number="notificationHour"
              type="number"
              min="0"
              max="23"
              class="w-24 text-sm border border-gray-200 rounded px-3 py-2"
            />
          </div>

          <div class="flex flex-col gap-1">
            <label class="text-xs text-gray-500">{{ $t('strings.daysOfWeek') }}</label>
            <div class="flex gap-1.5">
              <button
                v-for="day in DAYS"
                :key="day.value"
                @click="toggleDay(day.value)"
                :class="[
                  'w-9 h-9 rounded-full text-xs font-medium transition-colors',
                  notificationDays.includes(day.value)
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-100 text-gray-500 hover:bg-gray-200',
                ]"
              >
                {{ day.label }}
              </button>
            </div>
          </div>

          <button
            @click="handleSaveSettings"
            class="self-start text-sm text-white bg-gray-800 hover:bg-gray-700 px-4 py-2 rounded transition-colors"
          >
            {{ $t('settings.save') }}
          </button>
          <button
            @click="sendTestPush"
            class="self-start text-sm text-white bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded transition-colors"
          >
            {{ $t('settings.testNotifications') }}
          </button>
        </div>
      </div>
    </div>

    <div class="px-5 py-4">
      <h2 class="font-medium text-gray-900 mb-3">{{ $t('settings.versions') }}</h2>
      <div class="flex flex-col gap-1.5 font-mono text-sm text-gray-500">
        <div class="flex justify-between">
          <span>Frontend</span>
          <span>{{ short(frontendSHA) }}</span>
        </div>
        <div class="flex justify-between">
          <span>Backend</span>
          <span>{{ short(backendSHA) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
<style scoped></style>
