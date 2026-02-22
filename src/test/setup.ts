import { vi, afterEach } from "vitest";
import { config } from "@vue/test-utils";

// Mock Tauri IPC globally
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  Channel: vi.fn().mockImplementation(function (this: { onmessage: null }) { this.onmessage = null }),
}));

// Provide default stubs for router-link
config.global.stubs = {
  RouterLink: {
    template: "<a><slot /></a>",
  },
};

afterEach(() => {
  vi.restoreAllMocks();
});
