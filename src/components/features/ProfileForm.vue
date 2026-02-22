<script setup lang="ts">
import { reactive, computed, watch } from 'vue'
import FormInput from '@/components/ui/FormInput.vue'
import FormSelect from '@/components/ui/FormSelect.vue'
import FormTextarea from '@/components/ui/FormTextarea.vue'
import Button from '@/components/ui/Button.vue'
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
      <FormInput
        v-model="form.project"
        label="Project"
        placeholder="my-app"
      />
      <FormInput
        v-model="form.name"
        label="Profile Name"
        placeholder="local-main"
      />
    </div>

    <div class="grid grid-cols-2 gap-4">
      <FormSelect
        v-model="form.dbType"
        label="Database Type"
        :options="dbTypeOptions"
      />
      <FormSelect
        v-model="form.environment"
        label="Environment"
        :options="environmentOptions"
      />
    </div>

    <div v-if="!isSqlite" class="grid grid-cols-2 gap-4">
      <FormInput
        v-model="form.host"
        label="Host"
        placeholder="127.0.0.1"
      />
      <FormInput
        v-model="form.port"
        label="Port"
        type="number"
        :placeholder="form.dbType === 'mysql' ? '3306' : '5432'"
      />
    </div>

    <FormInput
      v-model="form.databaseName"
      :label="isSqlite ? 'Database File Path' : 'Database Name'"
      :placeholder="isSqlite ? '/path/to/database.db' : 'my_database'"
    />

    <div v-if="!isSqlite" class="grid grid-cols-2 gap-4">
      <FormInput
        v-model="form.username"
        label="Username"
        placeholder="root"
      />
      <FormInput
        v-model="form.password"
        label="Password"
        type="password"
        placeholder="Enter password"
      />
    </div>

    <FormTextarea
      v-model="form.notes"
      label="Notes"
      placeholder="Optional notes about this connection"
      :rows="3"
    />

    <div class="border-t border-border-subtle pt-4">
      <SshTunnelConfig v-model="form.ssh" />
    </div>

    <!-- Sticky footer -->
    <div class="sticky bottom-0 -mx-6 mt-6 flex items-center justify-end gap-3 border-t border-border-subtle bg-surface-base/80 px-6 py-4 backdrop-blur-sm">
      <Button variant="ghost" size="md" @click="emit('cancel')">
        Cancel
      </Button>
      <Button variant="primary" size="md" type="submit" :loading="submitting">
        {{ initialData?.id ? 'Save Changes' : 'Create Profile' }}
      </Button>
    </div>
  </form>
</template>
