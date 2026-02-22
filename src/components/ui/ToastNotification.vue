<script setup lang="ts">
import { onMounted } from 'vue'

interface ToastProps {
  message: string
  type?: 'success' | 'error' | 'info'
  duration?: number
}

const props = withDefaults(defineProps<ToastProps>(), {
  type: 'info',
  duration: 3000,
})

const emit = defineEmits<{
  dismiss: []
}>()

onMounted(() => {
  setTimeout(() => emit('dismiss'), props.duration)
})

const iconColourClass: Record<string, string> = {
  success: 'icon-success',
  error: 'icon-error',
  info: 'icon-info',
}
</script>

<template>
  <div
    class="toast flex items-center gap-3 rounded-xl px-4 py-3 shadow-lg"
    :role="type === 'error' ? 'alert' : 'status'"
    :aria-live="type === 'error' ? 'assertive' : 'polite'"
  >
    <div
      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg"
      :class="iconColourClass[type]"
    >
      <!-- Success icon -->
      <svg v-if="type === 'success'" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M20 6 9 17l-5-5" />
      </svg>
      <!-- Error icon -->
      <svg v-else-if="type === 'error'" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10" />
        <line x1="15" y1="9" x2="9" y2="15" />
        <line x1="9" y1="9" x2="15" y2="15" />
      </svg>
      <!-- Info icon -->
      <svg v-else class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10" />
        <path d="M12 16v-4" />
        <path d="M12 8h.01" />
      </svg>
    </div>

    <p class="text-sm text-text-primary">{{ message }}</p>

    <button
      class="ml-auto shrink-0 text-text-tertiary hover:text-text-primary transition-colors"
      aria-label="Dismiss notification"
      @click="emit('dismiss')"
    >
      <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.toast {
  background: rgba(22, 22, 26, 0.85);
  backdrop-filter: blur(20px) saturate(1.3);
  -webkit-backdrop-filter: blur(20px) saturate(1.3);
  border: 1px solid color-mix(in srgb, white 6%, transparent);
  min-width: 300px;
  max-width: 420px;
}

.icon-success {
  background: color-mix(in srgb, var(--color-success) 10%, transparent);
  color: var(--color-success);
}

.icon-error {
  background: color-mix(in srgb, var(--color-danger) 10%, transparent);
  color: var(--color-danger);
}

.icon-info {
  background: color-mix(in srgb, var(--color-accent) 10%, transparent);
  color: var(--color-accent);
}
</style>
