<script setup lang="ts">
  import { ref, onMounted } from "vue";
  import { useRouter } from "vue-router";
  import { invoke } from "@tauri-apps/api/core";
  import { useToastStack, SButton, SCard, SInput } from "@stuntrocket/ui";
  import type { DiscoveredTool } from "@/types";

  const router = useRouter();
  const toast = useToastStack();

  const tools = ref<DiscoveredTool[]>([]);
  const scanning = ref(false);
  const customPaths = ref<Record<string, string>>({});

  onMounted(async () => {
    await scanTools();
  });

  async function scanTools() {
    scanning.value = true;
    try {
      tools.value = await invoke<DiscoveredTool[]>("discover_tools");
    } catch (e) {
      toast.error(`Tool discovery failed: ${e}`);
    } finally {
      scanning.value = false;
    }
  }

  async function saveCustomPath(toolName: string) {
    const path = customPaths.value[toolName];
    if (!path?.trim()) return;
    try {
      await invoke("save_tool_path", { toolName, path: path.trim() });
      toast.success(`Path saved for ${toolName}.`);
      await scanTools();
    } catch (e) {
      toast.error(`Failed to save path: ${e}`);
    }
  }

  async function completeSetup() {
    try {
      await invoke("settings_set", { key: "setup_complete", value: "true" });
      toast.success("Setup complete.");
      router.push("/");
    } catch (e) {
      toast.error(`Failed to save setup status: ${e}`);
    }
  }
</script>

<template>
  <div class="mx-auto max-w-2xl py-8">
    <div class="mb-8 text-center">
      <h1 class="mb-2 text-2xl font-semibold text-text-primary">Welcome to Amber</h1>
      <p class="text-sm text-text-secondary">
        Amber needs database tools to create and restore snapshots. Let's check what's available on
        your system.
      </p>
    </div>

    <SCard class="mb-6">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-sm font-semibold text-text-primary">Database Tools</h2>
        <SButton variant="ghost" size="sm" :loading="scanning" @click="scanTools">
          Re-scan
        </SButton>
      </div>

      <div v-if="scanning" class="py-8 text-center text-sm text-text-secondary">
        Scanning PATH for database tools&hellip;
      </div>

      <div v-else class="space-y-4">
        <div
          v-for="tool in tools"
          :key="tool.name"
          class="flex items-start gap-3 rounded-lg border border-border-subtle px-3 py-3"
          :class="tool.found ? 'bg-success/5' : 'bg-surface-raised'"
        >
          <!-- Status indicator -->
          <div class="mt-0.5">
            <svg
              v-if="tool.found"
              class="h-5 w-5 text-success"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
              <polyline points="22 4 12 14.01 9 11.01" />
            </svg>
            <svg
              v-else
              class="h-5 w-5 text-text-tertiary"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
          </div>

          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="font-mono text-sm font-medium text-text-primary">{{ tool.name }}</span>
              <span v-if="tool.version" class="text-xs text-text-tertiary"
                >v{{ tool.version }}</span
              >
              <span class="text-xs text-text-tertiary">(min: v{{ tool.minVersion }})</span>
            </div>

            <div v-if="tool.found && tool.path" class="mt-0.5">
              <p class="truncate font-mono text-xs text-text-tertiary">{{ tool.path }}</p>
            </div>

            <div v-if="!tool.found" class="mt-2 space-y-2">
              <p class="text-xs text-text-secondary">
                Not found. Install with: <code class="text-accent">{{ tool.installHint }}</code>
              </p>
              <div class="flex items-center gap-2">
                <SInput
                  v-model="customPaths[tool.name]"
                  placeholder="/usr/local/bin/..."
                  class="flex-1"
                />
                <SButton
                  variant="secondary"
                  size="sm"
                  :disabled="!customPaths[tool.name]?.trim()"
                  @click="saveCustomPath(tool.name)"
                >
                  Set Path
                </SButton>
              </div>
            </div>
          </div>
        </div>
      </div>
    </SCard>

    <!-- Summary -->
    <SCard class="mb-6">
      <h2 class="mb-2 text-sm font-semibold text-text-primary">Summary</h2>
      <p class="text-sm text-text-secondary">
        {{ tools.filter((t) => t.found).length }} of {{ tools.length }} tools found.
        <span v-if="tools.filter((t) => t.found).length >= 3" class="text-success">
          You're ready to start using Amber.
        </span>
        <span v-else class="text-warning">
          Some tools are missing. You can still use Amber with the available database types.
        </span>
      </p>
    </SCard>

    <div class="flex items-center justify-between">
      <SButton variant="ghost" size="md" @click="router.push('/')"> Skip for now </SButton>
      <SButton variant="primary" size="md" @click="completeSetup"> Complete Setup </SButton>
    </div>
  </div>
</template>
