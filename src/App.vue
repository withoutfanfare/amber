<script setup lang="ts">
  import { onMounted } from "vue";
  import { useRouter } from "vue-router";
  import { invoke } from "@tauri-apps/api/core";
  import AppShell from "@/components/layout/AppShell.vue";
  import { useProfileStore } from "@/stores/profiles";

  const router = useRouter();
  const profileStore = useProfileStore();

  // Pause blob animations when tab is hidden
  document.addEventListener("visibilitychange", () => {
    document.documentElement.classList.toggle("page-hidden", document.hidden);
  });

  // First-run setup wizard: trigger when no profiles exist and setup not completed
  onMounted(async () => {
    try {
      const setupComplete = await invoke<boolean>("get_setup_complete");
      if (!setupComplete) {
        await profileStore.fetchAll();
        if (profileStore.profiles.length === 0) {
          router.push("/setup");
        }
      }
    } catch {
      // Non-critical — skip wizard check if it fails
    }
  });
</script>

<template>
  <AppShell />
</template>
