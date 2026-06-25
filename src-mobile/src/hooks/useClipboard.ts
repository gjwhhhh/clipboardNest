import { useEffect } from "react";
import { useClipboardStore } from "../stores/clipboardStore";
import { useDebounce } from "./useDebounce";

export function useClipboard() {
  const store = useClipboardStore();
  const debouncedSearch = useDebounce(store.searchQuery, 300);

  useEffect(() => {
    if (debouncedSearch) {
      store.search(debouncedSearch);
    } else {
      store.fetchItems();
    }
  }, [debouncedSearch, store.activeFilter]);

  return store;
}
