<script setup lang="ts">
interface FormTextareaProps {
  modelValue: string
  label?: string
  placeholder?: string
  rows?: number
  error?: string
  disabled?: boolean
}

withDefaults(defineProps<FormTextareaProps>(), {
  rows: 4,
  disabled: false,
})

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <div class="form-field flex flex-col gap-1.5">
    <label v-if="label" class="form-label text-sm font-medium text-text-secondary">
      {{ label }}
    </label>
    <textarea
      :value="modelValue"
      :placeholder="placeholder"
      :rows="rows"
      :disabled="disabled"
      class="form-input"
      :class="error ? 'form-input-error' : ''"
      @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />
    <p v-if="error" class="text-xs text-danger mt-1">{{ error }}</p>
  </div>
</template>

<style scoped>
.form-input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 10px;
  border: 1px solid var(--color-border);
  background: color-mix(in srgb, var(--color-surface-base) 80%, transparent);
  color: var(--color-text-primary);
  font-size: var(--text-sm);
  font-family: inherit;
  min-height: 5rem;
  resize: vertical;
  line-height: 1.6;
  transition: border-color 150ms ease, box-shadow 150ms ease;
}

.form-input::placeholder {
  color: var(--color-text-muted);
}

.form-input:hover {
  border-color: var(--color-border-strong);
}

.form-input:focus {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-focus);
  outline: none;
}

.form-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-input-error {
  border-color: var(--color-danger);
}

.form-input-error:focus {
  border-color: var(--color-danger);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-danger) 22%, transparent);
}
</style>
