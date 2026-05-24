<script setup lang="ts">
import { BellOff } from 'lucide-vue-next'
defineEmits(['snooze'])
</script>

<template>
  <button
    @click="$emit('snooze')"
    class="flex items-center gap-1.5 text-sm bg-amber-100 hover:bg-amber-200 text-amber-700 px-3 py-1.5 rounded-md transition-colors"
  >
    <BellOff class="w-4 h-4" />
    Snooze all
  </button>
</template>
