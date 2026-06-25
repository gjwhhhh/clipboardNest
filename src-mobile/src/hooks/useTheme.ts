import { useEffect } from "react";
import { useSettingsStore } from "../stores/settingsStore";

export function useTheme() {
  const { settings } = useSettingsStore();

  useEffect(() => {
    const root = document.documentElement;
    root.classList.remove("light", "dark");

    if (settings.theme === "system") {
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const applySystemTheme = (e: MediaQueryListEvent | MediaQueryList) => {
        root.classList.toggle("dark", e.matches);
      };

      applySystemTheme(mediaQuery);
      mediaQuery.addEventListener("change", applySystemTheme);

      return () => {
        mediaQuery.removeEventListener("change", applySystemTheme);
      };
    } else {
      root.classList.add(settings.theme);
    }
  }, [settings.theme]);
}
