<script setup lang="ts">
import { useMidnightRefresh } from './composables/useMidnightRefresh'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from './stores/auth'
import { storeToRefs } from 'pinia'
import ToastContainer from './components/ToastContainer.vue'

const authStore = useAuthStore()
const { isLoggedIn } = storeToRefs(authStore)
const { logout } = authStore

useMidnightRefresh()
const route = useRoute()
const router = useRouter()

function handleLogout() {
  logout()
  router.replace({ name: 'login' })
}

function handleLogoClick() {
  if (route.name !== 'dashboard') {
    router.replace({ name: 'dashboard' })
  }
}
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <nav class="sticky top-0 z-10 bg-white border-b border-gray-200 px-6 py-4 shadow-sm">
      <div class="max-w-2xl mx-auto flex items-center justify-between">
        <button
          @click="handleLogoClick"
          class="font-bold text-xl text-gray-900 hover:text-gray-700 transition-colors cursor-pointer"
        >
          <span class="font-semibold text-gray-800">💊 Medication Tracker</span>
        </button>
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
  <ToastContainer />
</template>

<style scoped></style>
