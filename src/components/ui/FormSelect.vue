<script setup lang="ts">
interface FormSelectProps {
  modelValue: string | number
  label?: string
  options: Array<{ value: string | number; label: string }>
  size?: 'sm' | 'md'
  error?: string
  disabled?: boolean
}

withDefaults(defineProps<FormSelectProps>(), {
  size: 'md',
  disabled: false,
})

defineEmits<{
  'update:modelValue': [value: string | number]
}>()
</script>

<template>
  <div class="form-field flex flex-col gap-1.5">
    <label v-if="label" class="form-label text-sm font-medium text-text-secondary">
      {{ label }}
    </label>
    <select
      :value="modelValue"
      :disabled="disabled"
      class="form-select"
      :class="[
        size === 'sm' ? 'min-h-[2rem]' : 'min-h-[2.5rem]',
        error ? 'form-input-error' : '',
      ]"
      @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <option
        v-for="opt in options"
        :key="opt.value"
        :value="opt.value"
      >
        {{ opt.label }}
      </option>
    </select>
    <p v-if="error" class="text-xs text-danger mt-1">{{ error }}</p>
  </div>
</template>

<style scoped>
.form-select {
  width: 100%;
  padding: 0.5rem 2.25rem 0.5rem 0.75rem;
  border-radius: 10px;
  border: 1px solid var(--color-border);
  background: color-mix(in srgb, var(--color-surface-base) 80%, transparent);
  color: var(--color-text-primary);
  font-size: var(--text-sm);
  font-family: inherit;
  appearance: none;
  cursor: pointer;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23999999' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 0.75rem center;
  transition: border-color 150ms ease, box-shadow 150ms ease;
}

.form-select:hover {
  border-color: var(--color-border-strong);
}

.form-select:focus {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-focus);
  outline: none;
}

.form-select:disabled {
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
