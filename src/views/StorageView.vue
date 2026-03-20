<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { useSnapshotStore } from '@/stores/snapshots'
import { useToastStack, SCard, SButton, SInput, SConfirmDialog } from '@stuntrocket/ui'
import PageHeader from '@/components/layout/PageHeader.vue'
import StorageBreakdown from '@/components/features/StorageBreakdown.vue'
import type { StorageInfo } from '@/types'

const snapshotStore = useSnapshotStore()
const toast = useToastStack()

const storageInfo = ref<StorageInfo | null>(null)
const loading = ref(false)
const pruneDays = ref(30)
const pruning = ref(false)
const deleteProjectDialogOpen = ref(false)
const projectToDelete = ref<string>('')
const snapshotsDir = ref('')

onMounted(async () => {
  snapshotsDir.value = await invoke<string>('get_snapshots_dir')
  await loadStorage()
  if (snapshotStore.snapshots.length === 0) {
    await snapshotStore.fetchAll()
  }
})

async function loadStorage() {
  loading.value = true
  try {
    storageInfo.value = await invoke<StorageInfo>('storage_usage')
  } catch (e) {
    toast.error(`Failed to load storage info: ${e}`)
  } finally {
    loading.value = false
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

async function handlePrune() {
  const cutoff = Date.now() - pruneDays.value * 24 * 60 * 60 * 1000
  const oldSnapshots = snapshotStore.snapshots.filter(
    s => new Date(s.createdAt).getTime() < cutoff
  )

  if (oldSnapshots.length === 0) {
    toast.info(`No snapshots older than ${pruneDays.value} days.`)
    return
  }

  pruning.value = true
  try {
    for (const snapshot of oldSnapshots) {
      await snapshotStore.remove(snapshot.id)
    }
    toast.success(`Pruned ${oldSnapshots.length} snapshot${oldSnapshots.length !== 1 ? 's' : ''}.`)
    await loadStorage()
  } catch (e) {
    toast.error(`Failed to prune snapshots: ${e}`)
  } finally {
    pruning.value = false
  }
}

async function handleRevealProject(project: string) {
  if (!snapshotsDir.value) return
  try {
    await revealItemInDir(`${snapshotsDir.value}/${project}`)
  } catch (e) {
    toast.error(`Failed to reveal folder: ${e}`)
  }
}

function handleDeleteProjectRequest(project: string) {
  projectToDelete.value = project
  deleteProjectDialogOpen.value = true
}

async function handleDeleteProjectConfirm() {
  const project = projectToDelete.value
  deleteProjectDialogOpen.value = false

  try {
    await invoke('snapshot_delete_by_project', { project })
    toast.success(`All snapshots for "${project}" deleted.`)
    await snapshotStore.fetchAll()
    await loadStorage()
  } catch {
    toast.error(`Failed to delete snapshots for project "${project}".`)
  } finally {
    projectToDelete.value = ''
  }
}
</script>

<template>
  <div>
    <PageHeader>
      <template #prepend>
        <h1 class="text-lg font-semibold">Storage</h1>
        <p class="text-sm text-text-tertiary">Manage disk space used by your snapshot files.</p>
      </template>
    </PageHeader>

    <!-- Loading state -->
    <div v-if="loading" class="text-sm text-text-secondary py-8 text-center">
      Loading storage information&hellip;
    </div>

    <div v-else class="space-y-6">
      <!-- Total storage KPI -->
      <SCard v-if="storageInfo">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-xs text-text-secondary uppercase tracking-wide">Total Storage Used</p>
            <p class="text-2xl font-semibold text-text-primary">{{ formatSize(storageInfo.totalBytes) }}</p>
          </div>
          <div class="text-right">
            <p class="text-xs text-text-secondary uppercase tracking-wide">Snapshots</p>
            <p class="text-2xl font-semibold text-text-primary">
              {{ storageInfo.projects.reduce((sum, p) => sum + p.snapshotCount, 0) }}
            </p>
          </div>
        </div>
      </SCard>

      <!-- Per-project breakdown -->
      <StorageBreakdown
        v-if="storageInfo"
        :usage="storageInfo.projects"
        @reveal="handleRevealProject"
        @delete-project="handleDeleteProjectRequest"
      />

      <!-- Pruning controls -->
      <SCard>
        <h3 class="text-sm font-semibold text-text-primary mb-4">Prune Old Snapshots</h3>
        <div class="flex items-end gap-3">
          <div class="flex items-center gap-2">
            <span class="text-sm text-text-secondary whitespace-nowrap">Delete snapshots older than</span>
            <div class="w-20">
              <SInput
                v-model="pruneDays"
                type="number"
              />
            </div>
            <span class="text-sm text-text-secondary">days</span>
          </div>
          <SButton
            variant="danger"
            size="sm"
            :loading="pruning"
            @click="handlePrune"
          >
            Prune
          </SButton>
        </div>
      </SCard>
    </div>

    <!-- Delete project confirmation -->
    <SConfirmDialog
      :open="deleteProjectDialogOpen"
      title="Delete Project Snapshots"
      :message="`Are you sure you want to delete all snapshots for &quot;${projectToDelete}&quot;? This action cannot be undone.`"
      confirm-label="Delete All"
      :danger="true"
      @confirm="handleDeleteProjectConfirm"
      @cancel="deleteProjectDialogOpen = false"
      @close="deleteProjectDialogOpen = false"
    />
  </div>
</template>
