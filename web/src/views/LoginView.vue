<script setup lang="ts">
import { login } from '@/api/auth'
import { loginWithPasskey } from '@/api/passkey'
import { ref, computed } from 'vue'
import { useRouter, RouterLink } from 'vue-router'
import { KeyRound } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import { useToast } from '@/composables/useToast'

const { addToast } = useToast()
const router = useRouter()
const store = useAuthStore()
const username = ref('')
const password = ref('')
const isLoading = ref(false)
const isValid = computed(() => username.value.length > 0 && password.value.length > 0)

const handleSubmit = async () => {
  isLoading.value = true
  try {
    await login({ username: username.value, password: password.value })
    router.replace({ name: 'dashboard' })
  } catch {
    addToast('Invalid username or password', 'error')
  } finally {
    isLoading.value = false
  }
}

const handlePasskeyLogin = async () => {
  isLoading.value = true
  try {
    const response = await loginWithPasskey(username.value)
    store.login(response.token)
    router.replace({ name: 'dashboard' })
  } catch {
    addToast('Passkey login failed', 'error')
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="flex items-center justify-center bg-gray-50">
    <div class="bg-white rounded-xl border border-gray-200 p-8 w-full max-w-sm">
      <h2 class="text-xl font-semibold text-gray-900 mb-6">Login</h2>

      <form @submit.prevent="handleSubmit" class="flex flex-col gap-4">
        <div class="flex flex-col gap-1">
          <label class="text-sm text-gray-600">Username</label>
          <input
            v-model.trim="username"
            class="border border-gray-200 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-sm text-gray-600">Password</label>
          <input
            type="password"
            v-model.trim="password"
            class="border border-gray-200 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <button
          type="submit"
          :disabled="isLoading || !isValid"
          class="bg-blue-600 text-white text-sm rounded-md px-4 py-2 hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Login
        </button>
        <div class="relative my-4">
          <div class="absolute inset-0 flex items-center">
            <div class="w-full border-t border-gray-200"></div>
          </div>
          <div class="relative flex justify-center text-sm">
            <span class="px-2 bg-white text-gray-400">or</span>
          </div>
        </div>

        <button
          @click="handlePasskeyLogin"
          :disabled="isLoading || username.length == 0"
          class="w-full flex items-center justify-center gap-2 border border-gray-200 rounded-md px-4 py-2 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
        >
          <KeyRound class="w-4 h-4" />
          Sign in with passkey
        </button>
      </form>

      <p class="text-sm text-gray-500 mt-4 text-center">
        No account yet?
        <RouterLink :to="{ name: 'register' }" class="text-blue-600 hover:text-blue-700">
          Register
        </RouterLink>
      </p>
    </div>
  </div>
</template>
<style scoped></style>
