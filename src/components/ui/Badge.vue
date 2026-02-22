<script setup lang="ts">
interface BadgeProps {
  label: string
  variant?: 'success' | 'danger' | 'warning' | 'info' | 'accent' | 'muted' | 'default'
}

withDefaults(defineProps<BadgeProps>(), {
  variant: 'default',
})
</script>

<template>
  <span
    class="badge inline-flex items-center rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide border"
    :class="`badge-${variant}`"
  >
    {{ label }}
  </span>
</template>

<style scoped>
.badge-success {
  background: color-mix(in srgb, var(--color-success) 15%, transparent);
  border-color: color-mix(in srgb, var(--color-success) 20%, transparent);
  color: var(--color-success);
}

.badge-danger {
  background: color-mix(in srgb, var(--color-danger) 15%, transparent);
  border-color: color-mix(in srgb, var(--color-danger) 20%, transparent);
  color: var(--color-danger);
}

.badge-warning {
  background: color-mix(in srgb, var(--color-warning) 15%, transparent);
  border-color: color-mix(in srgb, var(--color-warning) 20%, transparent);
  color: var(--color-warning);
}

.badge-info {
  background: color-mix(in srgb, var(--color-info) 15%, transparent);
  border-color: color-mix(in srgb, var(--color-info) 20%, transparent);
  color: var(--color-info);
}

.badge-accent {
  background: color-mix(in srgb, var(--color-accent) 15%, transparent);
  border-color: color-mix(in srgb, var(--color-accent) 20%, transparent);
  color: var(--color-accent);
}

.badge-muted {
  background: color-mix(in srgb, var(--color-text-muted) 15%, transparent);
  border-color: color-mix(in srgb, var(--color-text-muted) 20%, transparent);
  color: var(--color-text-muted);
}

.badge-default {
  background: color-mix(in srgb, var(--color-text-secondary) 15%, transparent);
  border-color: color-mix(in srgb, var(--color-text-secondary) 20%, transparent);
  color: var(--color-text-secondary);
}
</style>
