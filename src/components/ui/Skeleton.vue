<script setup lang="ts">
interface SkeletonProps {
  width?: string
  height?: string
  rounded?: 'sm' | 'md' | 'lg' | 'full'
}

withDefaults(defineProps<SkeletonProps>(), {
  width: '100%',
  height: '1rem',
  rounded: 'md',
})

const radiusMap: Record<string, string> = {
  sm: '4px',
  md: '6px',
  lg: '8px',
  full: '9999px',
}
</script>

<template>
  <div
    class="skeleton"
    :style="{
      width,
      height,
      borderRadius: radiusMap[rounded],
    }"
  />
</template>

<style scoped>
.skeleton {
  background: color-mix(in srgb, var(--color-surface-raised) 60%, transparent);
  border: 1px solid var(--color-border-subtle);
  box-shadow: var(--shadow-inset);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
