import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { useSnapshotStore } from "../snapshots";
import { useProfileStore } from "../profiles";
import type { Snapshot } from "@/types";
import type { RestoreRecord } from "@/types";

const mockInvoke = vi.mocked(invoke);

const makeSnapshot = (databaseName: string): Snapshot => ({
  id: `snap-${databaseName}`,
  profileId: "prof-1",
  databaseName,
  name: "Before migration",
  note: null,
  filePath: `project/${databaseName}.sql.gz`,
  sizeBytes: 1024,
  dbVersion: null,
  dumpToolVersion: null,
  checksum: null,
  restoreTestStatus: "passed",
  restoreTestMessage: "Local restore test passed.",
  restoreTestedAt: "2026-01-01T00:00:00Z",
  createdAt: "2026-01-01T00:00:00Z",
  restoredAt: null,
  pinned: false,
  tags: [],
});

const mockRestoreRecord: RestoreRecord = {
  id: "rec-1",
  snapshotId: "snap-1",
  snapshotName: "Test",
  targetProfileId: "prof-1",
  targetDbName: "mydb",
  durationSecs: 0.5,
  restoredAt: "2026-01-01T00:00:00Z",
};

describe("useSnapshotStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  describe("create", () => {
    it("creates one snapshot for every database in the profile", async () => {
      const profileStore = useProfileStore();
      profileStore.profiles = [
        {
          id: "prof-1",
          project: "Scooda",
          name: "Local",
          dbType: "mysql",
          host: "127.0.0.1",
          port: 3306,
          databaseName: "scooda_landlord",
          databaseNames: ["scooda_landlord", "scooda_tenant_1"],
          username: "root",
          sshEnabled: false,
          sshHost: null,
          sshPort: 22,
          sshUser: null,
          environment: "local",
          notes: null,
          createdAt: "2026-01-01T00:00:00Z",
          updatedAt: "2026-01-01T00:00:00Z",
        },
      ];
      mockInvoke
        .mockResolvedValueOnce(makeSnapshot("scooda_landlord"))
        .mockResolvedValueOnce(makeSnapshot("scooda_tenant_1"));

      const store = useSnapshotStore();
      const created = await store.create({ profileId: "prof-1", name: "Before migration" });

      expect(created).toHaveLength(2);
      expect(mockInvoke).toHaveBeenNthCalledWith(
        1,
        "snapshot_create",
        expect.objectContaining({ databaseName: "scooda_landlord" }),
      );
      expect(mockInvoke).toHaveBeenNthCalledWith(
        2,
        "snapshot_create",
        expect.objectContaining({ databaseName: "scooda_tenant_1" }),
      );
    });

    it("attempts the remaining databases when one snapshot fails", async () => {
      const profileStore = useProfileStore();
      profileStore.profiles = [
        {
          id: "prof-1",
          project: "Scooda",
          name: "Local",
          dbType: "mysql",
          host: "127.0.0.1",
          port: 3306,
          databaseName: "scooda_landlord",
          databaseNames: ["scooda_landlord", "scooda_tenant_1"],
          username: "root",
          sshEnabled: false,
          sshHost: null,
          sshPort: 22,
          sshUser: null,
          environment: "local",
          notes: null,
          createdAt: "2026-01-01T00:00:00Z",
          updatedAt: "2026-01-01T00:00:00Z",
        },
      ];
      mockInvoke
        .mockRejectedValueOnce(new Error("database unavailable"))
        .mockResolvedValueOnce(makeSnapshot("scooda_tenant_1"));

      const store = useSnapshotStore();
      await expect(store.create({ profileId: "prof-1", name: "Before migration" })).rejects.toThrow(
        "1 of 2 database snapshots created",
      );

      expect(mockInvoke).toHaveBeenCalledTimes(2);
      expect(store.snapshots.map((snapshot) => snapshot.databaseName)).toEqual(["scooda_tenant_1"]);
    });
  });

  describe("restore", () => {
    it("calls invoke with snapshotId and null options when no options passed", async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord);
      const store = useSnapshotStore();
      await store.restore("snap-1");
      expect(mockInvoke).toHaveBeenCalledWith("snapshot_restore", {
        snapshotId: "snap-1",
        options: null,
        onProgress: expect.anything(),
      });
    });

    it("passes targetProfileId in options when provided", async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord);
      const store = useSnapshotStore();
      await store.restore("snap-1", { targetProfileId: "profile-2" });
      expect(mockInvoke).toHaveBeenCalledWith("snapshot_restore", {
        snapshotId: "snap-1",
        options: { targetProfileId: "profile-2" },
        onProgress: expect.anything(),
      });
    });

    it("passes targetDatabaseName in options when provided", async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord);
      const store = useSnapshotStore();
      await store.restore("snap-1", { targetDatabaseName: "new_db" });
      expect(mockInvoke).toHaveBeenCalledWith("snapshot_restore", {
        snapshotId: "snap-1",
        options: { targetDatabaseName: "new_db" },
        onProgress: expect.anything(),
      });
    });

    it("passes both overrides together", async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord);
      const store = useSnapshotStore();
      await store.restore("snap-1", {
        targetProfileId: "profile-2",
        targetDatabaseName: "new_db",
      });
      expect(mockInvoke).toHaveBeenCalledWith("snapshot_restore", {
        snapshotId: "snap-1",
        options: { targetProfileId: "profile-2", targetDatabaseName: "new_db" },
        onProgress: expect.anything(),
      });
    });

    it("sets restoring flag during operation", async () => {
      let resolveInvoke: (value: unknown) => void;
      mockInvoke.mockReturnValueOnce(
        new Promise((resolve) => {
          resolveInvoke = resolve;
        }),
      );
      const store = useSnapshotStore();
      expect(store.restoring).toBe(false);
      const promise = store.restore("snap-1");
      expect(store.restoring).toBe(true);
      resolveInvoke!(mockRestoreRecord);
      await promise;
      expect(store.restoring).toBe(false);
    });

    it("updates restoredAt on the snapshot after success", async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord);
      const store = useSnapshotStore();
      store.snapshots = [
        {
          id: "snap-1",
          profileId: "prof-1",
          databaseName: "mydb",
          name: "Test",
          note: null,
          filePath: "test/snap-1.sql.gz",
          sizeBytes: 1024,
          dbVersion: null,
          dumpToolVersion: null,
          checksum: null,
          restoreTestStatus: "not_configured",
          restoreTestMessage: null,
          restoreTestedAt: null,
          createdAt: "2026-01-01T00:00:00Z",
          restoredAt: null,
          pinned: false,
          tags: [],
        },
      ];
      await store.restore("snap-1");
      expect(store.snapshots[0].restoredAt).not.toBeNull();
    });

    it("propagates errors from invoke", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Restore failed"));
      const store = useSnapshotStore();
      await expect(store.restore("snap-1")).rejects.toThrow("Restore failed");
      expect(store.error).toBe("Error: Restore failed");
      expect(store.restoring).toBe(false);
    });
  });
});
