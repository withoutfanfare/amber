import { ref, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { useProfileStore } from "@/stores/profiles";

export function useKeyboardShortcuts() {
  const router = useRouter();
  const profileStore = useProfileStore();
  const helpOverlayOpen = ref(false);

  const shortcuts = [
    { keys: "Cmd+N", description: "Create new snapshot" },
    { keys: "Cmd+1-9", description: "Switch between profiles" },
    { keys: "Cmd+F", description: "Focus search/filter input" },
    { keys: "Cmd+/", description: "Show this help overlay" },
    { keys: "Cmd+,", description: "Open settings" },
  ];

  function handleKeydown(e: KeyboardEvent) {
    const meta = e.metaKey || e.ctrlKey;

    // Ignore if typing in an input
    const target = e.target as HTMLElement;
    if (
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.tagName === "SELECT" ||
      target.isContentEditable
    ) {
      // Allow Cmd+/ even in inputs for help overlay
      if (!(meta && e.key === "/")) {
        return;
      }
    }

    // Cmd+N — Create new snapshot
    if (meta && e.key === "n") {
      e.preventDefault();
      router.push("/snapshots/create");
      return;
    }

    // Cmd+/ — Toggle help overlay
    if (meta && e.key === "/") {
      e.preventDefault();
      helpOverlayOpen.value = !helpOverlayOpen.value;
      return;
    }

    // Cmd+, — Open settings
    if (meta && e.key === ",") {
      e.preventDefault();
      router.push("/settings");
      return;
    }

    // Cmd+F — Focus search input (handled by individual views, but as a fallback)
    if (meta && e.key === "f") {
      e.preventDefault();
      const searchInput = document.getElementById("snapshot-search");
      if (searchInput) {
        searchInput.focus();
      } else {
        // Navigate to snapshots view which has search
        router.push("/snapshots");
      }
      return;
    }

    // Cmd+1 through Cmd+9 — Switch profiles
    if (meta && e.key >= "1" && e.key <= "9") {
      e.preventDefault();
      const index = parseInt(e.key, 10) - 1;
      if (index < profileStore.profiles.length) {
        profileStore.setActive(profileStore.profiles[index].id);
      }
      return;
    }

    // Escape — Close help overlay
    if (e.key === "Escape") {
      if (helpOverlayOpen.value) {
        helpOverlayOpen.value = false;
      }
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", handleKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown);
  });

  return {
    helpOverlayOpen,
    shortcuts,
  };
}
