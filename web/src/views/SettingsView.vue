<script setup lang="ts">
import { registerPasskey } from '@/api/passkey'
import { KeyRound } from 'lucide-vue-next'
const handleAddPasskey = async () => {
  try {
    await registerPasskey()
  } catch (e) {
    console.error(e)
  }
}
</script>

<template>
  <h1 class="text-2xl font-bold text-gray-900 mb-6">Settings</h1>

  <div class="bg-white rounded-lg border border-gray-200 divide-y divide-gray-100">
    <div class="px-5 py-4">
      <h2 class="font-medium text-gray-900 mb-1">Passkeys</h2>
      <p class="text-sm text-gray-500 mb-4">Melde dich ohne Passwort an</p>
      <button
        @click="handleAddPasskey"
        class="flex items-center gap-2 bg-blue-600 text-white text-sm rounded-md px-4 py-2 hover:bg-blue-700 transition-colors"
      >
        <KeyRound class="w-4 h-4" />
        Neuen Passkey hinzufügen
      </button>
    </div>
  </div>
</template>
<style scoped></style>
