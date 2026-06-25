// 键盘事件 Hook
import { useEffect, useCallback } from "react";

interface UseKeyboardOptions {
  onEnter?: () => void;
  onEscape?: () => void;
  onArrowUp?: () => void;
  onArrowDown?: () => void;
  onDelete?: () => void;
}

/** 判断当前焦点是否在输入元素上 */
function isInputElement(el: EventTarget | null): boolean {
  if (!el || !(el instanceof HTMLElement)) return false;
  const tag = el.tagName.toLowerCase();
  return tag === "input" || tag === "textarea" || el.isContentEditable;
}

export function useKeyboard(handlers: UseKeyboardOptions) {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const inInput = isInputElement(event.target);

      switch (event.key) {
        case "Enter":
          if (!inInput) handlers.onEnter?.();
          break;
        case "Escape":
          handlers.onEscape?.();
          break;
        case "ArrowUp":
          if (inInput) return;
          event.preventDefault();
          handlers.onArrowUp?.();
          break;
        case "ArrowDown":
          if (inInput) return;
          event.preventDefault();
          handlers.onArrowDown?.();
          break;
        case "Delete":
        case "Backspace":
          if (inInput) return;
          if (event.metaKey) {
            handlers.onDelete?.();
          }
          break;
      }
    },
    [handlers]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}
