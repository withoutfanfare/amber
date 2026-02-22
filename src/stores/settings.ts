import { ref } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useSettingsStore = defineStore('settings', () => {
  // --- State ---
  const settings = ref<Map<string, string>>(new Map())
  const loading = ref(false)

  // --- Convenience Getters ---
  function get(key: string, defaultValue: string = ''): string {
    return settings.value.get(key) ?? defaultValue
  }

  // --- Actions ---
  async function fetchAll() {
    loading.value = true
    try {
      const entries = await invoke<Record<string, string>>('settings_list')
      settings.value = new Map(Object.entries(entries))
    } finally {
      loading.value = false
    }
  }

  async function set(key: string, value: string) {
    await invoke('settings_set', { key, value })
    settings.value.set(key, value)
  }

  return {
    settings, loading,
    get, fetchAll, set,
  }
})
