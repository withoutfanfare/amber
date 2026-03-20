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
} from "@/types";
import { useRestoreHistoryStore } from "./restoreHistory";

export const useSnapshotStore = defineStore("snapshots", () => {
  // --- State ---
  const snapshots = ref<Snapshot[]>([]);
  const loading = ref(false);
  const creating = ref(false);
  const restoring = ref(false);
  const error = ref<string | null>(null);
  const progress = ref<{ phase: string; message: string; percentage: number } | null>(null);

  // --- Getters ---
  const sortedByDate = computed(() =>
    [...snapshots.value].sort(
      (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    ),
  );

  const totalSizeBytes = computed(() => snapshots.value.reduce((sum, s) => sum + s.sizeBytes, 0));

  const recentSnapshots = computed(() => sortedByDate.value.slice(0, 5));

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

  return {
    snapshots,
    loading,
    creating,
    restoring,
    error,
    progress,
    sortedByDate,
    totalSizeBytes,
    recentSnapshots,
    fetchForProfile,
    fetchAll,
    create,
    restore,
    remove,
    verifyIntegrity,
  };
});
