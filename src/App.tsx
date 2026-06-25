// 主应用组件 - Alfred 风格左右分栏布局
import { useState } from "react";
import { SearchBar } from "./components/SearchBar";
import { ClipboardList } from "./components/ClipboardList";
import { PreviewPanel } from "./components/PreviewPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import { Toast } from "./components/Toast";
import { ResizeHandles } from "./components/ResizeHandles";
import { useClipboardStore } from "./stores/clipboardStore";
import { useClipboardRefresh } from "./hooks/useClipboardRefresh";
import { useWindowControls } from "./hooks/useWindowControls";
import { Settings } from "lucide-react";

function App() {
  useClipboardRefresh();
  const { toggleMaximize, startDragging, startResize } = useWindowControls();

  const [showSettings, setShowSettings] = useState(false);
  const { toast, hideToast } = useClipboardStore();

  return (
    <div className="h-screen flex flex-col glass-effect rounded-xl overflow-hidden relative">
      {/* 头部 - 可拖动区域 */}
      <div
        className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700 cursor-grab active:cursor-grabbing"
        onMouseDown={startDragging}
        onDoubleClick={toggleMaximize}
      >
        <h1 className="text-sm font-semibold text-gray-700 dark:text-gray-300 select-none">
          剪切板管理器
        </h1>
        <button
          onClick={() => setShowSettings(true)}
          className="p-1.5 hover:bg-gray-200 dark:hover:bg-gray-700 rounded"
        >
          <Settings className="w-4 h-4" />
        </button>
      </div>

      {/* 搜索 */}
      <SearchBar />

      {/* 左右分栏：列表 + 预览 */}
      <div className="flex flex-1 overflow-hidden">
        {/* 左侧列表 */}
        <div className="w-[320px] border-r border-gray-200 dark:border-gray-700 flex flex-col overflow-hidden">
          <ClipboardList />
        </div>

        {/* 右侧预览 */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <PreviewPanel />
        </div>
      </div>

      {/* 设置弹窗 */}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}

      {/* Toast 提示 */}
      <Toast toast={toast} onClose={hideToast} />

      {/* 缩放手柄 */}
      <ResizeHandles onResizeStart={startResize} />
    </div>
  );
}

export default App;
