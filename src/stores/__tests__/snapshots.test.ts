import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { useSnapshotStore } from '../snapshots'
import type { RestoreRecord } from '@/types'

const mockInvoke = vi.mocked(invoke)

const mockRestoreRecord: RestoreRecord = {
  id: 'rec-1',
  snapshotId: 'snap-1',
  snapshotName: 'Test',
  targetProfileId: 'prof-1',
  targetDbName: 'mydb',
  durationSecs: 0.5,
  restoredAt: '2026-01-01T00:00:00Z',
}

describe('useSnapshotStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('restore', () => {
    it('calls invoke with snapshotId and null options when no options passed', async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord)
      const store = useSnapshotStore()
      await store.restore('snap-1')
      expect(mockInvoke).toHaveBeenCalledWith('snapshot_restore', {
        snapshotId: 'snap-1',
        options: null,
        onProgress: expect.anything(),
      })
    })

    it('passes targetProfileId in options when provided', async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord)
      const store = useSnapshotStore()
      await store.restore('snap-1', { targetProfileId: 'profile-2' })
      expect(mockInvoke).toHaveBeenCalledWith('snapshot_restore', {
        snapshotId: 'snap-1',
        options: { targetProfileId: 'profile-2' },
        onProgress: expect.anything(),
      })
    })

    it('passes targetDatabaseName in options when provided', async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord)
      const store = useSnapshotStore()
      await store.restore('snap-1', { targetDatabaseName: 'new_db' })
      expect(mockInvoke).toHaveBeenCalledWith('snapshot_restore', {
        snapshotId: 'snap-1',
        options: { targetDatabaseName: 'new_db' },
        onProgress: expect.anything(),
      })
    })

    it('passes both overrides together', async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord)
      const store = useSnapshotStore()
      await store.restore('snap-1', {
        targetProfileId: 'profile-2',
        targetDatabaseName: 'new_db',
      })
      expect(mockInvoke).toHaveBeenCalledWith('snapshot_restore', {
        snapshotId: 'snap-1',
        options: { targetProfileId: 'profile-2', targetDatabaseName: 'new_db' },
        onProgress: expect.anything(),
      })
    })

    it('sets restoring flag during operation', async () => {
      let resolveInvoke: (value: unknown) => void
      mockInvoke.mockReturnValueOnce(
        new Promise((resolve) => { resolveInvoke = resolve })
      )
      const store = useSnapshotStore()
      expect(store.restoring).toBe(false)
      const promise = store.restore('snap-1')
      expect(store.restoring).toBe(true)
      resolveInvoke!(mockRestoreRecord)
      await promise
      expect(store.restoring).toBe(false)
    })

    it('updates restoredAt on the snapshot after success', async () => {
      mockInvoke.mockResolvedValueOnce(mockRestoreRecord)
      const store = useSnapshotStore()
      store.snapshots = [{
        id: 'snap-1',
        profileId: 'prof-1',
        name: 'Test',
        note: null,
        filePath: 'test/snap-1.sql.gz',
        sizeBytes: 1024,
        dbVersion: null,
        dumpToolVersion: null,
        createdAt: '2026-01-01T00:00:00Z',
        restoredAt: null,
      }]
      await store.restore('snap-1')
      expect(store.snapshots[0].restoredAt).not.toBeNull()
    })

    it('propagates errors from invoke', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Restore failed'))
      const store = useSnapshotStore()
      await expect(store.restore('snap-1')).rejects.toThrow('Restore failed')
      expect(store.error).toBe('Error: Restore failed')
      expect(store.restoring).toBe(false)
    })
  })
})
