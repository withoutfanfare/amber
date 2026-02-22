import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Profile, ProfileCreatePayload, ProfileUpdateInput, ConnectionTestResult } from '@/types'

export const useProfileStore = defineStore('profiles', () => {
  // --- State ---
  const profiles = ref<Profile[]>([])
  const activeProfileId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const testResults = ref<Map<string, ConnectionTestResult>>(new Map())

  // --- Getters ---
  const activeProfile = computed(() =>
    profiles.value.find(p => p.id === activeProfileId.value) ?? null
  )

  const profilesByProject = computed(() => {
    const grouped = new Map<string, Profile[]>()
    for (const profile of profiles.value) {
      const list = grouped.get(profile.project) ?? []
      list.push(profile)
      grouped.set(profile.project, list)
    }
    return grouped
  })

  // --- Actions ---
  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      profiles.value = await invoke<Profile[]>('profile_list')
      // Auto-select first profile if none active
      if (!activeProfileId.value && profiles.value.length > 0) {
        activeProfileId.value = profiles.value[0].id
      }
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function create(payload: ProfileCreatePayload): Promise<Profile> {
    const profile = await invoke<Profile>('profile_create', { input: payload })
    profiles.value.push(profile)
    activeProfileId.value = profile.id
    return profile
  }

  async function update(id: string, input: ProfileUpdateInput): Promise<Profile> {
    const updated = await invoke<Profile>('profile_update', { id, input })
    const index = profiles.value.findIndex(p => p.id === updated.id)
    if (index >= 0) profiles.value[index] = updated
    return updated
  }

  async function remove(id: string): Promise<void> {
    await invoke('profile_delete', { id })
    profiles.value = profiles.value.filter(p => p.id !== id)
    if (activeProfileId.value === id) {
      activeProfileId.value = profiles.value[0]?.id ?? null
    }
  }

  async function testConnection(id: string): Promise<ConnectionTestResult> {
    const result = await invoke<ConnectionTestResult>('profile_test_connection', { id })
    testResults.value.set(id, result)
    return result
  }

  function setActive(id: string) {
    activeProfileId.value = id
  }

  return {
    profiles, activeProfileId, loading, error, testResults,
    activeProfile, profilesByProject,
    fetchAll, create, update, remove, testConnection, setActive,
  }
})
