import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { RestoreRecord } from '@/types'

export const useRestoreHistoryStore = defineStore('restoreHistory', () => {
  // --- State ---
  const records = ref<RestoreRecord[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // --- Getters ---
  const recentRestores = computed(() => records.value.slice(0, 5))
  const lastRestore = computed(() => records.value[0] ?? null)

  // --- Actions ---
  async function fetchAll(limit?: number) {
    loading.value = true
    error.value = null
    try {
      records.value = await invoke<RestoreRecord[]>('restore_history_list', {
        limit: limit ?? null,
      })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  function addLocal(record: RestoreRecord) {
    records.value.unshift(record)
  }

  return {
    records, loading, error,
    recentRestores, lastRestore,
    fetchAll, addLocal,
  }
})
