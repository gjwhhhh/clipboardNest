import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useClipboardStore } from "../stores/clipboardStore";

export function useClipboardRefresh() {
  useEffect(() => {
    const refresh = () => {
      useClipboardStore.getState().fetchItems();
    };

    // 初始加载
    refresh();

    // 监听剪切板更新事件
    const unlistenClipboardUpdated = listen("clipboard-updated", refresh);

    return () => {
      unlistenClipboardUpdated.then((unlisten) => unlisten());
    };
  }, []);
}
