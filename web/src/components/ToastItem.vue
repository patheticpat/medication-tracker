<script setup lang="ts">
import type { Toast } from '@/types/toast'
import { useToast } from '@/composables/useToast'
import { CheckCircle, XCircle, Info, X } from 'lucide-vue-next'

const { removeToast } = useToast()
const props = defineProps<{ toast: Toast }>()
</script>

<template>
  <div
    class="flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg border min-w-72 max-w-sm"
    :class="{
      'bg-emerald-50 border-emerald-200 text-emerald-800': props.toast.kind === 'success',
      'bg-red-50 border-red-200 text-red-800': props.toast.kind === 'error',
      'bg-blue-50 border-blue-200 text-blue-800': props.toast.kind === 'info',
    }"
  >
    <CheckCircle v-if="props.toast.kind === 'success'" class="w-5 h-5 shrink-0 mt-0.5" />
    <XCircle v-else-if="props.toast.kind === 'error'" class="w-5 h-5 shrink-0 mt-0.5" />
    <Info v-else class="w-5 h-5 shrink-0 mt-0.5" />

    <span class="flex-1 text-sm font-medium">{{ props.toast.message }}</span>

    <button
      @click="removeToast(props.toast.id)"
      class="shrink-0 opacity-60 hover:opacity-100 transition-opacity cursor-pointer"
    >
      <X class="w-4 h-4" />
    </button>
  </div>
</template>
