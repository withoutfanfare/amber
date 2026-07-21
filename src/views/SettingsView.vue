<script setup lang="ts">
  import { ref, computed, onMounted, watch } from "vue";
  import { useRouter } from "vue-router";
  import { invoke } from "@tauri-apps/api/core";
  import { useSettingsStore } from "@/stores/settings";
  import { useProfileStore } from "@/stores/profiles";
  import { useToastStack, SCard, SInput, SButton, SSelect, SFormField } from "@stuntrocket/ui";
  import PageHeader from "@/components/layout/PageHeader.vue";
  import type {
    RetentionPolicy,
    RetentionEnforcementResult,
    ScheduleConfig,
    ScheduleInterval,
    RestoreTestSettings,
  } from "@/types";

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

  const selectedScheduleProfile = ref("");
  const scheduleInterval = ref<ScheduleInterval>("disabled");
  const scheduleLastSnapshot = ref<string | null>(null);
  const scheduleNextDue = ref<string | null>(null);
  const scheduleSaving = ref(false);

  const restoreTestMysqlEnabled = ref(false);
  const restoreTestMysqlPort = ref(3306);
  const restoreTestMysqlUsername = ref("root");
  const restoreTestMysqlPassword = ref("");
  const restoreTestPostgresqlEnabled = ref(false);
  const restoreTestPostgresqlPort = ref(5432);
  const restoreTestPostgresqlUsername = ref("postgres");
  const restoreTestPostgresqlPassword = ref("");
  const restoreTestSaving = ref(false);

  onMounted(async () => {
    await settingsStore.fetchAll();
    if (profileStore.profiles.length === 0) {
      await profileStore.fetchAll();
    }
    autoPruneEnabled.value = settingsStore.get("auto_prune_enabled", "false") === "true";
    autoPruneDays.value = parseInt(settingsStore.get("auto_prune_days", "30"), 10) || 30;

    try {
      const restoreSettings = await invoke<RestoreTestSettings>("restore_test_settings_get");
      restoreTestMysqlEnabled.value = restoreSettings.mysql.enabled;
      restoreTestMysqlPort.value = restoreSettings.mysql.port;
      restoreTestMysqlUsername.value = restoreSettings.mysql.username;
      restoreTestPostgresqlEnabled.value = restoreSettings.postgresql.enabled;
      restoreTestPostgresqlPort.value = restoreSettings.postgresql.port;
      restoreTestPostgresqlUsername.value = restoreSettings.postgresql.username;
    } catch (e) {
      toast.error(`Failed to load local restore-test settings: ${e}`);
    }
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

  watch(selectedScheduleProfile, async (profileId) => {
    if (!profileId) return;
    try {
      const config = await invoke<ScheduleConfig | null>("schedule_config_get", { profileId });
      if (config) {
        scheduleInterval.value = config.interval as ScheduleInterval;
        scheduleLastSnapshot.value = config.lastSnapshotAt;
        scheduleNextDue.value = config.nextDueAt;
      } else {
        scheduleInterval.value = "disabled";
        scheduleLastSnapshot.value = null;
        scheduleNextDue.value = null;
      }
    } catch {
      // Non-critical
    }
  });

  const scheduleIntervalOptions = [
    { value: "disabled", label: "Disabled" },
    { value: "hourly", label: "Every hour" },
    { value: "every_6h", label: "Every 6 hours" },
    { value: "daily", label: "Daily" },
    { value: "weekly", label: "Weekly" },
  ];

  function formatRelativeTime(iso: string | null): string {
    if (!iso) return "Never";
    const date = new Date(iso);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    if (diff < 0) {
      const mins = Math.round(Math.abs(diff) / 60000);
      if (mins < 60) return `in ${mins} minute${mins === 1 ? "" : "s"}`;
      const hrs = Math.round(mins / 60);
      if (hrs < 24) return `in ${hrs} hour${hrs === 1 ? "" : "s"}`;
      const days = Math.round(hrs / 24);
      return `in ${days} day${days === 1 ? "" : "s"}`;
    }
    const mins = Math.round(diff / 60000);
    if (mins < 60) return `${mins} minute${mins === 1 ? "" : "s"} ago`;
    const hrs = Math.round(mins / 60);
    if (hrs < 24) return `${hrs} hour${hrs === 1 ? "" : "s"} ago`;
    const days = Math.round(hrs / 24);
    return `${days} day${days === 1 ? "" : "s"} ago`;
  }

  async function saveScheduleConfig() {
    if (!selectedScheduleProfile.value) return;
    scheduleSaving.value = true;
    try {
      const config = await invoke<ScheduleConfig>("schedule_config_set", {
        profileId: selectedScheduleProfile.value,
        interval: scheduleInterval.value,
      });
      scheduleLastSnapshot.value = config.lastSnapshotAt;
      scheduleNextDue.value = config.nextDueAt;
      toast.success(
        scheduleInterval.value === "disabled"
          ? "Scheduled snapshots disabled."
          : `Scheduled snapshots set to ${scheduleIntervalOptions.find((o) => o.value === scheduleInterval.value)?.label?.toLowerCase() ?? scheduleInterval.value}.`,
      );
    } catch (e) {
      toast.error(`Failed to save schedule: ${e}`);
    } finally {
      scheduleSaving.value = false;
    }
  }

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

  async function saveRestoreTestSettings() {
    restoreTestSaving.value = true;
    try {
      await invoke("restore_test_settings_set", {
        input: {
          mysqlEnabled: restoreTestMysqlEnabled.value,
          mysqlPort: Number(restoreTestMysqlPort.value),
          mysqlUsername: restoreTestMysqlUsername.value,
          mysqlPassword: restoreTestMysqlPassword.value || null,
          postgresqlEnabled: restoreTestPostgresqlEnabled.value,
          postgresqlPort: Number(restoreTestPostgresqlPort.value),
          postgresqlUsername: restoreTestPostgresqlUsername.value,
          postgresqlPassword: restoreTestPostgresqlPassword.value || null,
        },
      });
      restoreTestMysqlPassword.value = "";
      restoreTestPostgresqlPassword.value = "";
      toast.success("Local restore-test settings saved.");
    } catch (e) {
      toast.error(`Failed to save local restore-test settings: ${e}`);
    } finally {
      restoreTestSaving.value = false;
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

      <SCard>
        <h3 class="mb-2 text-sm font-semibold text-text-primary">Automatic Restore Testing</h3>
        <p class="mb-4 text-sm text-text-secondary">
          Every new snapshot is imported into a randomly named temporary database, checked, and
          deleted. These tests are hard-wired to <code class="text-xs">127.0.0.1</code>; project
          hosts, remote credentials, and SSH tunnels are never used.
        </p>

        <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
          <div class="space-y-3 rounded-lg border border-border-subtle p-4">
            <label class="flex cursor-pointer items-center gap-2">
              <input
                v-model="restoreTestMysqlEnabled"
                type="checkbox"
                class="h-4 w-4 rounded border-border accent-accent"
              />
              <span class="text-sm font-medium text-text-primary">Test MySQL snapshots</span>
            </label>
            <SFormField label="Local port">
              <SInput
                v-model="restoreTestMysqlPort"
                type="number"
                :disabled="!restoreTestMysqlEnabled"
              />
            </SFormField>
            <SFormField label="Local username">
              <SInput v-model="restoreTestMysqlUsername" :disabled="!restoreTestMysqlEnabled" />
            </SFormField>
            <SFormField label="Local password">
              <SInput
                v-model="restoreTestMysqlPassword"
                type="password"
                placeholder="Leave blank to keep existing"
                :disabled="!restoreTestMysqlEnabled"
              />
            </SFormField>
          </div>

          <div class="space-y-3 rounded-lg border border-border-subtle p-4">
            <label class="flex cursor-pointer items-center gap-2">
              <input
                v-model="restoreTestPostgresqlEnabled"
                type="checkbox"
                class="h-4 w-4 rounded border-border accent-accent"
              />
              <span class="text-sm font-medium text-text-primary">Test PostgreSQL snapshots</span>
            </label>
            <SFormField label="Local port">
              <SInput
                v-model="restoreTestPostgresqlPort"
                type="number"
                :disabled="!restoreTestPostgresqlEnabled"
              />
            </SFormField>
            <SFormField label="Local username">
              <SInput
                v-model="restoreTestPostgresqlUsername"
                :disabled="!restoreTestPostgresqlEnabled"
              />
            </SFormField>
            <SFormField label="Local password">
              <SInput
                v-model="restoreTestPostgresqlPassword"
                type="password"
                placeholder="Leave blank to keep existing"
                :disabled="!restoreTestPostgresqlEnabled"
              />
            </SFormField>
          </div>
        </div>

        <p class="mt-3 text-xs text-text-tertiary">
          SQLite snapshots are always tested using a temporary file inside Amber's local app data.
          MySQL and PostgreSQL users need permission to create and drop local databases. Passwords
          are stored in macOS Keychain.
        </p>
        <div class="mt-4">
          <SButton
            variant="primary"
            size="sm"
            :loading="restoreTestSaving"
            @click="saveRestoreTestSettings"
          >
            Save Restore-Test Settings
          </SButton>
        </div>
      </SCard>

      <!-- Scheduled Snapshots — full width -->
      <SCard>
        <h3 class="mb-3 text-sm font-semibold text-text-primary">Scheduled Snapshots</h3>
        <p class="mb-4 text-sm text-text-secondary">
          Configure automatic snapshots on a recurring interval per profile. Scheduled snapshots are
          tagged with <code class="text-xs">[auto] scheduled</code> and subject to retention
          policies.
        </p>

        <SFormField label="Profile" class="mb-4 max-w-sm">
          <SSelect v-model="selectedScheduleProfile">
            <option value="" disabled>Select a profile&hellip;</option>
            <option v-for="opt in profileOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </SSelect>
        </SFormField>

        <div v-if="selectedScheduleProfile" class="space-y-3">
          <div class="flex items-center gap-2">
            <span class="w-40 text-sm text-text-secondary">Interval</span>
            <div class="w-48">
              <SSelect v-model="scheduleInterval">
                <option v-for="opt in scheduleIntervalOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </SSelect>
            </div>
          </div>

          <div
            v-if="scheduleInterval !== 'disabled'"
            class="space-y-1 rounded-lg border border-border-subtle bg-surface-raised px-3 py-2"
          >
            <p class="text-xs text-text-secondary">
              Last snapshot:
              <span class="font-medium text-text-primary">{{
                formatRelativeTime(scheduleLastSnapshot)
              }}</span>
            </p>
            <p class="text-xs text-text-secondary">
              Next due:
              <span class="font-medium text-text-primary">{{
                formatRelativeTime(scheduleNextDue)
              }}</span>
            </p>
          </div>

          <div class="pt-2">
            <SButton
              variant="primary"
              size="sm"
              :loading="scheduleSaving"
              @click="saveScheduleConfig"
            >
              Save Schedule
            </SButton>
          </div>
        </div>
      </SCard>

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
