<script setup lang="ts">
defineProps<{
  label: string
  value: string | number
  icon?: string
}>()
</script>

<template>
  <div class="card rounded-xl p-4">
    <div class="flex items-start gap-3">
      <!-- Icon -->
      <div v-if="icon" class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-surface-raised">
        <!-- Database icon -->
        <svg v-if="icon === 'database'" class="h-4.5 w-4.5 text-text-tertiary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <ellipse cx="12" cy="5" rx="9" ry="3" />
          <path d="M3 5V19A9 3 0 0 0 21 19V5" />
          <path d="M3 12A9 3 0 0 0 21 12" />
        </svg>
        <!-- Camera / snapshot icon -->
        <svg v-else-if="icon === 'camera'" class="h-4.5 w-4.5 text-text-tertiary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z" />
          <circle cx="12" cy="13" r="3" />
        </svg>
        <!-- Hard drive / storage icon -->
        <svg v-else-if="icon === 'hard-drive'" class="h-4.5 w-4.5 text-text-tertiary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="22" y1="12" x2="2" y2="12" />
          <path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />
          <line x1="6" y1="16" x2="6.01" y2="16" />
          <line x1="10" y1="16" x2="10.01" y2="16" />
        </svg>
        <!-- Clock icon -->
        <svg v-else-if="icon === 'clock'" class="h-4.5 w-4.5 text-text-tertiary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <polyline points="12 6 12 12 16 14" />
        </svg>
      </div>

      <div class="min-w-0">
        <p class="text-xs text-text-secondary uppercase tracking-wide">{{ label }}</p>
        <p class="text-2xl font-semibold text-text-primary truncate">{{ value }}</p>
      </div>
    </div>
  </div>
</template>
