<script setup lang="ts">
  import { ref, computed } from "vue";
  import { SButton } from "@stuntrocket/ui";
  import type { Snapshot, Profile } from "@/types";

  const props = defineProps<{
    snapshots: Snapshot[];
    profiles: Profile[];
  }>();

  const emit = defineEmits<{
    restore: [id: string];
    delete: [id: string];
    reveal: [snapshot: Snapshot];
    verify: [id: string];
    export: [id: string];
    pin: [id: string, pinned: boolean];
    addTag: [id: string];
    removeTag: [id: string, tag: string];
    compare: [id: string];
    preview: [id: string];
    versionCheck: [id: string];
    browse: [id: string];
  }>();

  type SortField = "name" | "createdAt" | "sizeBytes";
  type SortDirection = "asc" | "desc";

  const sortField = ref<SortField>("createdAt");
  const sortDirection = ref<SortDirection>("desc");
  const selectedForCompare = ref<string[]>([]);

  function toggleSort(field: SortField) {
    if (sortField.value === field) {
      sortDirection.value = sortDirection.value === "asc" ? "desc" : "asc";
    } else {
      sortField.value = field;
      sortDirection.value = field === "name" ? "asc" : "desc";
    }
  }

  const sortedSnapshots = computed(() => {
    const list = [...props.snapshots];
    const dir = sortDirection.value === "asc" ? 1 : -1;
    return list.sort((a, b) => {
      switch (sortField.value) {
        case "name":
          return dir * a.name.localeCompare(b.name);
        case "createdAt":
          return dir * (new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime());
        case "sizeBytes":
          return dir * (a.sizeBytes - b.sizeBytes);
        default:
          return 0;
      }
    });
  });

  function getProfileName(profileId: string): string {
    const profile = props.profiles.find((p) => p.id === profileId);
    return profile ? `${profile.project} / ${profile.name}` : "Unknown profile";
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function formatDate(iso: string): string {
    const date = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString("en-GB", { day: "numeric", month: "short", year: "numeric" });
  }

  function sortIndicator(field: SortField): string {
    if (sortField.value !== field) return "";
    return sortDirection.value === "asc" ? " \u2191" : " \u2193";
  }

  function toggleCompareSelect(id: string) {
    const idx = selectedForCompare.value.indexOf(id);
    if (idx >= 0) {
      selectedForCompare.value.splice(idx, 1);
    } else if (selectedForCompare.value.length < 2) {
      selectedForCompare.value.push(id);
    }
  }

  function handleCompare() {
    if (selectedForCompare.value.length === 2) {
      emit("compare", selectedForCompare.value[0]);
    }
  }

  const canCompare = computed(() => selectedForCompare.value.length === 2);
</script>

<template>
  <div>
    <!-- Compare toolbar -->
    <div
      v-if="selectedForCompare.length > 0"
      class="mb-3 flex items-center gap-3 rounded-lg bg-surface-raised px-3 py-2 text-sm"
    >
      <span class="text-text-secondary"
        >{{ selectedForCompare.length }}/2 selected for comparison</span
      >
      <SButton v-if="canCompare" variant="primary" size="sm" @click="handleCompare">
        Compare Schema
      </SButton>
      <SButton variant="ghost" size="sm" @click="selectedForCompare = []">Clear</SButton>
    </div>

    <div class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead>
          <tr
            class="border-b border-border-subtle text-left text-xs tracking-wide text-text-tertiary uppercase"
          >
            <th class="w-8 px-2 py-2.5 font-semibold">
              <span title="Select for comparison" class="cursor-help">Cmp</span>
            </th>
            <th class="sortable-header px-3 py-2.5 font-semibold" @click="toggleSort('name')">
              Name{{ sortIndicator("name") }}
            </th>
            <th class="px-3 py-2.5 font-semibold">Tags</th>
            <th class="px-3 py-2.5 font-semibold">Profile</th>
            <th class="px-3 py-2.5 font-semibold">Database</th>
            <th class="sortable-header px-3 py-2.5 font-semibold" @click="toggleSort('createdAt')">
              Created{{ sortIndicator("createdAt") }}
            </th>
            <th
              class="sortable-header px-3 py-2.5 text-right font-semibold"
              @click="toggleSort('sizeBytes')"
            >
              Size{{ sortIndicator("sizeBytes") }}
            </th>
            <th class="px-3 py-2.5 font-semibold">Version</th>
            <th class="px-3 py-2.5 text-center font-semibold">Status</th>
            <th class="px-3 py-2.5 text-right font-semibold">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="snapshot in sortedSnapshots"
            :key="snapshot.id"
            class="table-row border-b border-border-subtle"
            :class="{ 'bg-accent/5': selectedForCompare.includes(snapshot.id) }"
          >
            <!-- Compare checkbox -->
            <td class="px-2 py-3">
              <input
                type="checkbox"
                :checked="selectedForCompare.includes(snapshot.id)"
                :disabled="
                  !selectedForCompare.includes(snapshot.id) && selectedForCompare.length >= 2
                "
                class="h-3.5 w-3.5 rounded border-border accent-accent"
                @change="toggleCompareSelect(snapshot.id)"
              />
            </td>
            <!-- Name + pinned indicator -->
            <td class="px-3 py-3">
              <div class="flex items-center gap-1.5">
                <button
                  v-if="snapshot.pinned"
                  class="text-accent"
                  title="Pinned — protected from auto-cleanup"
                  @click="emit('pin', snapshot.id, false)"
                >
                  <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M16 2l-4 4-6-2-2 10 4.5 4.5L2 25l6.5-6.5L13 23l10-2-2-6 4-4-9-9z" />
                  </svg>
                </button>
                <span class="font-medium text-text-primary">{{ snapshot.name }}</span>
              </div>
              <p
                v-if="snapshot.note"
                class="mt-0.5 max-w-[200px] truncate text-xs text-text-tertiary"
              >
                {{ snapshot.note }}
              </p>
            </td>
            <!-- Tags -->
            <td class="px-3 py-3">
              <div class="flex flex-wrap gap-1">
                <span
                  v-for="tag in snapshot.tags"
                  :key="tag"
                  class="inline-flex items-center gap-0.5 rounded-full bg-accent/10 px-2 py-0.5 text-[10px] font-medium text-accent"
                >
                  {{ tag }}
                  <button
                    class="ml-0.5 text-accent/50 hover:text-accent"
                    @click.stop="emit('removeTag', snapshot.id, tag)"
                  >
                    &times;
                  </button>
                </span>
                <button
                  class="rounded-full border border-dashed border-border-subtle px-1.5 py-0.5 text-[10px] text-text-tertiary transition-colors hover:border-accent hover:text-accent"
                  @click="emit('addTag', snapshot.id)"
                >
                  + tag
                </button>
              </div>
            </td>
            <td class="px-3 py-3 text-text-secondary">
              {{ getProfileName(snapshot.profileId) }}
            </td>
            <td class="px-3 py-3 font-mono text-xs text-text-secondary">
              {{ snapshot.databaseName ?? "—" }}
            </td>
            <td class="px-3 py-3 whitespace-nowrap text-text-secondary">
              {{ formatDate(snapshot.createdAt) }}
            </td>
            <td class="px-3 py-3 text-right font-mono whitespace-nowrap text-text-secondary">
              {{ formatSize(snapshot.sizeBytes) }}
            </td>
            <!-- Version info -->
            <td class="px-3 py-3">
              <span v-if="snapshot.dumpToolVersion" class="font-mono text-xs text-text-tertiary">
                v{{ snapshot.dumpToolVersion }}
              </span>
              <span v-else class="text-xs text-text-tertiary">&mdash;</span>
            </td>
            <!-- Status (integrity + pinned) -->
            <td class="px-3 py-3 text-center">
              <div class="flex items-center justify-center gap-1">
                <span
                  v-if="snapshot.restoreTestStatus === 'passed'"
                  class="inline-flex h-5 w-5 items-center justify-center rounded-full bg-success/10 text-xs font-semibold text-success"
                  :title="snapshot.restoreTestMessage ?? 'Local restore test passed'"
                  :aria-label="snapshot.restoreTestMessage ?? 'Local restore test passed'"
                  >R</span
                >
                <span
                  v-else-if="snapshot.restoreTestStatus === 'failed'"
                  class="inline-flex h-5 w-5 items-center justify-center rounded-full bg-danger/10 text-xs font-semibold text-danger"
                  :title="snapshot.restoreTestMessage ?? 'Local restore test failed'"
                  :aria-label="snapshot.restoreTestMessage ?? 'Local restore test failed'"
                  >R</span
                >
                <span
                  v-else
                  class="inline-flex h-5 w-5 items-center justify-center rounded-full bg-surface-raised text-xs font-semibold text-text-tertiary"
                  :title="snapshot.restoreTestMessage ?? 'Local restore testing not configured'"
                  :aria-label="
                    snapshot.restoreTestMessage ?? 'Local restore testing not configured'
                  "
                  >R</span
                >
                <button
                  v-if="snapshot.checksum"
                  class="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-success/80 transition-colors hover:bg-success/10 hover:text-success"
                  title="SHA-256 checksum recorded — click to verify"
                  @click="emit('verify', snapshot.id)"
                >
                  <svg
                    class="h-3.5 w-3.5"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
                  </svg>
                </button>
                <span
                  v-else
                  class="text-xs text-text-tertiary"
                  title="No checksum — created before integrity tracking"
                  >&mdash;</span
                >
              </div>
            </td>
            <td class="px-3 py-3 text-right">
              <div class="flex items-center justify-end gap-1">
                <SButton
                  variant="ghost"
                  size="sm"
                  title="Browse snapshot contents"
                  @click="emit('browse', snapshot.id)"
                >
                  Browse
                </SButton>
                <SButton
                  variant="ghost"
                  size="sm"
                  title="Preview restore impact"
                  @click="emit('preview', snapshot.id)"
                >
                  Preview
                </SButton>
                <SButton variant="ghost" size="sm" @click="emit('restore', snapshot.id)">
                  Restore
                </SButton>
                <SButton
                  variant="ghost"
                  size="sm"
                  title="Export as SQL file"
                  @click="emit('export', snapshot.id)"
                >
                  <svg
                    class="h-3.5 w-3.5"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                    <polyline points="7 10 12 15 17 10" />
                    <line x1="12" y1="15" x2="12" y2="3" />
                  </svg>
                </SButton>
                <SButton
                  variant="ghost"
                  size="sm"
                  :title="
                    snapshot.pinned ? 'Unpin snapshot' : 'Pin snapshot (protect from auto-cleanup)'
                  "
                  @click="emit('pin', snapshot.id, !snapshot.pinned)"
                >
                  <svg
                    class="h-3.5 w-3.5"
                    :class="snapshot.pinned ? 'text-accent' : ''"
                    viewBox="0 0 24 24"
                    :fill="snapshot.pinned ? 'currentColor' : 'none'"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <line x1="12" y1="17" x2="12" y2="22" />
                    <path
                      d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"
                    />
                  </svg>
                </SButton>
                <SButton
                  variant="ghost"
                  size="sm"
                  title="Show in Finder"
                  @click="emit('reveal', snapshot)"
                >
                  <svg
                    class="h-3.5 w-3.5"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path
                      d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                    />
                  </svg>
                </SButton>
                <SButton
                  variant="ghost"
                  size="sm"
                  class="text-danger hover:text-danger"
                  @click="emit('delete', snapshot.id)"
                >
                  Delete
                </SButton>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
  .sortable-header {
    cursor: pointer;
    user-select: none;
    transition: color 150ms ease;
  }

  .sortable-header:hover {
    color: var(--color-text-primary);
  }

  .table-row {
    transition: background-color 150ms ease;
  }

  .table-row:hover {
    background: color-mix(in srgb, var(--color-surface-raised) 40%, transparent);
  }
</style>
