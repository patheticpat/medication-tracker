<script setup lang="ts">
import { apiUrl } from '@/api/base'
import { deletePasskey, getPasskeys, registerPasskey } from '@/api/passkey'
import { KeyRound } from 'lucide-vue-next'
import { onMounted, ref } from 'vue'
import { useQuery, useQueryCache, useMutation } from '@pinia/colada'
import { Trash2 } from 'lucide-vue-next'
import { changePassword } from '@/api/auth'

declare const __GIT_SHA__: string

const backendSHA = ref<string>('unknown')
const frontendSHA = __GIT_SHA__

const queryCache = useQueryCache()

const {
  data: passkeys,
  isLoading,
  error,
} = useQuery({ key: () => ['PASSKEYS'], query: () => getPasskeys() })

const { mutateAsync: deletePasskeyAsync } = useMutation({
  mutation: (id: string) => deletePasskey(id),
  onSettled: () => queryCache.invalidateQueries({ key: ['PASSKEYS'] }),
})

const handleAddPasskey = async () => {
  try {
    await registerPasskey()
    queryCache.invalidateQueries({ key: ['PASSKEYS'] })
  } catch (e) {
    console.error(e)
  }
}

const currentPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const passwordError = ref('')
const passwordSuccess = ref(false)

const { mutateAsync: changePasswordAsync } = useMutation({
  mutation: (payload: { current_password: string; new_password: string }) =>
    changePassword(payload),
})

const handleChangePassword = async () => {
  passwordError.value = ''
  if (newPassword.value !== confirmPassword.value) {
    passwordError.value = 'Passwords do not match'
    return
  }
  if (!currentPassword.value || !newPassword.value) {
    passwordError.value = 'Please fill in all fields'
    return
  }
  try {
    await changePasswordAsync({
      current_password: currentPassword.value,
      new_password: newPassword.value,
    })
    passwordSuccess.value = true
    setTimeout(() => (passwordSuccess.value = false), 3000)
  } catch {
    passwordError.value = 'Current password is incorrect'
  } finally {
    currentPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
  }
}

onMounted(async () => {
  try {
    const r = await fetch(apiUrl('/health'))
    const data = await r.json()
    backendSHA.value = data.version ?? 'unknown'
  } catch {
    backendSHA.value = 'unknown'
  }
})

const formatDate = (ts: number) =>
  new Date(ts * 1000).toLocaleDateString('de-DE', { dateStyle: 'medium' })

const short = (sha: string | null) => sha?.slice(0, 7) ?? 'unknown'
const shortId = (id: string) => id.slice(0, 8) + '…'
</script>

<template>
  <h1 class="text-2xl font-bold text-gray-900 mb-6">Settings</h1>

  <div class="bg-white rounded-lg border border-gray-200 divide-y divide-gray-100">
    <div class="px-5 py-4">
      <h2 class="font-medium text-gray-900 mb-1">Passkeys</h2>
      <p class="text-sm text-gray-500 mb-4">Sign in without a password</p>
      <button
        @click="handleAddPasskey"
        class="flex items-center gap-2 bg-blue-600 text-white text-sm rounded-md px-4 py-2 hover:bg-blue-700 transition-colors"
      >
        <KeyRound class="w-4 h-4" />
        Add new passkey
      </button>

      <ul v-if="!isLoading && !error" class="divide-y divide-gray-100">
        <li
          v-for="passkey in passkeys"
          :key="passkey.credential_id"
          class="flex items-center justify-between py-3"
        >
          <div>
            <p class="text-sm font-medium text-gray-900">
              Added on {{ formatDate(passkey.added_at) }}
              <span class="ml-2 text-xs text-gray-300 font-mono">{{
                shortId(passkey.credential_id)
              }}</span>
            </p>
            <p class="text-xs text-gray-400">
              {{
                passkey.last_used_at
                  ? `Last used: ${formatDate(passkey.last_used_at)}`
                  : 'Never used'
              }}
            </p>
          </div>
          <button
            @click="deletePasskeyAsync(passkey.credential_id)"
            class="text-gray-400 hover:text-red-500 transition-colors cursor-pointer"
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </li>
      </ul>
      <p v-else-if="isLoading" class="text-sm text-gray-400">Loading...</p>
    </div>

    <div class="px-5 py-4">
      <h2 class="font-medium text-gray-900 mb-1">Change Password</h2>
      <div class="divide-y divide-gray-100">
        <div class="py-3">
          <label class="text-xs text-gray-500">Current password</label>
          <input
            v-model="currentPassword"
            type="password"
            class="mt-1 block w-full text-sm border border-gray-200 rounded px-3 py-2"
          />
        </div>
        <div class="py-3">
          <label class="text-xs text-gray-500">New password</label>
          <input
            v-model="newPassword"
            type="password"
            class="mt-1 block w-full text-sm border border-gray-200 rounded px-3 py-2"
          />
        </div>
        <div class="py-3">
          <label class="text-xs text-gray-500">Confirm new password</label>
          <input
            v-model="confirmPassword"
            type="password"
            class="mt-1 block w-full text-sm border border-gray-200 rounded px-3 py-2"
          />
        </div>
      </div>
      <p v-if="passwordError" class="text-xs text-red-500 mt-2">{{ passwordError }}</p>
      <p v-if="passwordSuccess" class="text-xs text-green-500 mt-2">
        Password updated successfully
      </p>
      <button
        @click="handleChangePassword"
        :disabled="newPassword !== confirmPassword || !currentPassword || !newPassword"
        class="mt-3 text-sm text-white bg-gray-800 hover:bg-gray-700 px-4 py-2 rounded cursor-pointer transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        Update password
      </button>
    </div>

    <div class="px-5 py-4">
      <h2 class="font-medium text-gray-900 mb-3">Version</h2>
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
