// 缩放手柄组件
import type { ResizeEdge } from "../../hooks/useWindowControls";

interface Props {
  onResizeStart: (edge: ResizeEdge) => void;
}

export function ResizeHandles({ onResizeStart }: Props) {
  return (
    <>
      {/* 边缘手柄 */}
      <div
        className="resize-handle resize-handle-top"
        onMouseDown={() => onResizeStart("top")}
      />
      <div
        className="resize-handle resize-handle-right"
        onMouseDown={() => onResizeStart("right")}
      />
      <div
        className="resize-handle resize-handle-bottom"
        onMouseDown={() => onResizeStart("bottom")}
      />
      <div
        className="resize-handle resize-handle-left"
        onMouseDown={() => onResizeStart("left")}
      />

      {/* 角落手柄 */}
      <div
        className="resize-handle resize-handle-top-left"
        onMouseDown={() => onResizeStart("top-left")}
      />
      <div
        className="resize-handle resize-handle-top-right"
        onMouseDown={() => onResizeStart("top-right")}
      />
      <div
        className="resize-handle resize-handle-bottom-left"
        onMouseDown={() => onResizeStart("bottom-left")}
      />
      <div
        className="resize-handle resize-handle-bottom-right"
        onMouseDown={() => onResizeStart("bottom-right")}
      />
    </>
  );
}
