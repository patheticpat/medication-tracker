// src/types/toast.ts
export type ToastKind = 'success' | 'error' | 'info'

export interface Toast {
  id: string
  message: string
  kind: ToastKind
  duration: number // ms
  action?: { label: string; onClick: () => void }
}
