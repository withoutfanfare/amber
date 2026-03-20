<script setup lang="ts">
import { computed } from 'vue'
import { SCard, SButton } from '@stuntrocket/ui'
import type { ProjectStorage } from '@/types'

const props = defineProps<{
  usage: ProjectStorage[]
}>()

const emit = defineEmits<{
  deleteProject: [project: string]
  reveal: [project: string]
}>()

const maxBytes = computed(() =>
  Math.max(...props.usage.map(u => u.sizeBytes), 1)
)

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}
</script>

<template>
  <SCard>
    <h3 class="text-sm font-semibold text-text-primary mb-4">Storage by Project</h3>

    <div v-if="usage.length === 0" class="text-sm text-text-secondary py-4 text-center">
      No storage data available.
    </div>

    <div v-else class="space-y-4">
      <div v-for="item in usage" :key="item.project" class="space-y-1.5">
        <div class="flex items-center justify-between">
          <div>
            <span class="text-sm font-semibold text-text-primary">{{ item.project }}</span>
            <span class="text-sm text-text-secondary ml-2">
              {{ item.snapshotCount }} snapshot{{ item.snapshotCount !== 1 ? 's' : '' }}
              &middot; {{ formatSize(item.sizeBytes) }}
            </span>
          </div>
          <div class="flex items-center gap-1">
            <SButton
              variant="ghost"
              size="sm"
              @click="emit('reveal', item.project)"
              title="Show in Finder"
            >
              <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
              </svg>
            </SButton>
            <SButton
              variant="ghost"
              size="sm"
              class="text-danger hover:text-danger"
              @click="emit('deleteProject', item.project)"
            >
              Delete All
            </SButton>
          </div>
        </div>
        <div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-raised">
          <div
            class="h-full rounded-full bg-accent transition-all duration-300 ease-out"
            :style="{ width: `${(item.sizeBytes / maxBytes) * 100}%` }"
          />
        </div>
      </div>
    </div>
  </SCard>
</template>
