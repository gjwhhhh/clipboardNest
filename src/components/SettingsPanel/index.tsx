// 设置面板组件
import { useEffect, useState } from "react";
import { useSettingsStore } from "../../stores/settingsStore";
import { useClipboardStore } from "../../stores/clipboardStore";
import { X, Trash2 } from "lucide-react";

interface Props {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: Props) {
  const { settings, fetchSettings, updateSetting, updateHotkey } = useSettingsStore();
  const { clearAll, showToast } = useClipboardStore();
  const [isEditingHotkey, setIsEditingHotkey] = useState(false);
  const [draftHotkey, setDraftHotkey] = useState(settings.hotkey);
  const [isSavingHotkey, setIsSavingHotkey] = useState(false);

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  useEffect(() => {
    if (!isEditingHotkey) {
      setDraftHotkey(settings.hotkey);
    }
  }, [isEditingHotkey, settings.hotkey]);

  const handleClearAll = async () => {
    if (window.confirm("确定要清除所有历史记录吗？此操作不可撤销。")) {
      await clearAll();
    }
  };

  const handleSaveHotkey = async () => {
    const nextHotkey = draftHotkey.trim();
    if (!nextHotkey) {
      showToast("error", "快捷键不能为空");
      return;
    }

    setIsSavingHotkey(true);
    try {
      await updateHotkey(nextHotkey);
      setIsEditingHotkey(false);
      showToast("success", "快捷键已更新");
    } catch (error) {
      showToast("error", `快捷键更新失败: ${String(error)}`);
    } finally {
      setIsSavingHotkey(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-900 rounded-2xl shadow-2xl w-[380px] max-h-[85vh] overflow-hidden flex flex-col">
        {/* 标题栏 */}
        <div className="sticky top-0 z-10 bg-white dark:bg-gray-900 flex items-center justify-between px-5 py-4 border-b border-gray-100 dark:border-gray-800">
          <h2 className="text-base font-semibold text-gray-900 dark:text-gray-100">设置</h2>
          <button
            onClick={onClose}
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
          >
            <X className="w-4 h-4 text-gray-500 dark:text-gray-400" />
          </button>
        </div>

        {/* 内容区域 */}
        <div className="overflow-y-auto flex-1 px-5 py-4 space-y-5">
          {/* 全局快捷键 */}
          <div>
            <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-2 uppercase tracking-wide">
              全局快捷键
            </label>
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={isEditingHotkey ? draftHotkey : settings.hotkey}
                readOnly={!isEditingHotkey}
                onChange={(e) => setDraftHotkey(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && isEditingHotkey) {
                    handleSaveHotkey();
                  }
                  if (e.key === "Escape" && isEditingHotkey) {
                    setIsEditingHotkey(false);
                    setDraftHotkey(settings.hotkey);
                  }
                }}
                placeholder="例如 Ctrl+Shift+V"
                className="flex-1 px-3 py-2 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent transition-all"
              />
              {isEditingHotkey ? (
                <>
                  <button
                    onClick={handleSaveHotkey}
                    disabled={isSavingHotkey}
                    className="px-3 py-2 bg-primary-500 text-white rounded-lg text-sm hover:bg-primary-600 disabled:opacity-50 transition-colors"
                  >
                    保存
                  </button>
                  <button
                    onClick={() => {
                      setIsEditingHotkey(false);
                      setDraftHotkey(settings.hotkey);
                    }}
                    className="px-3 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg text-sm hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                  >
                    取消
                  </button>
                </>
              ) : (
                <button
                  onClick={() => {
                    setDraftHotkey(settings.hotkey);
                    setIsEditingHotkey(true);
                  }}
                  className="px-3 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg text-sm hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                >
                  更改
                </button>
              )}
            </div>
            {isEditingHotkey && (
              <p className="mt-2 text-xs text-gray-400 dark:text-gray-500">
                支持 Ctrl、Cmd、Shift、Alt 与字母键(A-Z)、数字键(0-9)、功能键(F1-F12)组合
              </p>
            )}
          </div>

          {/* 保留天数 */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                保留历史
              </label>
              <span className="text-sm font-semibold text-primary-500">{settings.retentionDays} 天</span>
            </div>
            <input
              type="range"
              min="7"
              max="365"
              value={settings.retentionDays}
              onChange={(e) => updateSetting("retentionDays", parseInt(e.target.value))}
              className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
            />
            <div className="flex justify-between text-xs text-gray-400 dark:text-gray-500 mt-1">
              <span>7天</span>
              <span>365天</span>
            </div>
          </div>

          {/* 最大项目数 */}
          <div>
            <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-2 uppercase tracking-wide">
              最大项目数
            </label>
            <select
              value={settings.maxItems}
              onChange={(e) => updateSetting("maxItems", parseInt(e.target.value))}
              className="w-full px-3 py-2 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent transition-all appearance-none cursor-pointer"
            >
              <option value={1000}>1,000 条</option>
              <option value={3000}>3,000 条</option>
              <option value={5000}>5,000 条</option>
              <option value={10000}>10,000 条</option>
            </select>
          </div>

          {/* 开机自启 */}
          <div className="flex items-center justify-between py-2">
            <div>
              <span className="text-sm font-medium text-gray-900 dark:text-gray-100">开机自启</span>
              <p className="text-xs text-gray-400 dark:text-gray-500 mt-0.5">系统启动时自动运行应用</p>
            </div>
            <button
              onClick={() => updateSetting("launchAtLogin", !settings.launchAtLogin)}
              className={`
                relative w-11 h-6 rounded-full transition-colors duration-200
                ${settings.launchAtLogin ? "bg-primary-500" : "bg-gray-300 dark:bg-gray-600"}
              `}
            >
              <div
                className={`
                  absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow-sm transition-transform duration-200
                  ${settings.launchAtLogin ? "translate-x-5" : "translate-x-0"}
                `}
              />
            </button>
          </div>

          {/* 主题 */}
          <div>
            <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-2 uppercase tracking-wide">
              主题
            </label>
            <select
              value={settings.theme}
              onChange={(e) => updateSetting("theme", e.target.value as "system" | "light" | "dark")}
              className="w-full px-3 py-2 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent transition-all appearance-none cursor-pointer"
            >
              <option value="system">跟随系统</option>
              <option value="light">浅色</option>
              <option value="dark">深色</option>
            </select>
          </div>

          {/* 清除所有 */}
          <div className="pt-4 border-t border-gray-100 dark:border-gray-800">
            <button
              onClick={handleClearAll}
              className="flex items-center gap-2 px-4 py-2.5 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-colors w-full justify-center text-sm font-medium"
            >
              <Trash2 className="w-4 h-4" />
              清除所有历史
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
