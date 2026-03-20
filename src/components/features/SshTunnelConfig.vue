<script setup lang="ts">
import { computed } from 'vue'
import { SFormField, SInput } from '@stuntrocket/ui'

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
        <SFormField label="SSH Host">
          <SInput
            :model-value="model.host"
            placeholder="ssh.example.com"
            @update:model-value="update('host', $event)"
          />
        </SFormField>
        <SFormField label="SSH Port">
          <SInput
            :model-value="model.port"
            type="number"
            placeholder="22"
            @update:model-value="update('port', Number($event))"
          />
        </SFormField>
      </div>
      <SFormField label="SSH User">
        <SInput
          :model-value="model.user"
          placeholder="deploy"
          @update:model-value="update('user', $event)"
        />
      </SFormField>
      <SFormField label="SSH Key Path">
        <SInput
          :model-value="model.keyPath"
          placeholder="~/.ssh/id_rsa"
          @update:model-value="update('keyPath', $event)"
        />
      </SFormField>
      <SFormField label="SSH Password">
        <SInput
          :model-value="model.password"
          type="password"
          placeholder="SSH passphrase or password"
          @update:model-value="update('password', $event)"
        />
      </SFormField>
    </template>
  </div>
</template>
