<script setup lang="ts">
import { ref, computed } from 'vue'
import { SCard, SBadge, SButton } from '@stuntrocket/ui'
import ConnectionTestIndicator from './ConnectionTestIndicator.vue'
import type { Profile, ConnectionTestResult } from '@/types'

const props = defineProps<{
  profile: Profile
  testResult?: ConnectionTestResult | null
}>()

const emit = defineEmits<{
  edit: []
  delete: []
  test: []
}>()

const testing = ref(false)

const dbBadgeVariant = computed(() => {
  const map: Record<string, 'info' | 'success' | 'warning'> = {
    mysql: 'info',
    postgresql: 'success',
    sqlite: 'warning',
  }
  return map[props.profile.dbType] ?? 'default'
})

const envBadgeVariant = computed(() => {
  const map: Record<string, 'default' | 'warning' | 'danger'> = {
    local: 'default',
    staging: 'warning',
    live: 'danger',
  }
  return props.profile.environment ? map[props.profile.environment] ?? 'default' : 'default'
})

const connectionString = computed(() => {
  if (props.profile.dbType === 'sqlite') {
    return props.profile.databaseName
  }
  const host = props.profile.host ?? '127.0.0.1'
  const port = props.profile.port ? `:${props.profile.port}` : ''
  return `${host}${port}/${props.profile.databaseName}`
})

async function handleTest() {
  testing.value = true
  emit('test')
  // Parent handles the actual test; we show loading state briefly
  // The testing state is cleared when testResult changes
  setTimeout(() => { testing.value = false }, 5000)
}
</script>

<template>
  <SCard :hoverable="true">
    <div class="flex items-start justify-between mb-3">
      <div>
        <p class="text-text-tertiary text-xs">{{ profile.project }}</p>
        <h3 class="text-lg font-semibold text-text-primary">{{ profile.name }}</h3>
      </div>
      <div class="flex items-center gap-1">
        <!-- Edit button -->
        <SButton variant="ghost" size="sm" @click="emit('edit')">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
            <path d="m15 5 4 4" />
          </svg>
        </SButton>
        <!-- Delete button -->
        <SButton variant="ghost" size="sm" class="text-danger hover:text-danger" @click="emit('delete')">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
            <line x1="10" y1="11" x2="10" y2="17" />
            <line x1="14" y1="11" x2="14" y2="17" />
          </svg>
        </SButton>
      </div>
    </div>

    <div class="flex items-center gap-2 mb-3">
      <SBadge :variant="dbBadgeVariant">{{ profile.dbType }}</SBadge>
      <SBadge v-if="profile.environment" :variant="envBadgeVariant">{{ profile.environment }}</SBadge>
      <SBadge v-if="profile.sshEnabled" variant="accent">SSH</SBadge>
    </div>

    <p class="text-sm font-mono text-text-secondary mb-2">{{ connectionString }}</p>

    <p v-if="profile.notes" class="text-xs text-text-tertiary line-clamp-2 mb-4">
      {{ profile.notes }}
    </p>

    <div class="flex items-center justify-between border-t border-border-subtle pt-3 mt-3">
      <ConnectionTestIndicator :result="testResult ?? null" :testing="testing" />
      <SButton variant="secondary" size="sm" @click="handleTest">
        Test Connection
      </SButton>
    </div>
  </SCard>
</template>
