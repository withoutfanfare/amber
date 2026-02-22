import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { useRestoreHistoryStore } from '../restoreHistory'
import type { RestoreRecord } from '@/types'

const mockInvoke = vi.mocked(invoke)

const mockRecord: RestoreRecord = {
  id: 'rec-1',
  snapshotId: 'snap-1',
  snapshotName: 'My Snapshot',
  targetProfileId: 'prof-1',
  targetDbName: 'mydb',
  durationSecs: 1.23,
  restoredAt: '2026-02-01T12:00:00Z',
}

function makeRecords(count: number): RestoreRecord[] {
  return Array.from({ length: count }, (_, i) => ({
    ...mockRecord,
    id: `rec-${i}`,
    restoredAt: `2026-02-${String(count - i).padStart(2, '0')}T00:00:00Z`,
  }))
}

describe('useRestoreHistoryStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('fetchAll', () => {
    it('fetches restore history from backend', async () => {
      const records = [mockRecord]
      mockInvoke.mockResolvedValueOnce(records)
      const store = useRestoreHistoryStore()
      await store.fetchAll()
      expect(mockInvoke).toHaveBeenCalledWith('restore_history_list', { limit: null })
      expect(store.records).toEqual(records)
      expect(store.loading).toBe(false)
    })

    it('passes limit parameter when provided', async () => {
      mockInvoke.mockResolvedValueOnce([])
      const store = useRestoreHistoryStore()
      await store.fetchAll(10)
      expect(mockInvoke).toHaveBeenCalledWith('restore_history_list', { limit: 10 })
    })

    it('sets error on fetch failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))
      const store = useRestoreHistoryStore()
      await store.fetchAll()
      expect(store.error).toBe('Error: Network error')
      expect(store.records).toEqual([])
      expect(store.loading).toBe(false)
    })
  })

  describe('recentRestores', () => {
    it('returns first 5 records', () => {
      const store = useRestoreHistoryStore()
      store.records = makeRecords(8)
      expect(store.recentRestores).toHaveLength(5)
      expect(store.recentRestores[0].id).toBe('rec-0')
      expect(store.recentRestores[4].id).toBe('rec-4')
    })

    it('returns all records when fewer than 5', () => {
      const store = useRestoreHistoryStore()
      store.records = makeRecords(3)
      expect(store.recentRestores).toHaveLength(3)
    })
  })

  describe('lastRestore', () => {
    it('returns first record', () => {
      const store = useRestoreHistoryStore()
      store.records = [mockRecord]
      expect(store.lastRestore).toEqual(mockRecord)
    })

    it('returns null when empty', () => {
      const store = useRestoreHistoryStore()
      expect(store.lastRestore).toBeNull()
    })
  })

  describe('addLocal', () => {
    it('prepends a record', () => {
      const store = useRestoreHistoryStore()
      store.records = [mockRecord]
      const newRecord: RestoreRecord = { ...mockRecord, id: 'rec-new' }
      store.addLocal(newRecord)
      expect(store.records).toHaveLength(2)
      expect(store.records[0].id).toBe('rec-new')
      expect(store.records[1].id).toBe('rec-1')
    })
  })
})
