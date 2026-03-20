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
  }>();

  type SortField = "name" | "createdAt" | "sizeBytes";
  type SortDirection = "asc" | "desc";

  const sortField = ref<SortField>("createdAt");
  const sortDirection = ref<SortDirection>("desc");

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
</script>

<template>
  <div class="overflow-x-auto">
    <table class="w-full text-sm">
      <thead>
        <tr
          class="border-b border-border-subtle text-left text-xs tracking-wide text-text-tertiary uppercase"
        >
          <th class="sortable-header px-3 py-2.5 font-semibold" @click="toggleSort('name')">
            Name{{ sortIndicator("name") }}
          </th>
          <th class="px-3 py-2.5 font-semibold">Note</th>
          <th class="px-3 py-2.5 font-semibold">Profile</th>
          <th class="sortable-header px-3 py-2.5 font-semibold" @click="toggleSort('createdAt')">
            Created{{ sortIndicator("createdAt") }}
          </th>
          <th
            class="sortable-header px-3 py-2.5 text-right font-semibold"
            @click="toggleSort('sizeBytes')"
          >
            Size{{ sortIndicator("sizeBytes") }}
          </th>
          <th class="px-3 py-2.5 text-center font-semibold">Integrity</th>
          <th class="px-3 py-2.5 text-right font-semibold">Actions</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="snapshot in sortedSnapshots"
          :key="snapshot.id"
          class="table-row border-b border-border-subtle"
        >
          <td class="px-3 py-3">
            <span class="font-medium text-text-primary">{{ snapshot.name }}</span>
          </td>
          <td class="max-w-[200px] truncate px-3 py-3 text-text-secondary">
            {{ snapshot.note || "—" }}
          </td>
          <td class="px-3 py-3 text-text-secondary">
            {{ getProfileName(snapshot.profileId) }}
          </td>
          <td class="px-3 py-3 whitespace-nowrap text-text-secondary">
            {{ formatDate(snapshot.createdAt) }}
          </td>
          <td class="px-3 py-3 text-right font-mono whitespace-nowrap text-text-secondary">
            {{ formatSize(snapshot.sizeBytes) }}
          </td>
          <td class="px-3 py-3 text-center">
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
              Verify
            </button>
            <span
              v-else
              class="text-xs text-text-tertiary"
              title="No checksum — created before integrity tracking"
              >—</span
            >
          </td>
          <td class="px-3 py-3 text-right">
            <div class="flex items-center justify-end gap-1">
              <SButton variant="ghost" size="sm" @click="emit('restore', snapshot.id)">
                Restore
              </SButton>
              <SButton
                variant="ghost"
                size="sm"
                @click="emit('reveal', snapshot)"
                title="Show in Finder"
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
