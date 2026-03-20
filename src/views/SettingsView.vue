<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { useToastStack, SCard, SInput, SPageHeader } from '@stuntrocket/ui'

const settingsStore = useSettingsStore()
const toast = useToastStack()

const autoPruneEnabled = ref(false)
const autoPruneDays = ref(30)

onMounted(async () => {
  await settingsStore.fetchAll()
  // Hydrate local state from store
  autoPruneEnabled.value = settingsStore.get('auto_prune_enabled', 'false') === 'true'
  autoPruneDays.value = parseInt(settingsStore.get('auto_prune_days', '30'), 10) || 30
})

const snapshotDir = computed(() =>
  settingsStore.get('snapshot_directory', '~/Library/Application Support/com.dannyharding.amber/snapshots')
)

// Debounced save for settings changes
let saveTimeout: ReturnType<typeof setTimeout> | null = null

function debouncedSave(key: string, value: string) {
  if (saveTimeout) clearTimeout(saveTimeout)
  saveTimeout = setTimeout(async () => {
    try {
      await settingsStore.set(key, value)
    } catch (e) {
      toast.error(`Failed to save setting: ${e}`)
    }
  }, 500)
}

watch(autoPruneEnabled, (val) => {
  debouncedSave('auto_prune_enabled', String(val))
})

watch(autoPruneDays, (val) => {
  debouncedSave('auto_prune_days', String(val))
})
</script>

<template>
  <div>
    <SPageHeader>
      <template #prepend>
        <h1 class="text-lg font-semibold">Settings</h1>
      </template>
    </SPageHeader>

    <div v-if="settingsStore.loading" class="text-sm text-text-secondary py-8 text-center">
      Loading settings&hellip;
    </div>

    <div v-else class="max-w-xl space-y-6">
      <!-- Snapshot directory -->
      <SCard>
        <h3 class="text-sm font-semibold text-text-primary mb-3">Snapshot Directory</h3>
        <p class="text-sm text-text-secondary mb-1">Snapshots are stored at:</p>
        <p class="text-sm font-mono text-text-primary bg-surface-raised rounded-lg px-3 py-2 border border-border-subtle">
          {{ snapshotDir }}
        </p>
      </SCard>

      <!-- Auto-prune settings -->
      <SCard>
        <h3 class="text-sm font-semibold text-text-primary mb-3">Auto-Prune</h3>
        <div class="space-y-3">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              v-model="autoPruneEnabled"
              type="checkbox"
              class="h-4 w-4 rounded border-border accent-accent"
            />
            <span class="text-sm text-text-secondary">Automatically delete old snapshots</span>
          </label>

          <div v-if="autoPruneEnabled" class="flex items-center gap-2 pl-6">
            <span class="text-sm text-text-secondary">Delete snapshots older than</span>
            <div class="w-20">
              <SInput
                v-model="autoPruneDays"
                type="number"
              />
            </div>
            <span class="text-sm text-text-secondary">days</span>
          </div>
        </div>
      </SCard>

      <!-- Compression -->
      <SCard>
        <h3 class="text-sm font-semibold text-text-primary mb-3">Compression</h3>
        <p class="text-sm text-text-secondary">
          Compression level: <span class="text-text-primary font-medium">Default (gzip)</span>
        </p>
      </SCard>
    </div>
  </div>
</template>
