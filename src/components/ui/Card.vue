<script setup lang="ts">
interface CardProps {
  variant?: 'default' | 'interactive' | 'flush' | 'inset'
}

withDefaults(defineProps<CardProps>(), {
  variant: 'default',
})
</script>

<template>
  <div
    :class="{
      'card rounded-xl p-5': variant === 'default',
      'card-interactive card rounded-xl p-5': variant === 'interactive',
      'card rounded-xl overflow-hidden': variant === 'flush',
      'card-inset rounded-xl': variant === 'inset',
    }"
  >
    <slot />
  </div>
</template>

<style scoped>
.card-interactive {
  transition: border-color 150ms ease, background-color 150ms ease;
}

.card-interactive:hover {
  border-color: var(--color-border-strong);
  background: color-mix(in srgb, #0e0e0e 50%, transparent);
}
</style>
