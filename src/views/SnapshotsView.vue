<script setup lang="ts">
  import { ref, computed, watch, onMounted, onUnmounted } from "vue";
  import { useRouter } from "vue-router";
  import { invoke } from "@tauri-apps/api/core";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { useProfileStore } from "@/stores/profiles";
  import { useSnapshotStore } from "@/stores/snapshots";
  import { useToastStack } from "@stuntrocket/ui";
  import {
    SButton,
    SFormField,
    SSelect,
    SInput,
    SEmptyState,
    SConfirmDialog,
  } from "@stuntrocket/ui";
  import PageHeader from "@/components/layout/PageHeader.vue";
  import SnapshotTable from "@/components/features/SnapshotTable.vue";
  import SnapshotRestoreDialog from "@/components/features/SnapshotRestoreDialog.vue";
  import type {
    Snapshot,
    SnapshotRestoreOptions,
    SchemaDiff,
    RestorePreview,
    SnapshotContent,
    SnapshotContentTable,
  } from "@/types";

  const snapshotsDir = ref("");

  const router = useRouter();
  const profileStore = useProfileStore();
  const snapshotStore = useSnapshotStore();
  const toast = useToastStack();

  const selectedProfileId = ref<string>("all");
  const restoreDialogOpen = ref(false);
  const deleteDialogOpen = ref(false);
  const targetSnapshot = ref<Snapshot | null>(null);
  const tagDialogOpen = ref(false);
  const tagTarget = ref<string>("");
  const newTagInput = ref("");
  const previewDialogOpen = ref(false);
  const previewData = ref<RestorePreview | null>(null);
  const previewLoading = ref(false);
  const compareDialogOpen = ref(false);
  const compareData = ref<SchemaDiff | null>(null);
  const compareLoading = ref(false);
  const compareIds = ref<string[]>([]);
  const browseDialogOpen = ref(false);
  const browseData = ref<SnapshotContent | null>(null);
  const browseLoading = ref(false);
  const browseSelectedTable = ref<SnapshotContentTable | null>(null);

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

  const displayedSnapshots = computed(() => {
    const filtered = snapshotStore.filteredSnapshots;
    if (selectedProfileId.value === "all") return filtered;
    return filtered.filter((s) => s.profileId === selectedProfileId.value);
  });

  onMounted(async () => {
    snapshotsDir.value = await invoke<string>("get_snapshots_dir");
    if (profileStore.profiles.length === 0) {
      await profileStore.fetchAll();
    }
    loadSnapshots();
    snapshotStore.fetchAllTags();
    window.addEventListener("keydown", handleKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown);
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

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "f") {
      e.preventDefault();
      const searchInput = document.getElementById("snapshot-search");
      if (searchInput) searchInput.focus();
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

  async function handleExport(id: string) {
    try {
      const exportDir = snapshotsDir.value.replace(/\/snapshots$/, "");
      const result = await snapshotStore.exportSql(id, exportDir);
      toast.success(`Exported to ${result.outputPath}`);
    } catch (e) {
      toast.error(`Export failed: ${e}`);
    }
  }

  async function handlePin(id: string, pinned: boolean) {
    try {
      await snapshotStore.setPinned(id, pinned);
      toast.success(pinned ? "Snapshot pinned." : "Snapshot unpinned.");
    } catch (e) {
      toast.error(`Failed to update pin status: ${e}`);
    }
  }

  function handleAddTagRequest(id: string) {
    tagTarget.value = id;
    newTagInput.value = "";
    tagDialogOpen.value = true;
  }

  async function handleAddTagConfirm() {
    if (!tagTarget.value || !newTagInput.value.trim()) return;
    const tags = newTagInput.value
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    try {
      await snapshotStore.addTags(tagTarget.value, tags);
      toast.success("Tag(s) added.");
    } catch (e) {
      toast.error(`Failed to add tag: ${e}`);
    } finally {
      tagDialogOpen.value = false;
      tagTarget.value = "";
    }
  }

  async function handleRemoveTag(id: string, tag: string) {
    try {
      await snapshotStore.removeTag(id, tag);
    } catch (e) {
      toast.error(`Failed to remove tag: ${e}`);
    }
  }

  async function handlePreview(id: string) {
    previewLoading.value = true;
    previewDialogOpen.value = true;
    previewData.value = null;
    try {
      previewData.value = await snapshotStore.restorePreview(id);
    } catch (e) {
      toast.error(`Preview failed: ${e}`);
      previewDialogOpen.value = false;
    } finally {
      previewLoading.value = false;
    }
  }

  async function handleCompare(_id: string) {
    // The SnapshotTable tracks its own selectedForCompare state.
    // We receive the first selected id; find both from the displayed snapshots with checkboxes.
    // For simplicity, use a pair selection dialog approach.
    const allSnaps = displayedSnapshots.value;
    if (allSnaps.length < 2) {
      toast.error("Need at least two snapshots to compare.");
      return;
    }
    // Use compareIds from the table selection
    if (compareIds.value.length === 2) {
      compareLoading.value = true;
      compareDialogOpen.value = true;
      compareData.value = null;
      try {
        compareData.value = await snapshotStore.compareSchema(
          compareIds.value[0],
          compareIds.value[1],
        );
      } catch (e) {
        toast.error(`Schema comparison failed: ${e}`);
        compareDialogOpen.value = false;
      } finally {
        compareLoading.value = false;
      }
    }
  }

  async function handleBrowse(id: string) {
    browseLoading.value = true;
    browseDialogOpen.value = true;
    browseData.value = null;
    browseSelectedTable.value = null;
    try {
      browseData.value = await snapshotStore.browseContent(id);
    } catch (e) {
      toast.error(`Failed to browse snapshot contents: ${e}`);
      browseDialogOpen.value = false;
    } finally {
      browseLoading.value = false;
    }
  }

  function selectBrowseTable(table: SnapshotContentTable) {
    browseSelectedTable.value =
      browseSelectedTable.value?.tableName === table.tableName ? null : table;
  }

  function formatRowCount(count: number): string {
    if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
    if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`;
    return count.toString();
  }

  function handleTagFilter(tag: string) {
    snapshotStore.filterTag = snapshotStore.filterTag === tag ? null : tag;
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

    <!-- Filter bar: profile + search + tags -->
    <div class="mb-5 flex flex-wrap items-end gap-3">
      <div class="w-48">
        <SFormField label="Filter by profile">
          <SSelect v-model="selectedProfileId" size="sm">
            <option v-for="opt in profileFilterOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </SSelect>
        </SFormField>
      </div>
      <div class="max-w-xs min-w-[200px] flex-1">
        <SFormField label="Search">
          <SInput
            id="snapshot-search"
            v-model="snapshotStore.searchQuery"
            placeholder="Search by name, note, or tag..."
          />
        </SFormField>
      </div>
      <div v-if="snapshotStore.allTags.length > 0" class="flex flex-wrap items-center gap-1.5">
        <span class="text-xs text-text-tertiary">Tags:</span>
        <button
          v-for="tag in snapshotStore.allTags"
          :key="tag"
          class="rounded-full px-2 py-0.5 text-[10px] font-medium transition-colors"
          :class="
            snapshotStore.filterTag === tag
              ? 'bg-accent text-white'
              : 'bg-accent/10 text-accent hover:bg-accent/20'
          "
          @click="handleTagFilter(tag)"
        >
          {{ tag }}
        </button>
      </div>
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
      v-else-if="displayedSnapshots.length === 0"
      title="No snapshots yet. Create one to capture your database state."
    >
      <template #action>
        <SButton variant="primary" size="sm" @click="router.push('/snapshots/create')"
          >Create Snapshot</SButton
        >
      </template>
    </SEmptyState>

    <!-- Snapshot table -->
    <SnapshotTable
      v-else
      :snapshots="displayedSnapshots"
      :profiles="profileStore.profiles"
      @restore="handleRestoreRequest"
      @reveal="handleReveal"
      @delete="handleDeleteRequest"
      @verify="handleVerify"
      @export="handleExport"
      @pin="handlePin"
      @add-tag="handleAddTagRequest"
      @remove-tag="handleRemoveTag"
      @compare="handleCompare"
      @preview="handlePreview"
      @browse="handleBrowse"
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

    <!-- Add tag dialog -->
    <SConfirmDialog
      :open="tagDialogOpen"
      title="Add Tags"
      message="Enter one or more tags, separated by commas."
      confirm-label="Add"
      @confirm="handleAddTagConfirm"
      @cancel="tagDialogOpen = false"
      @close="tagDialogOpen = false"
    >
      <template #default>
        <div class="mb-4">
          <SInput
            v-model="newTagInput"
            placeholder="pre-migration, v2.1, clean-state"
            @keydown.enter.prevent="handleAddTagConfirm"
          />
          <div v-if="snapshotStore.allTags.length > 0" class="mt-2 flex flex-wrap gap-1">
            <span class="text-xs text-text-tertiary">Suggestions:</span>
            <button
              v-for="tag in snapshotStore.allTags"
              :key="tag"
              class="rounded-full bg-surface-raised px-2 py-0.5 text-[10px] text-text-secondary transition-colors hover:bg-accent/10 hover:text-accent"
              @click="newTagInput = newTagInput ? `${newTagInput}, ${tag}` : tag"
            >
              {{ tag }}
            </button>
          </div>
        </div>
      </template>
    </SConfirmDialog>

    <!-- Restore preview dialog -->
    <SConfirmDialog
      :open="previewDialogOpen"
      title="Restore Preview"
      message=""
      confirm-label="Close"
      @confirm="previewDialogOpen = false"
      @cancel="previewDialogOpen = false"
      @close="previewDialogOpen = false"
    >
      <template #default>
        <div v-if="previewLoading" class="py-4 text-center text-sm text-text-secondary">
          Analysing snapshot and current database&hellip;
        </div>
        <div v-else-if="previewData" class="space-y-4">
          <p class="text-sm font-medium text-text-primary">{{ previewData.snapshotName }}</p>
          <div
            v-if="previewData.warnings.length > 0"
            class="rounded-lg border border-warning/30 bg-warning/10 px-3 py-2"
          >
            <p v-for="(warning, i) in previewData.warnings" :key="i" class="text-sm text-warning">
              {{ warning }}
            </p>
          </div>
          <div class="grid grid-cols-3 gap-3 text-center">
            <div class="rounded-lg bg-surface-raised px-3 py-2">
              <p class="text-lg font-semibold text-text-primary">
                {{ previewData.tablesInCommon.length }}
              </p>
              <p class="text-xs text-text-secondary">Common</p>
            </div>
            <div class="rounded-lg bg-success/10 px-3 py-2">
              <p class="text-lg font-semibold text-success">
                +{{ previewData.tablesToAdd.length }}
              </p>
              <p class="text-xs text-text-secondary">To add</p>
            </div>
            <div class="rounded-lg bg-danger/10 px-3 py-2">
              <p class="text-lg font-semibold text-danger">
                -{{ previewData.tablesToRemove.length }}
              </p>
              <p class="text-xs text-text-secondary">To remove</p>
            </div>
          </div>
          <div v-if="previewData.tablesToAdd.length > 0">
            <p class="mb-1 text-xs font-semibold tracking-wide text-success uppercase">
              Tables to add
            </p>
            <p class="text-sm text-text-secondary">{{ previewData.tablesToAdd.join(", ") }}</p>
          </div>
          <div v-if="previewData.tablesToRemove.length > 0">
            <p class="mb-1 text-xs font-semibold tracking-wide text-danger uppercase">
              Tables to remove
            </p>
            <p class="text-sm text-text-secondary">{{ previewData.tablesToRemove.join(", ") }}</p>
          </div>
        </div>
      </template>
    </SConfirmDialog>

    <!-- Schema compare dialog -->
    <SConfirmDialog
      :open="compareDialogOpen"
      title="Schema Comparison"
      message=""
      confirm-label="Close"
      @confirm="compareDialogOpen = false"
      @cancel="compareDialogOpen = false"
      @close="compareDialogOpen = false"
    >
      <template #default>
        <div v-if="compareLoading" class="py-4 text-center text-sm text-text-secondary">
          Comparing schemas&hellip;
        </div>
        <div v-else-if="compareData" class="space-y-4">
          <div class="flex items-center gap-2 text-sm">
            <span class="font-medium text-text-primary">{{ compareData.snapshotAName }}</span>
            <span class="text-text-tertiary">&rarr;</span>
            <span class="font-medium text-text-primary">{{ compareData.snapshotBName }}</span>
          </div>
          <p class="text-sm text-text-secondary">{{ compareData.summary }}</p>
          <div v-if="compareData.tablesAdded.length > 0">
            <p class="mb-1 text-xs font-semibold tracking-wide text-success uppercase">
              Tables added
            </p>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="t in compareData.tablesAdded"
                :key="t"
                class="rounded bg-success/10 px-2 py-0.5 text-xs text-success"
                >{{ t }}</span
              >
            </div>
          </div>
          <div v-if="compareData.tablesRemoved.length > 0">
            <p class="mb-1 text-xs font-semibold tracking-wide text-danger uppercase">
              Tables removed
            </p>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="t in compareData.tablesRemoved"
                :key="t"
                class="rounded bg-danger/10 px-2 py-0.5 text-xs text-danger"
                >{{ t }}</span
              >
            </div>
          </div>
          <div v-if="compareData.tablesModified.length > 0">
            <p class="mb-1 text-xs font-semibold tracking-wide text-accent uppercase">
              Tables modified
            </p>
            <div
              v-for="td in compareData.tablesModified"
              :key="td.tableName"
              class="mb-2 rounded-lg bg-surface-raised px-3 py-2"
            >
              <p class="mb-1 text-sm font-medium text-text-primary">{{ td.tableName }}</p>
              <div v-if="td.columnsAdded.length > 0" class="text-xs">
                <span class="text-success">+ {{ td.columnsAdded.join(", ") }}</span>
              </div>
              <div v-if="td.columnsRemoved.length > 0" class="text-xs">
                <span class="text-danger">- {{ td.columnsRemoved.join(", ") }}</span>
              </div>
            </div>
          </div>
        </div>
      </template>
    </SConfirmDialog>

    <!-- Snapshot content browser dialog -->
    <SConfirmDialog
      :open="browseDialogOpen"
      title="Browse Snapshot Contents"
      message=""
      confirm-label="Close"
      @confirm="browseDialogOpen = false"
      @cancel="browseDialogOpen = false"
      @close="browseDialogOpen = false"
    >
      <template #default>
        <div v-if="browseLoading" class="py-4 text-center text-sm text-text-secondary">
          Extracting snapshot contents&hellip;
        </div>
        <div v-else-if="browseData" class="space-y-4">
          <!-- Header -->
          <div class="flex items-center justify-between">
            <p class="text-sm font-medium text-text-primary">{{ browseData.snapshotName }}</p>
            <div class="flex items-center gap-3 text-xs text-text-tertiary">
              <span>{{ browseData.tables.length }} tables</span>
              <span>{{ formatRowCount(browseData.totalRows) }} rows</span>
              <span class="rounded bg-surface-raised px-1.5 py-0.5 font-mono uppercase">{{
                browseData.dbType
              }}</span>
            </div>
          </div>

          <!-- Table list -->
          <div class="max-h-48 overflow-y-auto rounded-lg border border-border-subtle">
            <table class="w-full text-xs">
              <thead>
                <tr
                  class="sticky top-0 border-b border-border-subtle bg-surface-base text-left text-text-tertiary uppercase"
                >
                  <th class="px-3 py-1.5 font-semibold">Table</th>
                  <th class="px-3 py-1.5 text-right font-semibold">Columns</th>
                  <th class="px-3 py-1.5 text-right font-semibold">Rows</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="table in browseData.tables"
                  :key="table.tableName"
                  class="cursor-pointer border-b border-border-subtle transition-colors last:border-0 hover:bg-surface-raised/50"
                  :class="{
                    'bg-accent/5': browseSelectedTable?.tableName === table.tableName,
                  }"
                  @click="selectBrowseTable(table)"
                >
                  <td class="px-3 py-1.5 font-mono font-medium text-text-primary">
                    {{ table.tableName }}
                  </td>
                  <td class="px-3 py-1.5 text-right text-text-secondary">
                    {{ table.columns.length }}
                  </td>
                  <td class="px-3 py-1.5 text-right font-mono text-text-secondary">
                    {{ formatRowCount(table.rowCount) }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Selected table detail -->
          <div v-if="browseSelectedTable" class="space-y-3">
            <div class="flex items-center justify-between">
              <p class="text-sm font-medium text-text-primary">
                {{ browseSelectedTable.tableName }}
              </p>
              <span class="text-xs text-text-tertiary"
                >{{ browseSelectedTable.columns.length }} columns &middot;
                {{ formatRowCount(browseSelectedTable.rowCount) }} rows</span
              >
            </div>

            <!-- Column list -->
            <div class="flex flex-wrap gap-1">
              <span
                v-for="col in browseSelectedTable.columns"
                :key="col"
                class="rounded bg-surface-raised px-2 py-0.5 font-mono text-[10px] text-text-secondary"
                >{{ col }}</span
              >
            </div>

            <!-- Sample data grid -->
            <div
              v-if="browseSelectedTable.sampleRows.length > 0"
              class="max-h-60 overflow-auto rounded-lg border border-border-subtle"
            >
              <table class="w-full text-xs">
                <thead>
                  <tr
                    class="sticky top-0 border-b border-border-subtle bg-surface-base text-left text-text-tertiary"
                  >
                    <th class="px-2 py-1 text-center font-semibold">#</th>
                    <th
                      v-for="col in browseSelectedTable.columns"
                      :key="col"
                      class="max-w-[150px] truncate px-2 py-1 font-mono font-semibold"
                    >
                      {{ col }}
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="(row, idx) in browseSelectedTable.sampleRows"
                    :key="idx"
                    class="border-b border-border-subtle last:border-0"
                  >
                    <td class="px-2 py-1 text-center text-text-tertiary">{{ idx + 1 }}</td>
                    <td
                      v-for="(cell, ci) in row"
                      :key="ci"
                      class="max-w-[150px] truncate px-2 py-1 font-mono text-text-secondary"
                      :class="{ 'text-text-tertiary italic': cell === 'NULL' }"
                      :title="cell"
                    >
                      {{ cell }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            <p
              v-else
              class="rounded-lg bg-surface-raised px-3 py-2 text-center text-xs text-text-tertiary"
            >
              No data rows found for this table.
            </p>

            <p
              v-if="
                browseSelectedTable.sampleRows.length > 0 &&
                browseSelectedTable.rowCount > browseSelectedTable.sampleRows.length
              "
              class="text-[10px] text-text-tertiary"
            >
              Showing {{ browseSelectedTable.sampleRows.length }} of
              {{ formatRowCount(browseSelectedTable.rowCount) }} rows.
            </p>
          </div>

          <!-- Prompt to select a table -->
          <p
            v-else
            class="rounded-lg bg-surface-raised px-3 py-2 text-center text-xs text-text-tertiary"
          >
            Click a table above to view its columns and sample data.
          </p>
        </div>
      </template>
    </SConfirmDialog>
  </div>
</template>
