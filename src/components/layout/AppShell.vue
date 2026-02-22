<script setup lang="ts">
import { provide } from 'vue'
import { RouterView } from 'vue-router'
import AppSidebar from './AppSidebar.vue'
import ToastNotification from '@/components/ui/ToastNotification.vue'
import { ToastKey, useToastProvider } from '@/composables/useToast'

const toast = useToastProvider()
provide(ToastKey, toast)
</script>

<template>
  <div class="relative h-screen overflow-hidden text-text-primary antialiased selection:bg-accent selection:text-white">
    <!-- Ambient blobs -->
    <div class="ambient-blobs" aria-hidden="true">
      <div class="ambient-blob blob-1"></div>
      <div class="ambient-blob blob-2"></div>
      <div class="ambient-blob blob-3"></div>
    </div>

    <!-- Titlebar drag region -->
    <div class="titlebar-drag-region fixed top-0 left-0 right-0 z-[9999] h-7 border-b border-border-subtle"></div>

    <!-- Skip link -->
    <a href="#main-content" class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:bg-surface-raised focus:px-4 focus:py-2 focus:rounded-lg focus:text-text-primary">
      Skip to content
    </a>

    <!-- Layout -->
    <div class="flex h-full pt-7" style="position: relative; z-index: 2;">
      <AppSidebar />
      <main id="main-content" class="titlebar-no-drag flex flex-1 flex-col overflow-y-auto px-6 pb-6" style="background: transparent;">
        <RouterView />
      </main>
    </div>

    <!-- Toasts -->
    <div class="fixed bottom-4 right-4 z-[9999] flex flex-col gap-2">
      <TransitionGroup
        enter-from-class="translate-y-3 opacity-0"
        enter-active-class="transition-all duration-300 ease-out"
        leave-to-class="translate-y-3 opacity-0"
        leave-active-class="transition-all duration-200 ease-in"
      >
        <ToastNotification
          v-for="t in toast.toasts.value"
          :key="t.id"
          :message="t.message"
          :type="t.type"
          :duration="t.duration"
          @dismiss="toast.dismiss(t.id)"
        />
      </TransitionGroup>
    </div>
  </div>
</template>
