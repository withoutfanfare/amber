import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ProfileForm from "../ProfileForm.vue";

const SInputStub = {
  inheritAttrs: false,
  props: ["modelValue"],
  emits: ["update:modelValue"],
  template:
    '<input v-bind="$attrs" :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
};

const SButtonStub = {
  inheritAttrs: false,
  emits: ["click"],
  template: '<button v-bind="$attrs" type="button" @click="$emit(\'click\')"><slot /></button>',
};

const SFormFieldStub = {
  props: ["error"],
  template: '<div><slot /><span v-if="error">{{ error }}</span></div>',
};

function mountForm() {
  return mount(ProfileForm, {
    global: {
      stubs: {
        SInput: SInputStub,
        SButton: SButtonStub,
        SFormField: SFormFieldStub,
        SSelect: { template: "<select><slot /></select>" },
        STextarea: { template: "<textarea />" },
        SshTunnelConfig: { template: "<div />" },
      },
    },
  });
}

describe("ProfileForm", () => {
  it("shows validation and submits every configured database", async () => {
    const wrapper = mountForm();

    await wrapper.get('[data-test="profile-submit"]').trigger("click");
    expect(wrapper.text()).toContain("Enter a project name.");
    expect(wrapper.emitted("submit")).toBeUndefined();

    await wrapper.get('[data-test="project"]').setValue("Scooda");
    await wrapper.get('[data-test="profile-name"]').setValue("Local");
    await wrapper.get('[data-test="database-0"]').setValue("scooda_landlord");
    await wrapper.get("button:nth-of-type(1)").trigger("click");
    await wrapper.get('[data-test="database-1"]').setValue("scooda_tenant_1");
    await wrapper.get('[data-test="profile-submit"]').trigger("click");

    expect(wrapper.emitted("submit")?.[0]?.[0]).toMatchObject({
      project: "Scooda",
      name: "Local",
      databaseNames: ["scooda_landlord", "scooda_tenant_1"],
    });
  });
});
