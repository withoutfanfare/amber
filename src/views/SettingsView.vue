<script setup lang="ts">
  import { ref, computed, onMounted, watch } from "vue";
  import { useRouter } from "vue-router";
  import { invoke } from "@tauri-apps/api/core";
  import { useSettingsStore } from "@/stores/settings";
  import { useProfileStore } from "@/stores/profiles";
  import { useToastStack, SCard, SInput, SButton, SSelect, SFormField } from "@stuntrocket/ui";
  import PageHeader from "@/components/layout/PageHeader.vue";
  import type { RetentionPolicy, RetentionEnforcementResult } from "@/types";

  const router = useRouter();
  const settingsStore = useSettingsStore();
  const profileStore = useProfileStore();
  const toast = useToastStack();

  const autoPruneEnabled = ref(false);
  const autoPruneDays = ref(30);

  const selectedRetentionProfile = ref("");
  const retentionMaxCount = ref<number | undefined>(undefined);
  const retentionMaxAgeDays = ref<number | undefined>(undefined);
  const retentionMaxSizeMb = ref<number | undefined>(undefined);
  const retentionSaving = ref(false);
  const retentionEnforcing = ref(false);

  onMounted(async () => {
    await settingsStore.fetchAll();
    if (profileStore.profiles.length === 0) {
      await profileStore.fetchAll();
    }
    autoPruneEnabled.value = settingsStore.get("auto_prune_enabled", "false") === "true";
    autoPruneDays.value = parseInt(settingsStore.get("auto_prune_days", "30"), 10) || 30;
  });

  const snapshotDir = computed(() =>
    settingsStore.get(
      "snapshot_directory",
      "~/Library/Application Support/com.dannyharding.amber/snapshots",
    ),
  );

  const profileOptions = computed(() =>
    profileStore.profiles.map((p) => ({
      value: p.id,
      label: `${p.project} / ${p.name}`,
    })),
  );

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  function debouncedSave(key: string, value: string) {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      try {
        await settingsStore.set(key, value);
      } catch (e) {
        toast.error(`Failed to save setting: ${e}`);
      }
    }, 500);
  }

  watch(autoPruneEnabled, (val) => {
    debouncedSave("auto_prune_enabled", String(val));
  });

  watch(autoPruneDays, (val) => {
    debouncedSave("auto_prune_days", String(val));
  });

  watch(selectedRetentionProfile, async (profileId) => {
    if (!profileId) return;
    try {
      const policy = await invoke<RetentionPolicy | null>("retention_policy_get", { profileId });
      if (policy) {
        retentionMaxCount.value = policy.maxCount ?? undefined;
        retentionMaxAgeDays.value = policy.maxAgeDays ?? undefined;
        retentionMaxSizeMb.value = policy.maxSizeBytes
          ? Math.round(policy.maxSizeBytes / (1024 * 1024))
          : undefined;
      } else {
        retentionMaxCount.value = undefined;
        retentionMaxAgeDays.value = undefined;
        retentionMaxSizeMb.value = undefined;
      }
    } catch {
      // Non-critical
    }
  });

  async function saveRetentionPolicy() {
    if (!selectedRetentionProfile.value) return;
    retentionSaving.value = true;
    try {
      await invoke("retention_policy_set", {
        policy: {
          profileId: selectedRetentionProfile.value,
          maxCount: retentionMaxCount.value ?? null,
          maxAgeDays: retentionMaxAgeDays.value ?? null,
          maxSizeBytes: retentionMaxSizeMb.value ? retentionMaxSizeMb.value * 1024 * 1024 : null,
        },
      });
      toast.success("Retention policy saved.");
    } catch (e) {
      toast.error(`Failed to save retention policy: ${e}`);
    } finally {
      retentionSaving.value = false;
    }
  }

  async function enforceRetention() {
    if (!selectedRetentionProfile.value) return;
    retentionEnforcing.value = true;
    try {
      const result = await invoke<RetentionEnforcementResult>("retention_enforce", {
        profileId: selectedRetentionProfile.value,
      });
      if (result.deletedCount > 0) {
        toast.success(`Cleaned up ${result.deletedCount} snapshot(s). ${result.reasons.join(" ")}`);
      } else {
        toast.info("No snapshots needed cleanup.");
      }
    } catch (e) {
      toast.error(`Retention enforcement failed: ${e}`);
    } finally {
      retentionEnforcing.value = false;
    }
  }
</script>

<template>
  <div>
    <PageHeader>
      <template #prepend>
        <h1 class="text-lg font-semibold">Settings</h1>
      </template>
    </PageHeader>

    <div v-if="settingsStore.loading" class="py-8 text-center text-sm text-text-secondary">
      Loading settings&hellip;
    </div>

    <div v-else class="space-y-6">
      <!-- Top row: Snapshot directory + Auto-prune -->
      <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <SCard>
          <h3 class="mb-3 text-sm font-semibold text-text-primary">Snapshot Directory</h3>
          <p class="mb-1 text-sm text-text-secondary">Snapshots are stored at:</p>
          <p
            class="rounded-lg border border-border-subtle bg-surface-raised px-3 py-2 font-mono text-sm text-text-primary"
          >
            {{ snapshotDir }}
          </p>
        </SCard>

        <SCard>
          <h3 class="mb-3 text-sm font-semibold text-text-primary">Auto-Prune</h3>
          <div class="space-y-3">
            <label class="flex cursor-pointer items-center gap-2">
              <input
                v-model="autoPruneEnabled"
                type="checkbox"
                class="h-4 w-4 rounded border-border accent-accent"
              />
              <span class="text-sm text-text-secondary">Automatically delete old snapshots</span>
            </label>

            <div v-if="autoPruneEnabled" class="flex items-center gap-2 pl-6">
              <span class="text-sm text-text-secondary">Delete snapshots older than</span>
              <div class="w-20">
                <SInput v-model="autoPruneDays" type="number" />
              </div>
              <span class="text-sm text-text-secondary">days</span>
            </div>
          </div>
        </SCard>
      </div>

      <!-- Retention policies — full width -->
      <SCard>
        <h3 class="mb-3 text-sm font-semibold text-text-primary">Retention Policies</h3>
        <p class="mb-4 text-sm text-text-secondary">
          Configure per-profile rules to automatically manage snapshot storage. Pinned snapshots are
          protected from auto-cleanup.
        </p>

        <SFormField label="Profile" class="mb-4 max-w-sm">
          <SSelect v-model="selectedRetentionProfile">
            <option value="" disabled>Select a profile&hellip;</option>
            <option v-for="opt in profileOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </SSelect>
        </SFormField>

        <div v-if="selectedRetentionProfile" class="space-y-3">
          <div class="flex items-center gap-2">
            <span class="w-40 text-sm text-text-secondary">Keep last</span>
            <div class="w-20">
              <SInput v-model="retentionMaxCount" type="number" placeholder="N" />
            </div>
            <span class="text-sm text-text-secondary">snapshots</span>
          </div>

          <div class="flex items-center gap-2">
            <span class="w-40 text-sm text-text-secondary">Max age</span>
            <div class="w-20">
              <SInput v-model="retentionMaxAgeDays" type="number" placeholder="N" />
            </div>
            <span class="text-sm text-text-secondary">days</span>
          </div>

          <div class="flex items-center gap-2">
            <span class="w-40 text-sm text-text-secondary">Max total size</span>
            <div class="w-20">
              <SInput v-model="retentionMaxSizeMb" type="number" placeholder="N" />
            </div>
            <span class="text-sm text-text-secondary">MB</span>
          </div>

          <div class="flex items-center gap-3 pt-2">
            <SButton
              variant="primary"
              size="sm"
              :loading="retentionSaving"
              @click="saveRetentionPolicy"
            >
              Save Policy
            </SButton>
            <SButton
              variant="secondary"
              size="sm"
              :loading="retentionEnforcing"
              @click="enforceRetention"
            >
              Enforce Now
            </SButton>
          </div>
        </div>
      </SCard>

      <!-- Bottom row: Compression + Database Tools -->
      <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <SCard>
          <h3 class="mb-3 text-sm font-semibold text-text-primary">Compression</h3>
          <p class="text-sm text-text-secondary">
            Compression level: <span class="font-medium text-text-primary">Default (gzip)</span>
          </p>
        </SCard>

        <SCard>
          <h3 class="mb-3 text-sm font-semibold text-text-primary">Database Tools</h3>
          <p class="mb-3 text-sm text-text-secondary">
            Manage database tool paths and verify tool availability.
          </p>
          <SButton variant="secondary" size="sm" @click="router.push('/setup')">
            Run Setup Wizard
          </SButton>
        </SCard>
      </div>
    </div>
  </div>
</template>
