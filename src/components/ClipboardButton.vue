<script setup lang="ts">
import { ClipboardCopy, ClipboardCheck } from 'lucide-vue-next'
import { ref, onUnmounted } from 'vue'

const props = defineProps<{ text: string }>()
const copied = ref(false)
let timer: ReturnType<typeof setTimeout> | null = null

const copyToClipboard = async () => {
  await navigator.clipboard.writeText(props.text)
  copied.value = true
  timer = setTimeout(() => (copied.value = false), 2000)
}

onUnmounted(() => {
  if (timer !== null) {
    clearTimeout(timer)
  }
})
</script>

<template>
  <button
    @click="copyToClipboard"
    :disabled="copied"
    class="flex items-center gap-1.5 text-sm bg-red-100 hover:bg-red-200 text-red-700 px-3 py-1.5 rounded-md transition-colors disabled:opacity-50"
  >
    <ClipboardCheck v-if="copied" class="w-4 h-4" />
    <ClipboardCopy v-else class="w-4 h-4" />
    {{ copied ? 'Copied!' : 'Copy' }}
  </button>
</template>

<style scoped></style>
