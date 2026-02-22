<script setup lang="ts">
import AppShell from '@/components/layout/AppShell.vue'

// Pause blob animations when tab is hidden
document.addEventListener('visibilitychange', () => {
  document.documentElement.classList.toggle('page-hidden', document.hidden)
})
</script>

<template>
  <AppShell />
</template>
