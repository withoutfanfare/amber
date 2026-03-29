import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  Snapshot,
  SnapshotCreatePayload,
  SnapshotProgress,
  SnapshotRestoreOptions,
  RestoreRecord,
  IntegrityResult,
  SizeEstimation,
  SchemaDiff,
  RestorePreview,
  VersionCompatibility,
  ExportResult,
  OrphanedFile,
} from "@/types";
import { useRestoreHistoryStore } from "./restoreHistory";

export const useSnapshotStore = defineStore("snapshots", () => {
  // --- State ---
  const snapshots = ref<Snapshot[]>([]);
  const loading = ref(false);
  const creating = ref(false);
  const restoring = ref(false);
  const error = ref<string | null>(null);
  const progress = ref<{
    phase: string;
    message: string;
    percentage: number;
    currentTable?: string;
    bytesProcessed?: number;
  } | null>(null);
  const allTags = ref<string[]>([]);
  const searchQuery = ref("");
  const filterTag = ref<string | null>(null);

  // --- Getters ---
  const sortedByDate = computed(() =>
    [...snapshots.value].sort(
      (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    ),
  );

  const totalSizeBytes = computed(() => snapshots.value.reduce((sum, s) => sum + s.sizeBytes, 0));

  const recentSnapshots = computed(() => sortedByDate.value.slice(0, 5));

  const filteredSnapshots = computed(() => {
    let result = snapshots.value;
    if (filterTag.value) {
      result = result.filter((s) => s.tags.includes(filterTag.value!));
    }
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.trim().toLowerCase();
      result = result.filter(
        (s) =>
          s.name.toLowerCase().includes(q) ||
          (s.note && s.note.toLowerCase().includes(q)) ||
          s.tags.some((t) => t.toLowerCase().includes(q)),
      );
    }
    return result;
  });

  // --- Actions ---
  async function fetchForProfile(profileId: string) {
    loading.value = true;
    error.value = null;
    try {
      snapshots.value = await invoke<Snapshot[]>("snapshot_list", { profileId });
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function fetchAll() {
    loading.value = true;
    error.value = null;
    try {
      snapshots.value = await invoke<Snapshot[]>("snapshot_list", { profileId: null });
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function fetchAllTags() {
    try {
      allTags.value = await invoke<string[]>("snapshot_list_all_tags");
    } catch {
      // Non-critical
    }
  }

  async function create(payload: SnapshotCreatePayload): Promise<Snapshot> {
    creating.value = true;
    progress.value = null;
    error.value = null;
    try {
      const onProgress = new Channel<SnapshotProgress>();
      onProgress.onmessage = (msg) => {
        switch (msg.event) {
          case "started":
            progress.value = { phase: "started", message: "Creating snapshot...", percentage: 0 };
            break;
          case "phase":
            progress.value = {
              phase: msg.data.phase,
              message: msg.data.message,
              percentage: progress.value?.percentage ?? 0,
            };
            break;
          case "progress":
            progress.value = { ...progress.value!, percentage: msg.data.percentage };
            break;
          case "tableProgress":
            progress.value = {
              phase: "dumping",
              message: `Dumping table: ${msg.data.currentTable}`,
              percentage: progress.value?.percentage ?? 0,
              currentTable: msg.data.currentTable,
              bytesProcessed: msg.data.bytesProcessed,
            };
            break;
          case "completed":
            progress.value = { phase: "completed", message: msg.data.message, percentage: 100 };
            break;
          case "failed":
            progress.value = null;
            break;
        }
      };
      const snapshot = await invoke<Snapshot>("snapshot_create", {
        profileId: payload.profileId,
        name: payload.name,
        note: payload.note ?? null,
        tags: payload.tags ?? null,
        onProgress,
      });
      snapshots.value.unshift(snapshot);
      return snapshot;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      creating.value = false;
    }
  }

  async function restore(id: string, options?: SnapshotRestoreOptions): Promise<void> {
    restoring.value = true;
    progress.value = null;
    error.value = null;
    try {
      const onProgress = new Channel<SnapshotProgress>();
      onProgress.onmessage = (msg) => {
        switch (msg.event) {
          case "started":
            progress.value = { phase: "started", message: "Restoring snapshot...", percentage: 0 };
            break;
          case "phase":
            progress.value = {
              phase: msg.data.phase,
              message: msg.data.message,
              percentage: progress.value?.percentage ?? 0,
            };
            break;
          case "progress":
            progress.value = { ...progress.value!, percentage: msg.data.percentage };
            break;
          case "completed":
            progress.value = { phase: "completed", message: msg.data.message, percentage: 100 };
            break;
          case "failed":
            progress.value = null;
            break;
        }
      };
      const record = await invoke<RestoreRecord>("snapshot_restore", {
        snapshotId: id,
        options: options ?? null,
        onProgress,
      });
      // Update restoredAt timestamp locally
      const snapshot = snapshots.value.find((s) => s.id === id);
      if (snapshot) {
        snapshot.restoredAt = new Date().toISOString();
      }
      // Push to restore history store
      const historyStore = useRestoreHistoryStore();
      historyStore.addLocal(record);
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      restoring.value = false;
    }
  }

  async function remove(id: string): Promise<void> {
    await invoke("snapshot_delete", { snapshotId: id });
    snapshots.value = snapshots.value.filter((s) => s.id !== id);
  }

  async function verifyIntegrity(id: string): Promise<IntegrityResult> {
    return await invoke<IntegrityResult>("snapshot_verify_integrity", { snapshotId: id });
  }

  async function addTags(snapshotId: string, tags: string[]): Promise<string[]> {
    const result = await invoke<string[]>("snapshot_add_tags", { snapshotId, tags });
    const snap = snapshots.value.find((s) => s.id === snapshotId);
    if (snap) snap.tags = result;
    await fetchAllTags();
    return result;
  }

  async function removeTag(snapshotId: string, tag: string): Promise<string[]> {
    const result = await invoke<string[]>("snapshot_remove_tag", { snapshotId, tag });
    const snap = snapshots.value.find((s) => s.id === snapshotId);
    if (snap) snap.tags = result;
    await fetchAllTags();
    return result;
  }

  async function setPinned(snapshotId: string, pinned: boolean): Promise<void> {
    await invoke("snapshot_set_pinned", { snapshotId, pinned });
    const snap = snapshots.value.find((s) => s.id === snapshotId);
    if (snap) snap.pinned = pinned;
  }

  async function estimateSize(profileId: string): Promise<SizeEstimation> {
    return await invoke<SizeEstimation>("snapshot_estimate_size", { profileId });
  }

  async function compareSchema(snapshotAId: string, snapshotBId: string): Promise<SchemaDiff> {
    return await invoke<SchemaDiff>("snapshot_compare_schema", {
      snapshotAId,
      snapshotBId,
    });
  }

  async function restorePreview(snapshotId: string): Promise<RestorePreview> {
    return await invoke<RestorePreview>("snapshot_restore_preview", { snapshotId });
  }

  async function checkVersionCompatibility(snapshotId: string): Promise<VersionCompatibility> {
    return await invoke<VersionCompatibility>("snapshot_check_version_compatibility", {
      snapshotId,
    });
  }

  async function exportSql(snapshotId: string, outputDir: string): Promise<ExportResult> {
    return await invoke<ExportResult>("snapshot_export_sql", { snapshotId, outputDir });
  }

  async function scanOrphans(): Promise<OrphanedFile[]> {
    return await invoke<OrphanedFile[]>("scan_orphaned_snapshots");
  }

  async function deleteOrphans(filePaths: string[]): Promise<number> {
    return await invoke<number>("delete_orphaned_snapshots", { filePaths });
  }

  return {
    snapshots,
    loading,
    creating,
    restoring,
    error,
    progress,
    allTags,
    searchQuery,
    filterTag,
    sortedByDate,
    totalSizeBytes,
    recentSnapshots,
    filteredSnapshots,
    fetchForProfile,
    fetchAll,
    fetchAllTags,
    create,
    restore,
    remove,
    verifyIntegrity,
    addTags,
    removeTag,
    setPinned,
    estimateSize,
    compareSchema,
    restorePreview,
    checkVersionCompatibility,
    exportSql,
    scanOrphans,
    deleteOrphans,
  };
});
