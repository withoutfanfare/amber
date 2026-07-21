<script setup lang="ts">
  import { reactive, computed, ref, watch } from "vue";
  import { SFormField, SInput, SSelect, STextarea, SButton } from "@stuntrocket/ui";
  import SshTunnelConfig from "./SshTunnelConfig.vue";
  import type { Profile, DbType, Environment, ProfileCreatePayload } from "@/types";

  const props = withDefaults(
    defineProps<{
      initialData?: Partial<Profile>;
      submitting?: boolean;
    }>(),
    {
      initialData: undefined,
      submitting: false,
    },
  );

  const emit = defineEmits<{
    submit: [payload: ProfileCreatePayload];
    cancel: [];
  }>();

  const dbTypeOptions = [
    { value: "mysql", label: "MySQL" },
    { value: "postgresql", label: "PostgreSQL" },
    { value: "sqlite", label: "SQLite" },
  ];

  const environmentOptions = [
    { value: "", label: "Not set" },
    { value: "local", label: "Local" },
    { value: "staging", label: "Staging" },
    { value: "live", label: "Live" },
  ];

  const form = reactive({
    project: props.initialData?.project ?? "",
    name: props.initialData?.name ?? "",
    dbType: (props.initialData?.dbType ?? "mysql") as DbType,
    host: props.initialData?.host ?? "",
    port: props.initialData?.port ?? 3306,
    databaseNames: [
      ...(props.initialData?.databaseNames?.length
        ? props.initialData.databaseNames
        : [props.initialData?.databaseName ?? ""]),
    ],
    username: props.initialData?.username ?? "",
    password: "",
    environment: (props.initialData?.environment ?? "") as Environment | "",
    notes: props.initialData?.notes ?? "",
    ssh: {
      enabled: props.initialData?.sshEnabled ?? false,
      host: props.initialData?.sshHost ?? "",
      port: props.initialData?.sshPort ?? 22,
      user: props.initialData?.sshUser ?? "",
      keyPath: "",
      password: "",
    },
  });

  const isSqlite = computed(() => form.dbType === "sqlite");
  const showErrors = ref(false);

  const projectError = computed(() =>
    showErrors.value && !form.project.trim() ? "Enter a project name." : undefined,
  );
  const nameError = computed(() =>
    showErrors.value && !form.name.trim() ? "Enter a profile name." : undefined,
  );

  function databaseError(index: number) {
    if (!showErrors.value) return undefined;
    const name = form.databaseNames[index].trim();
    if (!name) return isSqlite.value ? "Enter a database file path." : "Enter a database name.";
    if (form.databaseNames.findIndex((item) => item.trim() === name) !== index) {
      return "Database names must be unique.";
    }
    return undefined;
  }

  // Set sensible default port when DB type changes
  watch(
    () => form.dbType,
    (newType) => {
      if (newType === "mysql") form.port = 3306;
      else if (newType === "postgresql") form.port = 5432;
      else form.port = 0;
    },
  );

  function handleSubmit() {
    showErrors.value = true;
    if (
      !form.project.trim() ||
      !form.name.trim() ||
      form.databaseNames.some((_, index) => databaseError(index))
    )
      return;

    const payload: ProfileCreatePayload = {
      project: form.project.trim(),
      name: form.name.trim(),
      dbType: form.dbType,
      databaseNames: form.databaseNames.map((name) => name.trim()),
      ...(isSqlite.value
        ? {}
        : {
            host: form.host || undefined,
            port: form.port || undefined,
            username: form.username || undefined,
          }),
      ...(form.password ? { password: form.password } : {}),
      ...(form.environment ? { environment: form.environment as Environment } : {}),
      ...(form.notes ? { notes: form.notes } : {}),
      ...(form.ssh.enabled
        ? {
            sshEnabled: true,
            sshHost: form.ssh.host || undefined,
            sshPort: form.ssh.port || undefined,
            sshUser: form.ssh.user || undefined,
            sshKeyPath: form.ssh.keyPath || undefined,
            sshPassword: form.ssh.password || undefined,
          }
        : { sshEnabled: false }),
    };
    emit("submit", payload);
  }
</script>

<template>
  <form class="space-y-5" @submit.prevent="handleSubmit">
    <div class="grid grid-cols-2 gap-4">
      <SFormField label="Project" required :error="projectError">
        <SInput v-model="form.project" data-test="project" placeholder="my-app" />
      </SFormField>
      <SFormField label="Profile Name" required :error="nameError">
        <SInput v-model="form.name" data-test="profile-name" placeholder="local" />
      </SFormField>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <SFormField label="Database Type">
        <SSelect v-model="form.dbType">
          <option v-for="opt in dbTypeOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </SSelect>
      </SFormField>
      <SFormField label="Environment">
        <SSelect v-model="form.environment">
          <option v-for="opt in environmentOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </SSelect>
      </SFormField>
    </div>

    <div v-if="!isSqlite" class="grid grid-cols-2 gap-4">
      <SFormField label="Host">
        <SInput v-model="form.host" placeholder="127.0.0.1" />
      </SFormField>
      <SFormField label="Port">
        <SInput
          v-model="form.port"
          type="number"
          :placeholder="form.dbType === 'mysql' ? '3306' : '5432'"
        />
      </SFormField>
    </div>

    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-[13px] font-medium text-text-primary">
            {{ isSqlite ? "Database Files" : "Databases" }}
            <span class="text-danger">*</span>
          </p>
          <p class="text-xs text-text-tertiary">
            Every database in this profile is captured when you create a snapshot.
          </p>
        </div>
        <SButton variant="secondary" size="sm" @click="form.databaseNames.push('')">
          Add Database
        </SButton>
      </div>

      <div v-for="(_, index) in form.databaseNames" :key="index" class="flex items-start gap-2">
        <SFormField
          class="min-w-0 flex-1"
          :label="form.databaseNames.length > 1 ? `Database ${index + 1}` : undefined"
          :error="databaseError(index)"
        >
          <SInput
            v-model="form.databaseNames[index]"
            :data-test="`database-${index}`"
            :placeholder="isSqlite ? '/path/to/database.db' : 'my_database'"
          />
        </SFormField>
        <SButton
          v-if="form.databaseNames.length > 1"
          variant="ghost"
          size="sm"
          class="mt-6 text-danger hover:text-danger"
          :aria-label="`Remove database ${index + 1}`"
          @click="form.databaseNames.splice(index, 1)"
        >
          Remove
        </SButton>
      </div>
    </div>

    <div v-if="!isSqlite" class="grid grid-cols-2 gap-4">
      <SFormField label="Username">
        <SInput v-model="form.username" placeholder="root" />
      </SFormField>
      <SFormField label="Password">
        <SInput v-model="form.password" type="password" placeholder="Enter password" />
      </SFormField>
    </div>

    <SFormField label="Notes">
      <STextarea
        v-model="form.notes"
        placeholder="Optional notes about this connection"
        :rows="3"
      />
    </SFormField>

    <div class="border-t border-border-subtle pt-4">
      <SshTunnelConfig v-model="form.ssh" />
    </div>

    <!-- Sticky footer -->
    <div
      class="sticky bottom-0 -mx-6 mt-6 flex items-center justify-end gap-3 border-t border-border-subtle bg-surface-base/80 px-6 py-4 backdrop-blur-sm"
    >
      <SButton variant="ghost" @click="emit('cancel')"> Cancel </SButton>
      <SButton
        data-test="profile-submit"
        variant="primary"
        :loading="submitting"
        @click="handleSubmit"
      >
        {{ initialData?.id ? "Save Changes" : "Create Profile" }}
      </SButton>
    </div>
  </form>
</template>
