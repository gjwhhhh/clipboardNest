import { create } from "zustand";
import type { Settings } from "../types";
import * as api from "../utils/tauri";

interface SettingsState {
  settings: Settings;
  isLoading: boolean;
  error: string | null;

  fetchSettings: () => Promise<void>;
  updateSetting: <K extends keyof Settings>(key: K, value: Settings[K]) => Promise<void>;
}

const defaultSettings: Settings = {
  retentionDays: 30,
  maxItems: 5000,
  pollIntervalMs: 500,
  theme: "system",
};

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: defaultSettings,
  isLoading: false,
  error: null,

  fetchSettings: async () => {
    set({ isLoading: true });
    try {
      const raw = await api.getSettings();
      const settings: Settings = {
        retentionDays: parseInt(raw.retention_days || "30"),
        maxItems: parseInt(raw.max_items || "5000"),
        pollIntervalMs: parseInt(raw.poll_interval_ms || "500"),
        theme: (raw.theme as Settings["theme"]) || "system",
      };
      set({ settings, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  updateSetting: async (key, value) => {
    try {
      const snakeKey = key.replace(/([A-Z])/g, "_$1").toLowerCase();
      await api.updateSetting(snakeKey, String(value));
      set((state) => ({
        settings: { ...state.settings, [key]: value },
      }));
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
