<script setup lang="ts">
  import { ref, onMounted, onUnmounted } from "vue";
  import { useRouter } from "vue-router";
  import { useProfileStore } from "@/stores/profiles";
  import { useToastStack } from "@stuntrocket/ui";
  import PageHeader from "@/components/layout/PageHeader.vue";
  import { SButton, SEmptyState, SConfirmDialog } from "@stuntrocket/ui";
  import ProfileCard from "@/components/features/ProfileCard.vue";
  import type { Profile } from "@/types";

  const router = useRouter();
  const profileStore = useProfileStore();
  const toast = useToastStack();

  const deleteDialogOpen = ref(false);
  const profileToDelete = ref<Profile | null>(null);
  let healthInterval: ReturnType<typeof setInterval> | null = null;

  async function runHealthChecks() {
    for (const profile of profileStore.profiles) {
      try {
        await profileStore.healthCheck(profile.id);
      } catch {
        // Non-critical — health check failure is handled in the store
      }
    }
  }

  onMounted(async () => {
    await profileStore.fetchAll();
    // Run initial health checks, then repeat every 60 seconds
    runHealthChecks();
    healthInterval = setInterval(runHealthChecks, 60_000);
  });

  onUnmounted(() => {
    if (healthInterval) {
      clearInterval(healthInterval);
      healthInterval = null;
    }
  });

  function handleEdit(profile: Profile) {
    router.push(`/profiles/${profile.id}/edit`);
  }

  function handleDeleteRequest(profile: Profile) {
    profileToDelete.value = profile;
    deleteDialogOpen.value = true;
  }

  async function handleDeleteConfirm() {
    if (!profileToDelete.value) return;
    try {
      await profileStore.remove(profileToDelete.value.id);
      toast.success(`Profile "${profileToDelete.value.name}" deleted.`);
    } catch (e) {
      toast.error(`Failed to delete profile: ${e}`);
    } finally {
      deleteDialogOpen.value = false;
      profileToDelete.value = null;
    }
  }

  async function handleTest(profile: Profile) {
    try {
      await profileStore.testConnection(profile.id);
    } catch (e) {
      toast.error(`Connection test failed: ${e}`);
    }
  }
</script>

<template>
  <div>
    <PageHeader>
      <template #prepend>
        <h1 class="text-lg font-semibold">Profiles</h1>
        <p class="text-sm text-text-tertiary">
          Connection profiles store your database credentials and connection details.
        </p>
      </template>
      <template #actions>
        <SButton variant="primary" size="sm" to="/profiles/create">New Profile</SButton>
      </template>
    </PageHeader>

    <!-- Loading state -->
    <div v-if="profileStore.loading" class="py-8 text-center text-sm text-text-secondary">
      Loading profiles&hellip;
    </div>

    <!-- Empty state -->
    <SEmptyState
      v-else-if="profileStore.profiles.length === 0"
      title="No connection profiles yet. Create one to get started."
    >
      <template #action>
        <SButton variant="primary" @click="router.push('/profiles/create')">Create Profile</SButton>
      </template>
    </SEmptyState>

    <!-- Profiles grouped by project -->
    <div v-else class="stagger-fade-in space-y-8">
      <section v-for="[project, profiles] of profileStore.profilesByProject" :key="project">
        <h2 class="mb-3 text-sm font-semibold tracking-wide text-text-tertiary uppercase">
          {{ project }}
        </h2>
        <div class="grid gap-4 sm:grid-cols-1 lg:grid-cols-2">
          <ProfileCard
            v-for="profile in profiles"
            :key="profile.id"
            :profile="profile"
            :test-result="profileStore.testResults.get(profile.id)"
            :health-status="profileStore.healthStatuses.get(profile.id)"
            :operation-status="profileStore.operationStatuses.get(profile.id)"
            @edit="handleEdit(profile)"
            @delete="handleDeleteRequest(profile)"
            @test="handleTest(profile)"
          />
        </div>
      </section>
    </div>

    <!-- Delete confirmation dialog -->
    <SConfirmDialog
      :open="deleteDialogOpen"
      title="Delete Profile"
      :message="`Are you sure you want to delete the profile &quot;${profileToDelete?.name ?? ''}&quot;? This will also remove all associated snapshots. This action cannot be undone.`"
      confirm-label="Delete"
      :danger="true"
      @confirm="handleDeleteConfirm"
      @cancel="deleteDialogOpen = false"
      @close="deleteDialogOpen = false"
    />
  </div>
</template>
