import { useState, useEffect } from "react";
import { SearchBar } from "./components/SearchBar";
import { ClipboardList } from "./components/ClipboardList";
import { PreviewPanel } from "./components/PreviewPanel";
import { SettingsScreen } from "./components/SettingsScreen";
import { SyncScreen } from "./components/SyncScreen";
import { Toast } from "./components/common/Toast";
import { PermissionGuide } from "./components/common/PermissionGuide";
import { useClipboardStore } from "./stores/clipboardStore";
import { useClipboardRefresh } from "./hooks/useClipboardRefresh";
import { useTheme } from "./hooks/useTheme";
import { useAppState } from "./hooks/useAppState";
import { useTranslation } from "./hooks/useTranslation";
import { Settings, Search, Star, Clock } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

type Tab = "home" | "search" | "favorites" | "settings";
type Page = Tab | "sync";

function App() {
  useTheme();
  useClipboardRefresh();
  const appState = useAppState();
  const { t } = useTranslation();

  const [activeTab, setActiveTab] = useState<Tab>("home");
  const [currentPage, setCurrentPage] = useState<Page>("home");
  const [showPermissionGuide, setShowPermissionGuide] = useState(false);
  const { toast, hideToast } = useClipboardStore();

  // 根据应用状态控制剪切板监控
  useEffect(() => {
    if (appState === "active") {
      invoke("plugin:clipboard-monitor|start_monitoring").catch(console.error);
    } else {
      invoke("plugin:clipboard-monitor|stop_monitoring").catch(console.error);
    }
  }, [appState]);

  // 检查是否需要显示权限引导
  useEffect(() => {
    const hasShown = localStorage.getItem("permission_guide_shown");
    if (!hasShown) {
      setShowPermissionGuide(true);
    }
  }, []);

  // 处理标签切换
  const handleTabChange = (tab: Tab) => {
    setActiveTab(tab);
    setCurrentPage(tab);
  };

  // 如果在同步页面
  if (currentPage === "sync") {
    return (
      <div className="min-h-screen">
        <SyncScreen onBack={() => setCurrentPage("settings")} />
        <Toast toast={toast} onClose={hideToast} />
      </div>
    );
  }

  // 如果在设置页面
  if (currentPage === "settings") {
    return (
      <div className="min-h-screen">
        <SettingsScreen
          onBack={() => handleTabChange("home")}
          onOpenSync={() => setCurrentPage("sync")}
        />
        <Toast toast={toast} onClose={hideToast} />
        {showPermissionGuide && (
          <PermissionGuide onDismiss={() => setShowPermissionGuide(false)} />
        )}
      </div>
    );
  }

  // 底部导航栏按钮配置
  const navItems = [
    { tab: "home" as Tab, icon: Clock, label: t("tabs.history") },
    { tab: "search" as Tab, icon: Search, label: t("tabs.search") },
    { tab: "favorites" as Tab, icon: Star, label: t("tabs.favorites") },
    { tab: "settings" as Tab, icon: Settings, label: t("tabs.settings") },
  ];

  return (
    <div className="min-h-screen flex flex-col safe-top">
      {/* 头部 */}
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h1 className="text-lg font-semibold">{t("app.title")}</h1>
      </div>

      {/* 搜索栏 */}
      <SearchBar />

      {/* 列表 */}
      <div className="flex-1 overflow-hidden">
        <ClipboardList />
      </div>

      {/* 底部导航栏 */}
      <div className="flex items-center justify-around py-2 border-t border-gray-200 dark:border-gray-700 safe-bottom bg-white dark:bg-gray-900">
        {navItems.map((item) => (
          <button
            key={item.tab}
            onClick={() => handleTabChange(item.tab)}
            className={`flex flex-col items-center gap-1 p-2 ${
              activeTab === item.tab ? "text-primary-500" : "text-gray-500"
            }`}
          >
            <item.icon className="w-5 h-5" />
            <span className="text-xs">{item.label}</span>
          </button>
        ))}
      </div>

      {/* 预览面板 */}
      <PreviewPanel />

      {/* Toast */}
      <Toast toast={toast} onClose={hideToast} />

      {/* 权限引导 */}
      {showPermissionGuide && (
        <PermissionGuide onDismiss={() => setShowPermissionGuide(false)} />
      )}
    </div>
  );
}

export default App;
