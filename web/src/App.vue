<script setup lang="ts">
import { useMidnightRefresh } from './composables/useMidnightRefresh'
import { useRouter } from 'vue-router'
import { useAuthStore } from './stores/auth'
import { storeToRefs } from 'pinia'

const authStore = useAuthStore()
const { isLoggedIn } = storeToRefs(authStore)
const { logout } = authStore

useMidnightRefresh()
const router = useRouter()

function handleLogout() {
  logout()
  router.replace({ name: 'login' })
}
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <nav class="sticky top-0 z-10 bg-white border-b border-gray-200 px-6 py-4 shadow-sm">
      <div class="max-w-2xl mx-auto flex items-center justify-between">
        <RouterLink
          :to="{ name: 'dashboard' }"
          class="font-bold text-xl text-gray-900 hover:text-gray-700 transition-colors"
        >
          <span class="font-semibold text-gray-800">💊 Medication Tracker</span>
        </RouterLink>
        <div class="flex items-center gap-4">
          <RouterLink
            :to="{ name: 'settings' }"
            class="text-sm text-gray-500 hover:text-gray-700 transition-colors"
            v-if="isLoggedIn"
          >
            Settings
          </RouterLink>
          <button
            v-if="isLoggedIn"
            @click="handleLogout"
            class="text-sm text-gray-500 hover:text-gray-700 transition-colors"
          >
            Logout
          </button>
        </div>
      </div>
    </nav>
    <main class="max-w-2xl mx-auto px-6 py-8">
      <RouterView />
    </main>
  </div>
</template>

<style scoped></style>
