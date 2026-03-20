<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useProfileStore } from '@/stores/profiles'
import { useSnapshotStore } from '@/stores/snapshots'
import { useRestoreHistoryStore } from '@/stores/restoreHistory'
import { SCard, SButton, SKpiCard, SPageHeader } from '@stuntrocket/ui'
import type { StorageInfo } from '@/types'

const profileStore = useProfileStore()
const snapshotStore = useSnapshotStore()
const restoreHistoryStore = useRestoreHistoryStore()

const storageInfo = ref<StorageInfo | null>(null)

onMounted(async () => {
  await Promise.all([
    profileStore.fetchAll(),
    snapshotStore.fetchAll(),
    restoreHistoryStore.fetchAll(),
    loadStorage(),
  ])
})

async function loadStorage() {
  try {
    storageInfo.value = await invoke<StorageInfo>('storage_usage')
  } catch {
    // Storage info may not be available yet
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

const totalStorage = computed(() => {
  if (storageInfo.value) return formatSize(storageInfo.value.totalBytes)
  return formatSize(snapshotStore.totalSizeBytes)
})

const lastRestore = computed(() => {
  const record = restoreHistoryStore.lastRestore
  if (!record) return 'Never'
  return record.snapshotName
})

function formatDuration(secs: number): string {
  if (secs < 1) return `${(secs * 1000).toFixed(0)}ms`
  if (secs < 60) return `${secs.toFixed(1)}s`
  return `${Math.floor(secs / 60)}m ${Math.round(secs % 60)}s`
}

function getProfileNameById(profileId: string): string {
  const profile = profileStore.profiles.find(p => p.id === profileId)
  return profile ? `${profile.project} / ${profile.name}` : 'Unknown'
}

function formatDate(iso: string): string {
  const date = new Date(iso)
  return date.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' })
}

function getProfileName(profileId: string): string {
  const profile = profileStore.profiles.find(p => p.id === profileId)
  return profile ? `${profile.project} / ${profile.name}` : 'Unknown'
}
</script>

<template>
  <div>
    <SPageHeader>
      <template #prepend>
        <h1 class="text-lg font-semibold">Dashboard</h1>
      </template>
    </SPageHeader>

    <div class="space-y-6 stagger-fade-in">
      <!-- KPI Grid -->
      <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
        <SKpiCard label="Profiles" :value="profileStore.profiles.length">
          <template #icon>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <ellipse cx="12" cy="5" rx="9" ry="3" />
              <path d="M3 5V19A9 3 0 0 0 21 19V5" />
              <path d="M3 12A9 3 0 0 0 21 12" />
            </svg>
          </template>
        </SKpiCard>
        <SKpiCard label="Snapshots" :value="snapshotStore.snapshots.length">
          <template #icon>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z" />
              <circle cx="12" cy="13" r="3" />
            </svg>
          </template>
        </SKpiCard>
        <SKpiCard label="Storage" :value="totalStorage">
          <template #icon>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="22" y1="12" x2="2" y2="12" />
              <path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />
              <line x1="6" y1="16" x2="6.01" y2="16" />
              <line x1="10" y1="16" x2="10.01" y2="16" />
            </svg>
          </template>
        </SKpiCard>
        <SKpiCard label="Last Restore" :value="lastRestore">
          <template #icon>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" />
              <polyline points="12 6 12 12 16 14" />
            </svg>
          </template>
        </SKpiCard>
      </div>

      <!-- Recent Snapshots -->
      <SCard>
        <h3 class="text-sm font-semibold text-text-primary mb-3">Recent Snapshots</h3>
        <div v-if="snapshotStore.recentSnapshots.length === 0" class="text-sm text-text-secondary py-4 text-center">
          No snapshots yet.
        </div>
        <table v-else class="w-full text-sm">
          <thead>
            <tr class="border-b border-border-subtle text-left text-xs text-text-tertiary uppercase tracking-wide">
              <th class="px-2 py-2 font-semibold">Name</th>
              <th class="px-2 py-2 font-semibold">Profile</th>
              <th class="px-2 py-2 font-semibold">Created</th>
              <th class="px-2 py-2 font-semibold text-right">Size</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="snapshot in snapshotStore.recentSnapshots"
              :key="snapshot.id"
              class="border-b border-border-subtle"
            >
              <td class="px-2 py-2 font-medium text-text-primary">{{ snapshot.name }}</td>
              <td class="px-2 py-2 text-text-secondary">{{ getProfileName(snapshot.profileId) }}</td>
              <td class="px-2 py-2 text-text-secondary whitespace-nowrap">{{ formatDate(snapshot.createdAt) }}</td>
              <td class="px-2 py-2 text-text-secondary text-right font-mono">{{ formatSize(snapshot.sizeBytes) }}</td>
            </tr>
          </tbody>
        </table>
      </SCard>

      <!-- Recent Restores -->
      <SCard>
        <h3 class="text-sm font-semibold text-text-primary mb-3">Recent Restores</h3>
        <div v-if="restoreHistoryStore.recentRestores.length === 0" class="text-sm text-text-secondary py-4 text-center">
          No restores yet.
        </div>
        <table v-else class="w-full text-sm">
          <thead>
            <tr class="border-b border-border-subtle text-left text-xs text-text-tertiary uppercase tracking-wide">
              <th class="px-2 py-2 font-semibold">Snapshot</th>
              <th class="px-2 py-2 font-semibold">Target Profile</th>
              <th class="px-2 py-2 font-semibold">Database</th>
              <th class="px-2 py-2 font-semibold">When</th>
              <th class="px-2 py-2 font-semibold text-right">Duration</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="record in restoreHistoryStore.recentRestores"
              :key="record.id"
              class="border-b border-border-subtle"
            >
              <td class="px-2 py-2 font-medium text-text-primary">{{ record.snapshotName }}</td>
              <td class="px-2 py-2 text-text-secondary">{{ getProfileNameById(record.targetProfileId) }}</td>
              <td class="px-2 py-2 text-text-secondary font-mono">{{ record.targetDbName }}</td>
              <td class="px-2 py-2 text-text-secondary whitespace-nowrap">{{ formatDate(record.restoredAt) }}</td>
              <td class="px-2 py-2 text-text-secondary text-right font-mono">{{ formatDuration(record.durationSecs) }}</td>
            </tr>
          </tbody>
        </table>
      </SCard>

      <!-- Quick Actions -->
      <SCard>
        <h3 class="text-sm font-semibold text-text-primary mb-3">Quick Actions</h3>
        <div class="flex flex-wrap gap-3">
          <SButton variant="primary" size="sm" to="/snapshots/create">Create Snapshot</SButton>
          <SButton variant="secondary" size="sm" to="/profiles/create">New Profile</SButton>
          <SButton variant="secondary" size="sm" to="/storage">View Storage</SButton>
        </div>
      </SCard>
    </div>
  </div>
</template>
