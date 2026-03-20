<script setup lang="ts">
  import { ref, computed, watch, onMounted } from "vue";
  import { useRouter } from "vue-router";
  import { invoke } from "@tauri-apps/api/core";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { useProfileStore } from "@/stores/profiles";
  import { useSnapshotStore } from "@/stores/snapshots";
  import { useToastStack } from "@stuntrocket/ui";
  import PageHeader from "@/components/layout/PageHeader.vue";
  import { SButton, SFormField, SSelect, SEmptyState, SConfirmDialog } from "@stuntrocket/ui";
  import SnapshotTable from "@/components/features/SnapshotTable.vue";
  import SnapshotRestoreDialog from "@/components/features/SnapshotRestoreDialog.vue";
  import type { Snapshot, SnapshotRestoreOptions } from "@/types";

  const snapshotsDir = ref("");

  const router = useRouter();
  const profileStore = useProfileStore();
  const snapshotStore = useSnapshotStore();
  const toast = useToastStack();

  const selectedProfileId = ref<string>("all");
  const restoreDialogOpen = ref(false);
  const deleteDialogOpen = ref(false);
  const targetSnapshot = ref<Snapshot | null>(null);

  const sourceProfile = computed(() => {
    if (!targetSnapshot.value) return null;
    return profileStore.profiles.find((p) => p.id === targetSnapshot.value!.profileId) ?? null;
  });

  const profileFilterOptions = computed(() => [
    { value: "all", label: "All Profiles" },
    ...profileStore.profiles.map((p) => ({
      value: p.id,
      label: `${p.project} / ${p.name}`,
    })),
  ]);

  onMounted(async () => {
    snapshotsDir.value = await invoke<string>("get_snapshots_dir");
    if (profileStore.profiles.length === 0) {
      await profileStore.fetchAll();
    }
    loadSnapshots();
  });

  watch(selectedProfileId, () => {
    loadSnapshots();
  });

  function loadSnapshots() {
    if (selectedProfileId.value === "all") {
      snapshotStore.fetchAll();
    } else {
      snapshotStore.fetchForProfile(selectedProfileId.value);
    }
  }

  function handleRestoreRequest(id: string) {
    targetSnapshot.value = snapshotStore.snapshots.find((s) => s.id === id) ?? null;
    restoreDialogOpen.value = true;
  }

  async function handleRestoreConfirm(options: SnapshotRestoreOptions) {
    if (!targetSnapshot.value) return;
    const name = targetSnapshot.value.name;
    restoreDialogOpen.value = false;
    try {
      await snapshotStore.restore(targetSnapshot.value.id, options);
      toast.success(`Snapshot "${name}" restored.`);
    } catch (e) {
      toast.error(`Failed to restore snapshot: ${e}`);
    } finally {
      targetSnapshot.value = null;
    }
  }

  async function handleReveal(snapshot: Snapshot) {
    if (!snapshotsDir.value) return;
    try {
      await revealItemInDir(`${snapshotsDir.value}/${snapshot.filePath}`);
    } catch (e) {
      toast.error(`Failed to reveal file: ${e}`);
    }
  }

  function handleDeleteRequest(id: string) {
    targetSnapshot.value = snapshotStore.snapshots.find((s) => s.id === id) ?? null;
    deleteDialogOpen.value = true;
  }

  async function handleDeleteConfirm() {
    if (!targetSnapshot.value) return;
    const name = targetSnapshot.value.name;
    deleteDialogOpen.value = false;
    try {
      await snapshotStore.remove(targetSnapshot.value.id);
      toast.success(`Snapshot "${name}" deleted.`);
    } catch (e) {
      toast.error(`Failed to delete snapshot: ${e}`);
    } finally {
      targetSnapshot.value = null;
    }
  }

  async function handleVerify(id: string) {
    try {
      const result = await snapshotStore.verifyIntegrity(id);
      if (result.valid) {
        toast.success("Integrity verified — checksum matches.");
      } else {
        toast.error(result.message);
      }
    } catch (e) {
      toast.error(`Integrity check failed: ${e}`);
    }
  }
</script>

<template>
  <div>
    <PageHeader>
      <template #prepend>
        <h1 class="text-lg font-semibold">Snapshots</h1>
        <p class="text-sm text-text-tertiary">
          Snapshots are compressed backups of your database at a point in time.
        </p>
      </template>
      <template #actions>
        <SButton variant="primary" size="sm" to="/snapshots/create">New Snapshot</SButton>
      </template>
    </PageHeader>

    <!-- Profile filter -->
    <div class="mb-5 max-w-xs">
      <SFormField label="Filter by profile">
        <SSelect v-model="selectedProfileId" size="sm">
          <option
            v-for="opt in profileFilterOptions"
            :key="opt.value"
            :value="opt.value"
          >{{ opt.label }}</option>
        </SSelect>
      </SFormField>
    </div>

    <!-- Progress indicator during restore -->
    <div v-if="snapshotStore.restoring && snapshotStore.progress" class="mb-5 space-y-2">
      <div class="h-2 w-full overflow-hidden rounded-full bg-surface-raised">
        <div
          class="h-full rounded-full bg-accent transition-all duration-300 ease-out"
          :style="{ width: `${snapshotStore.progress.percentage}%` }"
        />
      </div>
      <p class="text-xs text-text-secondary">{{ snapshotStore.progress.message }}</p>
    </div>

    <!-- Loading state -->
    <div v-if="snapshotStore.loading" class="py-8 text-center text-sm text-text-secondary">
      Loading snapshots&hellip;
    </div>

    <!-- Empty state -->
    <SEmptyState
      v-else-if="snapshotStore.snapshots.length === 0"
      title="No snapshots yet. Create one to capture your database state."
    >
      <template #action>
        <SButton variant="primary" size="sm" @click="router.push('/snapshots/create')">Create Snapshot</SButton>
      </template>
    </SEmptyState>

    <!-- Snapshot table -->
    <SnapshotTable
      v-else
      :snapshots="snapshotStore.snapshots"
      :profiles="profileStore.profiles"
      @restore="handleRestoreRequest"
      @reveal="handleReveal"
      @delete="handleDeleteRequest"
      @verify="handleVerify"
    />

    <!-- Restore dialog -->
    <SnapshotRestoreDialog
      :open="restoreDialogOpen"
      :snapshot="targetSnapshot"
      :profiles="profileStore.profiles"
      :source-profile="sourceProfile"
      @confirm="handleRestoreConfirm"
      @cancel="restoreDialogOpen = false"
    />

    <!-- Delete confirmation dialog -->
    <SConfirmDialog
      :open="deleteDialogOpen"
      title="Delete Snapshot"
      :message="`Are you sure you want to delete the snapshot &quot;${targetSnapshot?.name ?? ''}&quot;? The snapshot file will be permanently removed. This action cannot be undone.`"
      confirm-label="Delete"
      :danger="true"
      @confirm="handleDeleteConfirm"
      @cancel="deleteDialogOpen = false"
      @close="deleteDialogOpen = false"
    />
  </div>
</template>
