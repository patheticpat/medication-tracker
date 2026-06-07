import type { Toast, ToastKind } from '@/types/toast'
import { ref, readonly } from 'vue'

const toasts = ref<Toast[]>([])

const addToast = (
  message: string,
  kind: ToastKind,
  duration?: number,
  action?: Toast['action'],
) => {
  const toast = { id: crypto.randomUUID(), message, kind, duration: duration ?? 4000, action }
  toasts.value = [...toasts.value, toast]
  if (toast.duration > 0) {
    setTimeout(() => removeToast(toast.id), toast.duration)
  }
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
