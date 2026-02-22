<script setup lang="ts">
import { computed } from 'vue'
import FormInput from '@/components/ui/FormInput.vue'

interface SshTunnelValue {
  enabled: boolean
  host: string
  port: number
  user: string
  keyPath: string
  password: string
}

const props = defineProps<{
  modelValue: SshTunnelValue
}>()

const emit = defineEmits<{
  'update:modelValue': [value: SshTunnelValue]
}>()

const model = computed({
  get: () => props.modelValue,
  set: (val: SshTunnelValue) => emit('update:modelValue', val),
})

function update(field: keyof SshTunnelValue, value: string | number | boolean) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}
</script>

<template>
  <div class="space-y-4">
    <label class="flex items-center gap-2 cursor-pointer">
      <input
        type="checkbox"
        :checked="model.enabled"
        class="h-4 w-4 rounded border-border accent-accent"
        @change="update('enabled', ($event.target as HTMLInputElement).checked)"
      />
      <span class="text-sm font-medium text-text-secondary">Enable SSH tunnel</span>
    </label>

    <template v-if="model.enabled">
      <div class="grid grid-cols-2 gap-4">
        <FormInput
          :model-value="model.host"
          label="SSH Host"
          placeholder="ssh.example.com"
          @update:model-value="update('host', $event)"
        />
        <FormInput
          :model-value="model.port"
          label="SSH Port"
          type="number"
          placeholder="22"
          @update:model-value="update('port', Number($event))"
        />
      </div>
      <FormInput
        :model-value="model.user"
        label="SSH User"
        placeholder="deploy"
        @update:model-value="update('user', $event)"
      />
      <FormInput
        :model-value="model.keyPath"
        label="SSH Key Path"
        placeholder="~/.ssh/id_rsa"
        @update:model-value="update('keyPath', $event)"
      />
      <FormInput
        :model-value="model.password"
        label="SSH Password"
        type="password"
        placeholder="SSH passphrase or password"
        @update:model-value="update('password', $event)"
      />
    </template>
  </div>
</template>
