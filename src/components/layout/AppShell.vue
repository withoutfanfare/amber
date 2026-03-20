<script setup lang="ts">
  import { RouterView } from "vue-router";
  import AppSidebar from "./AppSidebar.vue";
  import { SToastContainer } from "@stuntrocket/ui";
  import { useKeyboardShortcuts } from "@/composables/useKeyboardShortcuts";

  const { helpOverlayOpen, shortcuts } = useKeyboardShortcuts();
</script>

<template>
  <div
    class="relative h-screen overflow-hidden text-text-primary antialiased selection:bg-accent selection:text-white"
  >
    <!-- Ambient blobs -->
    <div class="ambient-blobs" aria-hidden="true">
      <div class="ambient-blob blob-1"></div>
      <div class="ambient-blob blob-2"></div>
      <div class="ambient-blob blob-3"></div>
    </div>

    <!-- Titlebar drag region -->
    <div
      class="titlebar-drag-region fixed top-0 right-0 left-0 z-[9999] h-7 border-b border-border-subtle"
    ></div>

    <!-- Skip link -->
    <a
      href="#main-content"
      class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:rounded-lg focus:bg-surface-raised focus:px-4 focus:py-2 focus:text-text-primary"
    >
      Skip to content
    </a>

    <!-- Layout -->
    <div class="flex h-full pt-7" style="position: relative; z-index: 2">
      <AppSidebar />
      <main
        id="main-content"
        class="titlebar-no-drag flex flex-1 flex-col overflow-y-auto px-6 pb-6"
        style="background: transparent"
      >
        <RouterView />
      </main>
    </div>

    <!-- Keyboard shortcuts help overlay -->
    <Teleport to="body">
      <div
        v-if="helpOverlayOpen"
        class="fixed inset-0 z-[200] flex items-center justify-center bg-black/50 backdrop-blur-sm"
        @click.self="helpOverlayOpen = false"
      >
        <div class="w-96 rounded-xl border border-border-subtle bg-surface-base p-6 shadow-2xl">
          <div class="mb-4 flex items-center justify-between">
            <h2 class="text-base font-semibold text-text-primary">Keyboard Shortcuts</h2>
            <button
              class="text-text-tertiary transition-colors hover:text-text-primary"
              @click="helpOverlayOpen = false"
            >
              <svg
                class="h-5 w-5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
          <div class="space-y-2">
            <div
              v-for="shortcut in shortcuts"
              :key="shortcut.keys"
              class="flex items-center justify-between border-b border-border-subtle py-1.5 last:border-0"
            >
              <span class="text-sm text-text-secondary">{{ shortcut.description }}</span>
              <kbd
                class="rounded border border-border-subtle bg-surface-raised px-2 py-0.5 font-mono text-xs text-text-primary"
              >
                {{ shortcut.keys }}
              </kbd>
            </div>
          </div>
          <p class="mt-4 text-center text-xs text-text-tertiary">Press Escape to close</p>
        </div>
      </div>
    </Teleport>

    <!-- Toasts -->
    <SToastContainer />
  </div>
</template>
