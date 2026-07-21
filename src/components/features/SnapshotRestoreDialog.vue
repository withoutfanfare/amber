<script setup lang="ts">
  import { ref, computed, watch } from "vue";
  import { SConfirmDialog, SFormField, SSelect, SInput } from "@stuntrocket/ui";
  import type { Snapshot, Profile, SnapshotRestoreOptions } from "@/types";

  const props = defineProps<{
    open: boolean;
    snapshot: Snapshot | null;
    profiles: Profile[];
    sourceProfile: Profile | null;
  }>();

  const emit = defineEmits<{
    confirm: [options: SnapshotRestoreOptions];
    cancel: [];
  }>();

  const selectedProfileId = ref<string>("");
  const overrideDbName = ref(false);
  const customDbName = ref("");

  // Filter profiles to only show those with matching dbType
  const compatibleProfiles = computed(() => {
    if (!props.sourceProfile) return [];
    return props.profiles.filter((p) => p.dbType === props.sourceProfile!.dbType);
  });

  const profileOptions = computed(() =>
    compatibleProfiles.value.map((p) => ({
      value: p.id,
      label: `${p.project} / ${p.name}`,
    })),
  );

  const selectedProfile = computed(
    () => props.profiles.find((p) => p.id === selectedProfileId.value) ?? null,
  );

  const effectiveDbName = computed(() => {
    if (overrideDbName.value && customDbName.value.trim()) {
      return customDbName.value.trim();
    }
    return props.snapshot?.databaseName ?? selectedProfile.value?.databaseName ?? "";
  });

  const isOverridingProfile = computed(
    () => props.sourceProfile != null && selectedProfileId.value !== props.sourceProfile.id,
  );

  // Reset state when dialog opens
  watch(
    () => props.open,
    (isOpen) => {
      if (isOpen && props.sourceProfile) {
        selectedProfileId.value = props.sourceProfile.id;
        overrideDbName.value = false;
        customDbName.value =
          props.snapshot?.databaseName ?? selectedProfile.value?.databaseName ?? "";
      }
    },
    { immediate: true },
  );

  // Update customDbName when profile changes
  watch(selectedProfileId, () => {
    if (!overrideDbName.value) {
      customDbName.value = selectedProfile.value?.databaseName ?? "";
    }
  });

  function handleConfirm() {
    const options: SnapshotRestoreOptions = {};
    if (isOverridingProfile.value) {
      options.targetProfileId = selectedProfileId.value;
    }
    if (overrideDbName.value && customDbName.value.trim()) {
      options.targetDatabaseName = customDbName.value.trim();
    }
    emit("confirm", options);
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString("en-GB", {
      day: "numeric",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
</script>

<template>
  <SConfirmDialog
    :open="open"
    title="Restore Snapshot"
    message="Choose where to restore this snapshot. This will overwrite the target database."
    confirm-label="Restore"
    :danger="true"
    @confirm="handleConfirm"
    @cancel="$emit('cancel')"
    @close="$emit('cancel')"
  >
    <template v-if="snapshot" #default>
      <!-- Snapshot metadata -->
      <div class="card-inset mb-4 rounded-lg">
        <dl class="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
          <dt class="text-text-tertiary">Name</dt>
          <dd class="font-medium text-text-primary">{{ snapshot.name }}</dd>
          <dt class="text-text-tertiary">Created</dt>
          <dd class="text-text-secondary">{{ formatDate(snapshot.createdAt) }}</dd>
          <dt class="text-text-tertiary">Size</dt>
          <dd class="font-mono text-text-secondary">{{ formatSize(snapshot.sizeBytes) }}</dd>
          <dt class="text-text-tertiary">Source</dt>
          <dd class="text-text-secondary">
            {{ sourceProfile ? `${sourceProfile.project} / ${sourceProfile.name}` : "Unknown" }}
          </dd>
          <dt class="text-text-tertiary">Database</dt>
          <dd class="font-mono text-text-secondary">
            {{ snapshot.databaseName ?? sourceProfile?.databaseName ?? "Unknown" }}
          </dd>
        </dl>
      </div>

      <!-- Profile selector -->
      <div v-if="profileOptions.length > 1" class="mb-4">
        <SFormField label="Restore to profile">
          <SSelect v-model="selectedProfileId">
            <option v-for="opt in profileOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </SSelect>
        </SFormField>
      </div>

      <!-- Database name override -->
      <div class="mb-4">
        <label class="mb-2 flex cursor-pointer items-center gap-2 text-sm text-text-secondary">
          <input v-model="overrideDbName" type="checkbox" class="rounded border-border" />
          Override database name
        </label>
        <SFormField v-if="overrideDbName" label="Target database name">
          <SInput v-model="customDbName" placeholder="Enter database name" />
        </SFormField>
      </div>

      <!-- Warning banner -->
      <div class="mb-4 rounded-lg border border-warning/30 bg-warning/10 px-3 py-2 text-sm">
        <p class="mb-1 font-medium text-text-primary">This will overwrite:</p>
        <p class="text-text-secondary">
          <span class="font-mono text-xs">{{ selectedProfile?.name ?? "Unknown" }}</span>
          &rarr;
          <span class="font-mono text-xs">{{ effectiveDbName }}</span>
        </p>
        <p v-if="isOverridingProfile" class="mt-1 text-xs text-warning">
          Restoring to a different profile than the original source.
        </p>
      </div>
    </template>
  </SConfirmDialog>
</template>
