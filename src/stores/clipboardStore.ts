// 剪切板状态管理
import { create } from "zustand";
import type { ClipboardItem, FilterType } from "../types";
import type { ToastMessage } from "../components/Toast";
import * as api from "../utils/tauri";

interface ClipboardState {
  items: ClipboardItem[];
  filteredItems: ClipboardItem[];
  selectedItem: ClipboardItem | null;
  hoveredItem: ClipboardItem | null;
  searchQuery: string;
  activeFilter: FilterType;
  isLoading: boolean;
  error: string | null;
  toast: ToastMessage | null;

  // 操作
  fetchItems: () => Promise<void>;
  search: (query: string) => Promise<void>;
  setFilter: (filter: FilterType) => void;
  selectItem: (item: ClipboardItem | null) => void;
  setHoveredItem: (item: ClipboardItem | null) => void;
  copyItem: (item: ClipboardItem) => Promise<void>;
  deleteItem: (id: number) => Promise<void>;
  pinItem: (id: number, pinned: boolean) => Promise<void>;
  favoriteItem: (id: number, favorite: boolean) => Promise<void>;
  clearAll: () => Promise<void>;
  showToast: (type: "success" | "error", message: string) => void;
  hideToast: () => void;
}

let toastCounter = 0;

export const useClipboardStore = create<ClipboardState>((set, get) => ({
  items: [],
  filteredItems: [],
  selectedItem: null,
  hoveredItem: null,
  searchQuery: "",
  activeFilter: "all",
  isLoading: false,
  error: null,
  toast: null,

  fetchItems: async () => {
    set({ isLoading: true, error: null });
    try {
      const { activeFilter } = get();
      const filter = activeFilter === "all" ? undefined : activeFilter;
      const items = await api.getClipboardHistory(filter);
      set({ items, filteredItems: items, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  search: async (query: string) => {
    set({ searchQuery: query, isLoading: true });
    try {
      if (query.trim() === "") {
        await get().fetchItems();
      } else {
        const items = await api.searchClipboard(query);
        set({ filteredItems: items, isLoading: false });
      }
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  setFilter: async (filter: FilterType) => {
    set({ activeFilter: filter });
    await get().fetchItems();
  },

  selectItem: (item: ClipboardItem | null) => {
    set({ selectedItem: item });
  },

  setHoveredItem: (item: ClipboardItem | null) => {
    set({ hoveredItem: item });
  },

  copyItem: async (item: ClipboardItem) => {
    try {
      await api.copyToClipboard(item.id);
      get().showToast("success", "已复制到剪切板");
    } catch (error) {
      get().showToast("error", "复制失败");
      throw error; // 重新抛出，让调用方知道失败了
    }
  },

  deleteItem: async (id: number) => {
    try {
      await api.deleteClipboardItem(id);
      await get().fetchItems();
      get().showToast("success", "已删除");
    } catch (error) {
      get().showToast("error", "删除失败");
    }
  },

  pinItem: async (id: number, pinned: boolean) => {
    try {
      await api.pinClipboardItem(id, pinned);
      await get().fetchItems();
      get().showToast("success", pinned ? "已置顶" : "已取消置顶");
    } catch (error) {
      get().showToast("error", "置顶失败");
    }
  },

  favoriteItem: async (id: number, favorite: boolean) => {
    try {
      await api.favoriteClipboardItem(id, favorite);
      await get().fetchItems();
      get().showToast("success", favorite ? "已收藏" : "已取消收藏");
    } catch (error) {
      get().showToast("error", "收藏失败");
    }
  },

  clearAll: async () => {
    try {
      await api.clearAllHistory();
      set({ items: [], filteredItems: [], selectedItem: null });
      get().showToast("success", "已清除所有历史");
    } catch (error) {
      get().showToast("error", "清除失败");
    }
  },

  showToast: (type, message) => {
    const id = `toast_${++toastCounter}`;
    set({ toast: { id, type, message } });
  },
  hideToast: () => {
    set({ toast: null });
  },
}));
