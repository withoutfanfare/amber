<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useProfileStore } from '@/stores/profiles'
import { useToast } from '@/composables/useToast'
import PageHeader from '@/components/layout/PageHeader.vue'
import Button from '@/components/ui/Button.vue'
import ProfileForm from '@/components/features/ProfileForm.vue'
import type { ProfileCreatePayload } from '@/types'

const router = useRouter()
const profileStore = useProfileStore()
const toast = useToast()
const submitting = ref(false)

async function handleSubmit(payload: ProfileCreatePayload) {
  submitting.value = true
  try {
    await profileStore.create(payload)
    toast.success('Profile created successfully.')
    router.push('/profiles')
  } catch (e) {
    toast.error(`Failed to create profile: ${e}`)
  } finally {
    submitting.value = false
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
        <h1 class="text-lg font-semibold">New Profile</h1>
      </template>
    </PageHeader>

    <ProfileForm
      :submitting="submitting"
      @submit="handleSubmit"
      @cancel="router.push('/profiles')"
    />
  </div>
</template>
