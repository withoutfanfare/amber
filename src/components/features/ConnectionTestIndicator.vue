<script setup lang="ts">
  import type { ConnectionTestResult } from "@/types";

  defineProps<{
    result: ConnectionTestResult | null;
    testing: boolean;
  }>();
</script>

<template>
  <div class="flex flex-col gap-1">
    <div class="flex items-center gap-2 text-sm">
      <!-- Spinner whilst testing -->
      <template v-if="testing">
        <svg
          class="h-4 w-4 animate-spin text-accent"
          viewBox="0 0 24 24"
          fill="none"
          aria-hidden="true"
        >
          <circle
            class="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            stroke-width="4"
          />
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
          />
        </svg>
        <span class="text-text-secondary">Testing connection&hellip;</span>
      </template>

      <!-- Success result -->
      <template v-else-if="result?.success">
        <svg
          class="h-4 w-4 text-success"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M20 6 9 17l-5-5" />
        </svg>
        <span class="text-success">Connected</span>
        <span v-if="result.latencyMs != null" class="text-xs text-text-tertiary">
          {{ result.latencyMs }}ms
        </span>
      </template>

      <!-- Failure result -->
      <template v-else-if="result && !result.success">
        <svg
          class="h-4 w-4 text-danger"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
        <span class="text-danger">{{ result.message }}</span>
      </template>
    </div>

    <!-- Remediation hint for failures -->
    <p
      v-if="result && !result.success && result.remediation"
      class="ml-6 text-xs text-text-tertiary"
    >
      {{ result.remediation }}
    </p>
  </div>
</template>
