import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import SnapshotRestoreDialog from "../SnapshotRestoreDialog.vue";
import type { Snapshot, Profile } from "@/types";

// Stub @stuntrocket/ui components to render plain HTML for testing
const SConfirmDialogStub = {
  template: `<div v-if="open"><slot /><button data-test="confirm" @click="$emit('confirm')">Confirm</button><button data-test="cancel" @click="$emit('cancel')">Cancel</button></div>`,
  props: ["open", "title", "message", "confirmLabel", "danger"],
  emits: ["confirm", "cancel", "close"],
};

const SSelectStub = {
  template:
    '<select :value="modelValue" @change="$emit(\'update:modelValue\', $event.target.value)"><slot /></select>',
  props: ["modelValue"],
  emits: ["update:modelValue"],
};

const SInputStub = {
  template:
    '<input type="text" :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
  props: ["modelValue", "placeholder"],
  emits: ["update:modelValue"],
};

const SFormFieldStub = {
  template: "<div><slot /></div>",
  props: ["label"],
};

const makeSnapshot = (overrides: Partial<Snapshot> = {}): Snapshot => ({
  id: "snap-1",
  profileId: "prof-1",
  databaseName: "myproject_dev",
  name: "Test Snapshot",
  note: null,
  filePath: "project/snap-1.sql.gz",
  sizeBytes: 2048,
  dbVersion: null,
  dumpToolVersion: null,
  checksum: null,
  restoreTestStatus: "passed",
  restoreTestMessage: "Local restore test passed.",
  restoreTestedAt: "2026-01-15T10:31:00Z",
  createdAt: "2026-01-15T10:30:00Z",
  restoredAt: null,
  pinned: false,
  tags: [],
  ...overrides,
});

const makeProfile = (overrides: Partial<Profile> = {}): Profile => ({
  id: "prof-1",
  project: "MyProject",
  name: "Local Dev",
  dbType: "mysql",
  host: "localhost",
  port: 3306,
  databaseName: "myproject_dev",
  databaseNames: ["myproject_dev"],
  username: "root",
  sshEnabled: false,
  sshHost: null,
  sshPort: 22,
  sshUser: null,
  environment: null,
  notes: null,
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
  ...overrides,
});

function mountDialog(props: Record<string, unknown> = {}) {
  return mount(SnapshotRestoreDialog, {
    props: {
      open: true,
      snapshot: makeSnapshot(),
      profiles: [
        makeProfile(),
        makeProfile({ id: "prof-2", name: "Staging", databaseName: "myproject_staging" }),
        makeProfile({ id: "prof-3", name: "PG Dev", dbType: "postgresql", databaseName: "pg_dev" }),
      ],
      sourceProfile: makeProfile(),
      ...props,
    },
    global: {
      stubs: {
        SConfirmDialog: SConfirmDialogStub,
        SSelect: SSelectStub,
        SInput: SInputStub,
        SFormField: SFormFieldStub,
      },
    },
  });
}

describe("SnapshotRestoreDialog", () => {
  it("only shows profiles with matching dbType in the selector", () => {
    const wrapper = mountDialog();
    const options = wrapper.findAll("select option");
    // Should have 2 mysql profiles, not the postgresql one
    expect(options).toHaveLength(2);
    expect(options.map((o) => o.text())).not.toContain(expect.stringContaining("PG Dev"));
  });

  it("emits confirm with empty options when defaults unchanged", async () => {
    const wrapper = mountDialog();
    await wrapper.find('[data-test="confirm"]').trigger("click");
    const emitted = wrapper.emitted("confirm");
    expect(emitted).toBeTruthy();
    expect(emitted![0][0]).toEqual({});
  });

  it("emits confirm with targetProfileId when different profile selected", async () => {
    const wrapper = mountDialog();
    const select = wrapper.find("select");
    await select.setValue("prof-2");
    await nextTick();
    await wrapper.find('[data-test="confirm"]').trigger("click");
    const emitted = wrapper.emitted("confirm");
    expect(emitted![0][0]).toEqual({ targetProfileId: "prof-2" });
  });

  it("shows database override input when checkbox is ticked", async () => {
    const wrapper = mountDialog();
    expect(wrapper.find('input[type="text"]').exists()).toBe(false);
    const checkbox = wrapper.find('input[type="checkbox"]');
    await checkbox.setValue(true);
    await nextTick();
    expect(wrapper.find('input[type="text"]').exists()).toBe(true);
  });

  it("emits confirm with targetDatabaseName when override is provided", async () => {
    const wrapper = mountDialog();
    const checkbox = wrapper.find('input[type="checkbox"]');
    await checkbox.setValue(true);
    await nextTick();
    const input = wrapper.find('input[type="text"]');
    await input.setValue("custom_db");
    await nextTick();
    await wrapper.find('[data-test="confirm"]').trigger("click");
    const emitted = wrapper.emitted("confirm");
    expect(emitted![0][0]).toEqual({ targetDatabaseName: "custom_db" });
  });
});
