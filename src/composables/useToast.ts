import { inject, ref, type InjectionKey, type Ref } from 'vue'

export interface Toast {
  id: string
  message: string
  type: 'success' | 'error' | 'info'
  duration: number
}

export interface ToastContext {
  toasts: Ref<Toast[]>
  success: (message: string) => void
  error: (message: string) => void
  info: (message: string) => void
  dismiss: (id: string) => void
}

export const ToastKey: InjectionKey<ToastContext> = Symbol('toast')

export function useToastProvider(): ToastContext {
  const toasts = ref<Toast[]>([])

  function add(message: string, type: Toast['type'], duration = 3000) {
    const id = crypto.randomUUID()
    toasts.value.push({ id, message, type, duration })
    setTimeout(() => dismiss(id), duration)
  }

  function dismiss(id: string) {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  return {
    toasts,
    success: (msg) => add(msg, 'success'),
    error: (msg) => add(msg, 'error', 5000),
    info: (msg) => add(msg, 'info'),
    dismiss,
  }
}

export function useToast(): ToastContext {
  const ctx = inject(ToastKey)
  if (!ctx) throw new Error('useToast() called without ToastProvider')
  return ctx
}
