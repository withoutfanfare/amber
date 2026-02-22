<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'brand' | 'danger' | 'danger-outline' | 'ghost' | 'outline' | 'link'
  size?: 'sm' | 'md' | 'lg' | 'icon'
  to?: string
  type?: 'button' | 'submit' | 'reset'
  disabled?: boolean
  loading?: boolean
  block?: boolean
}

const props = withDefaults(defineProps<ButtonProps>(), {
  variant: 'primary',
  size: 'md',
  type: 'button',
  disabled: false,
  loading: false,
  block: false,
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

const sizeClasses: Record<string, string> = {
  sm: 'min-h-[28px] px-2.5 py-1.5 text-xs',
  md: 'min-h-[32px] px-4 py-1.5 text-sm',
  lg: 'min-h-[40px] px-5 py-2.5 text-base',
  icon: 'min-h-[32px] min-w-[32px] p-1.5',
}

const variantClasses: Record<string, string> = {
  primary: 'btn-solid text-accent',
  secondary: 'btn-solid text-text-secondary',
  brand: 'bg-accent-strong text-white hover:brightness-110',
  danger: 'btn-solid text-danger',
  'danger-outline': 'btn-solid text-danger',
  ghost: 'bg-transparent border-transparent text-text-secondary hover:bg-surface-overlay hover:text-text-primary',
  outline: 'btn-solid text-text-primary',
  link: 'bg-transparent border-0 shadow-none text-accent hover:underline p-0 h-auto min-h-0',
}

const classes = computed(() => {
  const base = 'inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-all duration-150 select-none'
  const size = sizeClasses[props.size]
  const variant = variantClasses[props.variant]
  const state = [
    'active:scale-[0.98]',
    props.disabled ? 'opacity-50 pointer-events-none' : '',
    props.loading ? 'opacity-70 pointer-events-none cursor-wait' : '',
    props.block ? 'w-full' : '',
  ].filter(Boolean).join(' ')

  return [base, size, variant, state].join(' ')
})

function handleClick(event: MouseEvent) {
  if (!props.disabled && !props.loading) {
    emit('click', event)
  }
}
</script>

<template>
  <RouterLink v-if="to" :to="to" :class="classes">
    <slot />
  </RouterLink>
  <button
    v-else
    :type="type"
    :disabled="disabled || loading"
    :class="classes"
    @click="handleClick"
  >
    <svg
      v-if="loading"
      class="h-4 w-4 animate-spin"
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden="true"
    >
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
    </svg>
    <template v-if="!loading">
      <slot />
    </template>
  </button>
</template>

<style scoped>
.btn-solid {
  background: var(--color-surface-base);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-sm);
}

.btn-solid:hover {
  border-color: var(--color-border-strong);
}

button:focus-visible,
a:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--color-accent) 55%, transparent);
  outline-offset: 2px;
}
</style>
