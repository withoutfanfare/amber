<script setup lang="ts"></script>

<template>
  <div class="page-header sticky top-0 z-20 -mx-6 mb-4 flex items-center justify-between px-6 py-3">
    <div class="flex items-center gap-4">
      <slot name="prepend" />
    </div>
    <div class="flex items-center gap-3">
      <slot name="actions" />
    </div>
  </div>
</template>
