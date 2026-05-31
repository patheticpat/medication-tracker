<script setup lang="ts">
import { useAuth0 } from '@auth0/auth0-vue'

const { loginWithRedirect } = useAuth0()

const handleLogin = () => {
  loginWithRedirect({
    appState: {
      target: '/medications',
    },
  })
}
</script>

<template>
  <button
    class="text-sm text-gray-500 hover:text-gray-700 transition-colors cursor-pointer"
    @click="handleLogin"
  >
    Log In
  </button>
</template>
