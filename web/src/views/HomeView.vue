<script setup lang="ts">
import { useAuth0 } from '@auth0/auth0-vue'
import { useRouter } from 'vue-router'

const { isAuthenticated, loginWithRedirect } = useAuth0()
const router = useRouter()

const handleGetStarted = () => {
  if (isAuthenticated.value) {
    router.push({ name: 'dashboard' })
  } else {
    loginWithRedirect({ appState: { target: '/medications' } })
  }
}
</script>

<template>
  <div class="flex flex-col items-center text-center py-12 gap-8">
    <div>
      <div class="text-5xl mb-4">💊</div>
      <h1 class="text-3xl font-bold text-gray-900 mb-2">Medication Tracker</h1>
      <p class="text-gray-500 max-w-sm">
        Stay on top of your medications. Get notified before you run out and never miss a refill.
      </p>
    </div>

    <button
      @click="handleGetStarted"
      class="bg-amber-400 hover:bg-amber-500 text-white font-medium px-8 py-3 rounded-full transition-colors shadow-sm"
    >
      {{ isAuthenticated ? 'Go to Dashboard' : 'Get Started' }}
    </button>
  </div>
</template>
