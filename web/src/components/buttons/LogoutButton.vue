<script setup lang="ts">
import { useAuth0 } from '@auth0/auth0-vue'
import { useQueryCache } from '@pinia/colada'
import { LogOut } from 'lucide-vue-next'

const { logout } = useAuth0()

const handleLogout = () => {
  logout({
    logoutParams: {
      returnTo: window.location.origin,
    },
  })

  const queryCache = useQueryCache()
  queryCache.invalidateQueries()
}
</script>

<template>
  <button
    class="text-sm text-gray-500 hover:text-gray-700 transition-colors cursor-pointer"
    @click="handleLogout"
  >
    <LogOut />
  </button>
</template>
