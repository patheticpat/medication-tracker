<script setup lang="ts">
import { useMidnightRefresh } from './composables/useMidnightRefresh'
import { useRoute, useRouter } from 'vue-router'
import { useAuth0 } from '@auth0/auth0-vue'
import ToastContainer from './components/ToastContainer.vue'
import LoginButton from './components/buttons/LoginButton.vue'
import LogoutButton from './components/buttons/LogoutButton.vue'
import { Settings2 } from 'lucide-vue-next'

const { isAuthenticated, isLoading } = useAuth0()

useMidnightRefresh()
const route = useRoute()
const router = useRouter()

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
          💊<span class="font-semibold text-gray-800 ml-3">{{ $t('strings.app') }}</span>
        </button>
        <div v-if="!isLoading" class="flex items-center gap-4">
          <RouterLink
            :to="{ name: 'settings' }"
            class="text-sm text-gray-500 hover:text-gray-700 transition-colors"
            v-if="isAuthenticated"
          >
            <Settings2 />
          </RouterLink>
          <LoginButton v-if="!isAuthenticated" />
          <LogoutButton v-if="isAuthenticated" />
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
