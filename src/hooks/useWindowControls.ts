// 窗口控制 Hook
import { useState, useEffect, useCallback, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";

export type ResizeEdge =
  | "right"
  | "bottom"
  | "left"
  | "top"
  | "top-right"
  | "top-left"
  | "bottom-right"
  | "bottom-left";

const MIN_WIDTH = 500;
const MIN_HEIGHT = 400;

export function useWindowControls() {
  const [isMaximized, setIsMaximized] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const resizeStateRef = useRef<{
    edge: ResizeEdge;
    startX: number;
    startY: number;
    startWidth: number;
    startHeight: number;
    startPosX: number;
    startPosY: number;
  } | null>(null);

  // 监听窗口状态变化
  useEffect(() => {
    const window = getCurrentWindow();

    // 初始状态
    window.isMaximized().then(setIsMaximized);

    // 监听窗口大小变化
    const unlisten = window.onResized(() => {
      window.isMaximized().then(setIsMaximized);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // 切换最大化/还原
  const toggleMaximize = useCallback(async () => {
    const window = getCurrentWindow();
    await window.toggleMaximize();
  }, []);

  // 启动拖动
  const startDragging = useCallback(async (e: React.MouseEvent) => {
    // 如果点击的是按钮，不启动拖动
    if ((e.target as HTMLElement).closest("button")) {
      return;
    }

    const window = getCurrentWindow();
    setIsDragging(true);

    try {
      // 使用 Tauri 2.0 的 startDragging API
      await window.startDragging();
    } catch (err) {
      console.error("拖动失败:", err);
    } finally {
      setIsDragging(false);
    }
  }, []);

  // 启动缩放
  const startResize = useCallback((edge: ResizeEdge) => {
    const window = getCurrentWindow();

    // 获取初始状态
    Promise.all([
      window.innerSize(),
      window.outerPosition(),
      window.scaleFactor(),
    ]).then(([size, pos, scale]) => {
      const startWidth = size.width / scale;
      const startHeight = size.height / scale;
      const startPosX = pos.x;
      const startPosY = pos.y;

      resizeStateRef.current = {
        edge,
        startX: 0,
        startY: 0,
        startWidth,
        startHeight,
        startPosX,
        startPosY,
      };
      setIsResizing(true);

      // 监听鼠标移动
      const handleMouseMove = (e: MouseEvent) => {
        const state = resizeStateRef.current;
        if (!state) return;

        const deltaX = e.clientX - state.startX;
        const deltaY = e.clientY - state.startY;

        let newWidth = state.startWidth;
        let newHeight = state.startHeight;
        let newPosX = state.startPosX;
        let newPosY = state.startPosY;

        // 根据边缘方向计算新尺寸
        if (state.edge.includes("right")) {
          newWidth = Math.max(MIN_WIDTH, state.startWidth + deltaX);
        }
        if (state.edge.includes("left")) {
          newWidth = Math.max(MIN_WIDTH, state.startWidth - deltaX);
          newPosX = state.startPosX + (state.startWidth - newWidth);
        }
        if (state.edge.includes("bottom")) {
          newHeight = Math.max(MIN_HEIGHT, state.startHeight + deltaY);
        }
        if (state.edge.includes("top")) {
          newHeight = Math.max(MIN_HEIGHT, state.startHeight - deltaY);
          newPosY = state.startPosY + (state.startHeight - newHeight);
        }

        // 应用新尺寸和位置
        window.setSize(new LogicalSize(newWidth, newHeight));
        if (state.edge.includes("left") || state.edge.includes("top")) {
          window.setPosition(new LogicalPosition(newPosX, newPosY));
        }
      };

      // 监听鼠标释放
      const handleMouseUp = () => {
        resizeStateRef.current = null;
        setIsResizing(false);
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      };

      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);

      // 设置缩放时的光标
      const cursorMap: Record<ResizeEdge, string> = {
        top: "ns-resize",
        bottom: "ns-resize",
        left: "ew-resize",
        right: "ew-resize",
        "top-left": "nwse-resize",
        "top-right": "nesw-resize",
        "bottom-left": "nesw-resize",
        "bottom-right": "nwse-resize",
      };
      document.body.style.cursor = cursorMap[edge];
      document.body.style.userSelect = "none";
    });
  }, []);

  return {
    isMaximized,
    isResizing,
    isDragging,
    toggleMaximize,
    startDragging,
    startResize,
  };
}
