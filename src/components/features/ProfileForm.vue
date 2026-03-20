<script setup lang="ts">
import { reactive, computed, watch } from 'vue'
import { SFormField, SInput, SSelect, STextarea, SButton } from '@stuntrocket/ui'
import SshTunnelConfig from './SshTunnelConfig.vue'
import type { Profile, DbType, Environment, ProfileCreatePayload } from '@/types'

const props = withDefaults(defineProps<{
  initialData?: Partial<Profile>
  submitting?: boolean
}>(), {
  submitting: false,
})

const emit = defineEmits<{
  submit: [payload: ProfileCreatePayload]
  cancel: []
}>()

const dbTypeOptions = [
  { value: 'mysql', label: 'MySQL' },
  { value: 'postgresql', label: 'PostgreSQL' },
  { value: 'sqlite', label: 'SQLite' },
]

const environmentOptions = [
  { value: '', label: 'Not set' },
  { value: 'local', label: 'Local' },
  { value: 'staging', label: 'Staging' },
  { value: 'live', label: 'Live' },
]

const form = reactive({
  project: props.initialData?.project ?? '',
  name: props.initialData?.name ?? '',
  dbType: (props.initialData?.dbType ?? 'mysql') as DbType,
  host: props.initialData?.host ?? '',
  port: props.initialData?.port ?? 3306,
  databaseName: props.initialData?.databaseName ?? '',
  username: props.initialData?.username ?? '',
  password: '',
  environment: (props.initialData?.environment ?? '') as Environment | '',
  notes: props.initialData?.notes ?? '',
  ssh: {
    enabled: props.initialData?.sshEnabled ?? false,
    host: props.initialData?.sshHost ?? '',
    port: props.initialData?.sshPort ?? 22,
    user: props.initialData?.sshUser ?? '',
    keyPath: '',
    password: '',
  },
})

const isSqlite = computed(() => form.dbType === 'sqlite')

// Set sensible default port when DB type changes
watch(() => form.dbType, (newType) => {
  if (newType === 'mysql') form.port = 3306
  else if (newType === 'postgresql') form.port = 5432
  else form.port = 0
})

function handleSubmit() {
  const payload: ProfileCreatePayload = {
    project: form.project,
    name: form.name,
    dbType: form.dbType,
    databaseName: form.databaseName,
    ...(isSqlite.value ? {} : {
      host: form.host || undefined,
      port: form.port || undefined,
      username: form.username || undefined,
    }),
    ...(form.password ? { password: form.password } : {}),
    ...(form.environment ? { environment: form.environment as Environment } : {}),
    ...(form.notes ? { notes: form.notes } : {}),
    ...(form.ssh.enabled ? {
      sshEnabled: true,
      sshHost: form.ssh.host || undefined,
      sshPort: form.ssh.port || undefined,
      sshUser: form.ssh.user || undefined,
      sshKeyPath: form.ssh.keyPath || undefined,
      sshPassword: form.ssh.password || undefined,
    } : { sshEnabled: false }),
  }
  emit('submit', payload)
}
</script>

<template>
  <form class="space-y-5" @submit.prevent="handleSubmit">
    <div class="grid grid-cols-2 gap-4">
      <SFormField label="Project">
        <SInput v-model="form.project" placeholder="my-app" />
      </SFormField>
      <SFormField label="Profile Name">
        <SInput v-model="form.name" placeholder="local-main" />
      </SFormField>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <SFormField label="Database Type">
        <SSelect v-model="form.dbType">
          <option v-for="opt in dbTypeOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </SSelect>
      </SFormField>
      <SFormField label="Environment">
        <SSelect v-model="form.environment">
          <option v-for="opt in environmentOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </SSelect>
      </SFormField>
    </div>

    <div v-if="!isSqlite" class="grid grid-cols-2 gap-4">
      <SFormField label="Host">
        <SInput v-model="form.host" placeholder="127.0.0.1" />
      </SFormField>
      <SFormField label="Port">
        <SInput v-model="form.port" type="number" :placeholder="form.dbType === 'mysql' ? '3306' : '5432'" />
      </SFormField>
    </div>

    <SFormField :label="isSqlite ? 'Database File Path' : 'Database Name'">
      <SInput v-model="form.databaseName" :placeholder="isSqlite ? '/path/to/database.db' : 'my_database'" />
    </SFormField>

    <div v-if="!isSqlite" class="grid grid-cols-2 gap-4">
      <SFormField label="Username">
        <SInput v-model="form.username" placeholder="root" />
      </SFormField>
      <SFormField label="Password">
        <SInput v-model="form.password" type="password" placeholder="Enter password" />
      </SFormField>
    </div>

    <SFormField label="Notes">
      <STextarea v-model="form.notes" placeholder="Optional notes about this connection" :rows="3" />
    </SFormField>

    <div class="border-t border-border-subtle pt-4">
      <SshTunnelConfig v-model="form.ssh" />
    </div>

    <!-- Sticky footer -->
    <div class="sticky bottom-0 -mx-6 mt-6 flex items-center justify-end gap-3 border-t border-border-subtle bg-surface-base/80 px-6 py-4 backdrop-blur-sm">
      <SButton variant="ghost" @click="emit('cancel')">
        Cancel
      </SButton>
      <SButton variant="primary" :loading="submitting">
        {{ initialData?.id ? 'Save Changes' : 'Create Profile' }}
      </SButton>
    </div>
  </form>
</template>
