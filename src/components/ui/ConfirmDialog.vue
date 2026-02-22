<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useFocusTrap } from '@/composables/useFocusTrap'

interface ConfirmDialogProps {
  open: boolean
  title: string
  message: string
  confirmLabel?: string
  danger?: boolean
  typedConfirmation?: string
}

const props = withDefaults(defineProps<ConfirmDialogProps>(), {
  confirmLabel: 'Confirm',
  danger: false,
})

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const dialogRef = ref<HTMLElement | null>(null)
const typedValue = ref('')
const { activate, deactivate } = useFocusTrap(dialogRef)

const canConfirm = () => {
  if (!props.typedConfirmation) return true
  return typedValue.value === props.typedConfirmation
}

watch(() => props.open, async (isOpen) => {
  if (isOpen) {
    typedValue.value = ''
    await nextTick()
    activate()
  } else {
    deactivate()
  }
})

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('cancel')
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="backdrop fixed inset-0 z-50 flex items-center justify-center"
      @keydown="handleKeyDown"
    >
      <div
        ref="dialogRef"
        role="alertdialog"
        aria-modal="true"
        :aria-label="title"
        class="modal animate-scale-in mx-4 w-full max-w-md p-6"
      >
        <h2 class="text-lg font-semibold text-text-primary mb-2">{{ title }}</h2>
        <p class="text-sm text-text-secondary mb-5">{{ message }}</p>

        <slot />

        <div v-if="typedConfirmation" class="mb-5">
          <p class="text-xs text-text-tertiary mb-2">
            Type <strong class="text-text-primary">{{ typedConfirmation }}</strong> to confirm
          </p>
          <input
            v-model="typedValue"
            type="text"
            class="control-field-sm w-full px-3 text-text-primary"
            :placeholder="typedConfirmation"
          />
        </div>

        <div class="flex justify-end gap-3">
          <button
            class="inline-flex items-center justify-center rounded-lg px-4 py-1.5 text-sm font-medium text-text-secondary transition-all duration-150 hover:bg-surface-overlay active:scale-[0.98]"
            @click="emit('cancel')"
          >
            Cancel
          </button>
          <button
            :disabled="!canConfirm()"
            class="inline-flex items-center justify-center rounded-lg px-4 py-1.5 text-sm font-medium transition-all duration-150 active:scale-[0.98] disabled:opacity-50 disabled:pointer-events-none"
            :class="danger ? 'bg-danger text-white hover:brightness-110' : 'bg-accent-strong text-white hover:brightness-110'"
            @click="emit('confirm')"
          >
            {{ confirmLabel }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
