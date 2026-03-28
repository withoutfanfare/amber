<script setup lang="ts">
  import { ref, computed, onMounted, watch } from "vue";
  import { useRouter } from "vue-router";
  import { useProfileStore } from "@/stores/profiles";
  import { useSnapshotStore } from "@/stores/snapshots";
  import { useToastStack } from "@stuntrocket/ui";
  import { SButton, SFormField, SInput, SSelect, STextarea } from "@stuntrocket/ui";
  import PageHeader from "@/components/layout/PageHeader.vue";
  import type { SizeEstimation, DiskSpaceInfo } from "@/types";

  const router = useRouter();
  const profileStore = useProfileStore();
  const snapshotStore = useSnapshotStore();
  const toast = useToastStack();

  const selectedProfileId = ref("");
  const name = ref("");
  const note = ref("");
  const tagsInput = ref("");
  const sizeEstimation = ref<SizeEstimation | null>(null);
  const estimating = ref(false);
  const diskSpace = ref<DiskSpaceInfo | null>(null);
  const operationBusy = ref(false);

  const profileOptions = computed(() =>
    profileStore.profiles.map((p) => ({
      value: p.id,
      label: `${p.project} / ${p.name} (${p.dbType})`,
    })),
  );

  onMounted(async () => {
    if (profileStore.profiles.length === 0) {
      await profileStore.fetchAll();
    }
    selectedProfileId.value = profileStore.activeProfileId ?? profileStore.profiles[0]?.id ?? "";
    snapshotStore.fetchAllTags();
  });

  async function runEstimation(profileId: string) {
    estimating.value = true;
    sizeEstimation.value = null;
    diskSpace.value = null;
    operationBusy.value = false;
    try {
      // Check if profile is busy
      const opStatus = await profileStore.checkOperationStatus(profileId);
      operationBusy.value = opStatus.busy;

      sizeEstimation.value = await snapshotStore.estimateSize(profileId);
      // Run disk space check with estimated compressed size
      if (sizeEstimation.value) {
        try {
          diskSpace.value = await profileStore.checkDiskSpace(
            sizeEstimation.value.estimatedCompressedBytes,
          );
        } catch {
          diskSpace.value = null;
        }
      }
    } catch {
      sizeEstimation.value = null;
    } finally {
      estimating.value = false;
    }
  }

  // Estimate size when profile changes
  watch(
    selectedProfileId,
    async (newId) => {
      if (!newId) {
        sizeEstimation.value = null;
        diskSpace.value = null;
        return;
      }
      await runEstimation(newId);
    },
    { immediate: false },
  );

  // Trigger initial estimation
  onMounted(async () => {
    if (selectedProfileId.value) {
      await runEstimation(selectedProfileId.value);
    }
  });

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  async function handleSubmit() {
    if (!selectedProfileId.value || !name.value.trim()) return;

    const tags = tagsInput.value
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);

    try {
      await snapshotStore.create({
        profileId: selectedProfileId.value,
        name: name.value.trim(),
        note: note.value.trim() || undefined,
        tags: tags.length > 0 ? tags : undefined,
      });
      toast.success("Snapshot created successfully.");
      router.push("/snapshots");
    } catch (e) {
      toast.error(`Failed to create snapshot: ${e}`);
    }
  }
</script>

<template>
  <div>
    <PageHeader>
      <template #prepend>
        <SButton variant="ghost" size="sm" to="/snapshots">
          <svg
            class="h-4 w-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="m15 18-6-6 6-6" />
          </svg>
          Back
        </SButton>
        <h1 class="text-lg font-semibold">Create Snapshot</h1>
      </template>
    </PageHeader>

    <form class="max-w-xl space-y-5" @submit.prevent="handleSubmit">
      <SFormField label="Profile">
        <SSelect v-model="selectedProfileId">
          <option v-for="opt in profileOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </SSelect>
      </SFormField>

      <!-- Size estimation -->
      <div
        v-if="estimating"
        class="flex items-center gap-2 rounded-lg bg-surface-raised px-3 py-2 text-sm text-text-secondary"
      >
        <svg
          class="h-4 w-4 animate-spin"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M21 12a9 9 0 1 1-6.219-8.56" />
        </svg>
        Estimating snapshot size&hellip;
      </div>
      <div v-else-if="sizeEstimation" class="rounded-lg bg-surface-raised px-3 py-2">
        <div class="flex items-center justify-between text-sm">
          <span class="text-text-secondary">Estimated snapshot size:</span>
          <span class="font-mono font-medium text-text-primary">
            ~{{ formatSize(sizeEstimation.estimatedCompressedBytes) }}
          </span>
        </div>
        <p class="mt-1 text-xs text-text-tertiary">
          Raw database: {{ formatSize(sizeEstimation.estimatedRawBytes) }} &middot; Compression: ~{{
            Math.round((1 - sizeEstimation.compressionRatio) * 100)
          }}%
          <span v-if="sizeEstimation.estimationMethod === 'historical'"
            >(based on previous snapshots)</span
          >
          <span v-else>(estimated)</span>
        </p>
      </div>

      <!-- Disk space warning -->
      <div
        v-if="diskSpace && !diskSpace.sufficient"
        class="flex items-start gap-2 rounded-lg border border-danger/30 bg-danger/5 px-3 py-2 text-sm"
      >
        <svg
          class="mt-0.5 h-4 w-4 shrink-0 text-danger"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
        <div>
          <p class="font-medium text-danger">Insufficient disk space</p>
          <p class="text-xs text-text-secondary">{{ diskSpace.message }}</p>
        </div>
      </div>
      <div
        v-else-if="diskSpace && diskSpace.sufficient"
        class="flex items-center gap-2 rounded-lg bg-success/5 px-3 py-2 text-xs text-success"
      >
        <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20 6 9 17l-5-5" />
        </svg>
        {{ formatSize(diskSpace.availableBytes) }} available
      </div>

      <!-- Operation in progress warning -->
      <div
        v-if="operationBusy"
        class="flex items-start gap-2 rounded-lg border border-warning/30 bg-warning/5 px-3 py-2 text-sm"
      >
        <svg
          class="mt-0.5 h-4 w-4 shrink-0 animate-spin text-warning"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M21 12a9 9 0 1 1-6.219-8.56" />
        </svg>
        <div>
          <p class="font-medium text-warning">Operation in progress</p>
          <p class="text-xs text-text-secondary">
            Another snapshot or restore operation is running on this profile. Please wait for it to
            finish before creating a new snapshot.
          </p>
        </div>
      </div>

      <SFormField label="Snapshot Name">
        <SInput v-model="name" placeholder="before-migration-v2" />
      </SFormField>

      <SFormField label="Note">
        <STextarea
          v-model="note"
          placeholder="Optional description of what this snapshot captures"
          :rows="3"
        />
      </SFormField>

      <SFormField label="Tags">
        <SInput v-model="tagsInput" placeholder="pre-migration, clean-state (comma-separated)" />
        <div v-if="snapshotStore.allTags.length > 0" class="mt-1.5 flex flex-wrap gap-1">
          <span class="text-xs text-text-tertiary">Recent:</span>
          <button
            v-for="tag in snapshotStore.allTags.slice(0, 10)"
            :key="tag"
            type="button"
            class="rounded-full bg-accent/10 px-2 py-0.5 text-[10px] text-accent transition-colors hover:bg-accent/20"
            @click="tagsInput = tagsInput ? `${tagsInput}, ${tag}` : tag"
          >
            {{ tag }}
          </button>
        </div>
      </SFormField>

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
        <p v-if="snapshotStore.progress.currentTable" class="text-xs text-text-tertiary">
          Current table: {{ snapshotStore.progress.currentTable }}
        </p>
      </div>

      <!-- Sticky footer -->
      <div
        class="sticky bottom-0 -mx-6 mt-6 flex items-center justify-end gap-3 border-t border-border-subtle bg-surface-base/80 px-6 py-4 backdrop-blur-sm"
      >
        <SButton variant="ghost" size="md" @click="router.push('/snapshots')"> Cancel </SButton>
        <SButton
          variant="primary"
          size="md"
          :loading="snapshotStore.creating"
          :disabled="!selectedProfileId || !name.trim() || (diskSpace && !diskSpace.sufficient) || operationBusy"
        >
          Create Snapshot
        </SButton>
      </div>
    </form>
  </div>
</template>
