import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useClipboardStore } from "../stores/clipboardStore";

export function useClipboardRefresh() {
  useEffect(() => {
    const refresh = () => {
      useClipboardStore.getState().fetchItems();
    };

    refresh();

    const unlistenClipboardUpdated = listen("clipboard-updated", refresh);
    const unlistenWindowShown = listen("window-shown", refresh);
    const unlistenFocusChanged = getCurrentWindow().onFocusChanged(({ payload }) => {
      if (payload) {
        refresh();
      }
    });

    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        refresh();
      }
    };
    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      unlistenClipboardUpdated.then((unlisten) => unlisten());
      unlistenWindowShown.then((unlisten) => unlisten());
      unlistenFocusChanged.then((unlisten) => unlisten());
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);
}
