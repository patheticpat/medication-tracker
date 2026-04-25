<script setup lang="ts">
import { register } from '@/api/auth'
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const username = ref('')
const password = ref('')
const password_confirmation = ref('')
const error = ref('')
const isLoading = ref(false)
const isValid = computed(
  () =>
    username.value.length > 0 &&
    password.value.length > 0 &&
    password.value == password_confirmation.value,
)

const handleSubmit = async () => {
  error.value = ''
  isLoading.value = true
  try {
    await register({ username: username.value, password: password.value })
    router.replace({ name: 'dashboard' })
  } catch {
    error.value = 'Something went wrong'
  } finally {
    isLoading.value = false
    password.value = ''
    password_confirmation.value = ''
  }
}
</script>

<template>
  <div class="flex items-center justify-center bg-gray-50">
    <div class="bg-white rounded-xl border border-gray-200 p-8 w-full max-w-sm">
      <h2 class="text-xl font-semibold text-gray-900 mb-6">Register</h2>

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
        <div class="flex flex-col gap-1">
          <label class="text-sm text-gray-600">Password (again)</label>
          <input
            type="password"
            v-model.trim="password_confirmation"
            class="border border-gray-200 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div v-if="error" class="text-sm text-red-500">{{ error }}</div>
        <button
          type="submit"
          :disabled="isLoading || !isValid"
          class="bg-blue-600 text-white text-sm rounded-md px-4 py-2 hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Register
        </button>
      </form>
    </div>
  </div>
</template>

<style scoped></style>
