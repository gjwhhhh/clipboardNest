import { useEffect, useState } from "react";

type AppState = "active" | "background" | "inactive";

export function useAppState() {
  const [appState, setAppState] = useState<AppState>("active");

  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        setAppState("active");
      } else {
        setAppState("background");
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  return appState;
}
