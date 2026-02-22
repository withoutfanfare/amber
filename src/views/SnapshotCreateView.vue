<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useProfileStore } from '@/stores/profiles'
import { useSnapshotStore } from '@/stores/snapshots'
import { useToast } from '@/composables/useToast'
import PageHeader from '@/components/layout/PageHeader.vue'
import Button from '@/components/ui/Button.vue'
import FormInput from '@/components/ui/FormInput.vue'
import FormSelect from '@/components/ui/FormSelect.vue'
import FormTextarea from '@/components/ui/FormTextarea.vue'

const router = useRouter()
const profileStore = useProfileStore()
const snapshotStore = useSnapshotStore()
const toast = useToast()

const selectedProfileId = ref('')
const name = ref('')
const note = ref('')

const profileOptions = computed(() =>
  profileStore.profiles.map(p => ({
    value: p.id,
    label: `${p.project} / ${p.name} (${p.dbType})`,
  }))
)

onMounted(async () => {
  if (profileStore.profiles.length === 0) {
    await profileStore.fetchAll()
  }
  // Default to active profile
  selectedProfileId.value = profileStore.activeProfileId ?? profileStore.profiles[0]?.id ?? ''
})

async function handleSubmit() {
  if (!selectedProfileId.value || !name.value.trim()) return

  try {
    await snapshotStore.create({
      profileId: selectedProfileId.value,
      name: name.value.trim(),
      note: note.value.trim() || undefined,
    })
    toast.success('Snapshot created successfully.')
    router.push('/snapshots')
  } catch (e) {
    toast.error(`Failed to create snapshot: ${e}`)
  }
}
</script>

<template>
  <div>
    <PageHeader>
      <template #prepend>
        <Button variant="ghost" size="sm" to="/snapshots">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m15 18-6-6 6-6" />
          </svg>
          Back
        </Button>
        <h1 class="text-lg font-semibold">Create Snapshot</h1>
      </template>
    </PageHeader>

    <form class="max-w-xl space-y-5" @submit.prevent="handleSubmit">
      <FormSelect
        v-model="selectedProfileId"
        label="Profile"
        :options="profileOptions"
      />

      <FormInput
        v-model="name"
        label="Snapshot Name"
        placeholder="before-migration-v2"
      />

      <FormTextarea
        v-model="note"
        label="Note"
        placeholder="Optional description of what this snapshot captures"
        :rows="3"
      />

      <!-- Progress section -->
      <div v-if="snapshotStore.creating && snapshotStore.progress" class="space-y-2">
        <div class="h-2 w-full overflow-hidden rounded-full bg-surface-raised">
          <div
            class="h-full rounded-full bg-accent transition-all duration-300 ease-out"
            :style="{ width: `${snapshotStore.progress.percentage}%` }"
          />
        </div>
        <p class="text-xs text-text-secondary">
          {{ snapshotStore.progress.message }}
        </p>
      </div>

      <!-- Sticky footer -->
      <div class="sticky bottom-0 -mx-6 mt-6 flex items-center justify-end gap-3 border-t border-border-subtle bg-surface-base/80 px-6 py-4 backdrop-blur-sm">
        <Button variant="ghost" size="md" @click="router.push('/snapshots')">
          Cancel
        </Button>
        <Button
          variant="primary"
          size="md"
          type="submit"
          :loading="snapshotStore.creating"
          :disabled="!selectedProfileId || !name.trim()"
        >
          Create Snapshot
        </Button>
      </div>
    </form>
  </div>
</template>
