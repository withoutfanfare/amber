<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useProfileStore } from '@/stores/profiles'
import { useToast } from '@/composables/useToast'
import PageHeader from '@/components/layout/PageHeader.vue'
import Button from '@/components/ui/Button.vue'
import ProfileForm from '@/components/features/ProfileForm.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import type { ProfileCreatePayload } from '@/types'

const props = defineProps<{
  id: string
}>()

const router = useRouter()
const profileStore = useProfileStore()
const toast = useToast()
const submitting = ref(false)
const deleteDialogOpen = ref(false)

onMounted(async () => {
  // Ensure profiles are loaded so we can find the one to edit
  if (profileStore.profiles.length === 0) {
    await profileStore.fetchAll()
  }
})

const profile = computed(() =>
  profileStore.profiles.find(p => p.id === props.id) ?? null
)

async function handleSubmit(payload: ProfileCreatePayload) {
  submitting.value = true
  try {
    await profileStore.update(props.id, payload)
    toast.success('Profile updated successfully.')
    router.push('/profiles')
  } catch (e) {
    toast.error(`Failed to update profile: ${e}`)
  } finally {
    submitting.value = false
  }
}

async function handleDelete() {
  try {
    await profileStore.remove(props.id)
    toast.success(`Profile "${profile.value?.name ?? ''}" deleted.`)
    router.push('/profiles')
  } catch (e) {
    toast.error(`Failed to delete profile: ${e}`)
  } finally {
    deleteDialogOpen.value = false
  }
}
</script>

<template>
  <div>
    <PageHeader>
      <template #prepend>
        <Button variant="ghost" size="sm" to="/profiles">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m15 18-6-6 6-6" />
          </svg>
          Back
        </Button>
        <h1 class="text-lg font-semibold">Edit Profile</h1>
      </template>
      <template #actions>
        <Button variant="ghost" size="sm" class="text-danger hover:text-danger" @click="deleteDialogOpen = true">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
          </svg>
          Delete
        </Button>
      </template>
    </PageHeader>

    <div v-if="!profile" class="py-8 text-center text-sm text-text-secondary">
      Loading profile&hellip;
    </div>

    <ProfileForm
      v-else
      :initial-data="profile"
      :submitting="submitting"
      @submit="handleSubmit"
      @cancel="router.push('/profiles')"
    />

    <ConfirmDialog
      :open="deleteDialogOpen"
      title="Delete Profile"
      :message="`Are you sure you want to delete &quot;${profile?.name ?? ''}&quot;? This will also remove all associated snapshots. This action cannot be undone.`"
      confirm-label="Delete"
      :danger="true"
      @confirm="handleDelete"
      @cancel="deleteDialogOpen = false"
    />
  </div>
</template>
