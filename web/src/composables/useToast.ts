import type { Toast, ToastKind } from '@/types/toast'
import { ref, readonly } from 'vue'

const toasts = ref<Toast[]>([])

const addToast = (message: string, kind: ToastKind, duration?: number) => {
  const toast = { id: crypto.randomUUID(), message, kind, duration: duration ?? 4000 }
  toasts.value = [...toasts.value, toast]
  setTimeout(() => removeToast(toast.id), toast.duration)
}

const removeToast = (id: string) => {
  toasts.value = toasts.value.filter((t) => t.id !== id)
}

export function useToast() {
  return {
    addToast,
    removeToast,
    toasts: readonly(toasts),
  }
}
