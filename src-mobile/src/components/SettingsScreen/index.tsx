import { useEffect } from "react";
import { useSettingsStore } from "../../stores/settingsStore";
import { useClipboardStore } from "../../stores/clipboardStore";
import { useTranslation } from "../../hooks/useTranslation";
import { ChevronLeft, Trash2, Wifi } from "lucide-react";

interface Props {
  onBack: () => void;
  onOpenSync?: () => void;
}

export function SettingsScreen({ onBack, onOpenSync }: Props) {
  const { settings, fetchSettings, updateSetting } = useSettingsStore();
  const { clearAll } = useClipboardStore();
  const { t, changeLanguage, currentLanguage } = useTranslation();

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  const handleClearAll = async () => {
    if (window.confirm(t("settings.clearConfirm"))) {
      await clearAll();
    }
  };

  return (
    <div className="min-h-screen bg-[rgb(var(--bg-primary))] safe-top">
      {/* 头部 */}
      <div className="flex items-center gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <button onClick={onBack} className="p-2">
          <ChevronLeft className="w-5 h-5" />
        </button>
        <h1 className="text-lg font-semibold">{t("settings.title")}</h1>
      </div>

      {/* 内容 */}
      <div className="p-4 space-y-6">
        {/* 同步设置 */}
        {onOpenSync && (
          <button
            onClick={onOpenSync}
            className="w-full flex items-center justify-between p-3 bg-gray-100 dark:bg-gray-800 rounded-xl"
          >
            <div className="flex items-center gap-3">
              <Wifi className="w-5 h-5 text-primary-500" />
              <span className="font-medium">{t("sync.title")}</span>
            </div>
            <ChevronLeft className="w-5 h-5 rotate-180" />
          </button>
        )}

        {/* 语言设置 */}
        <div>
          <label className="block text-sm font-medium mb-2">{t("settings.language")}</label>
          <select
            value={currentLanguage}
            onChange={(e) => changeLanguage(e.target.value)}
            className="w-full px-3 py-2.5 bg-gray-100 dark:bg-gray-800 rounded-xl text-sm"
          >
            <option value="zh">{t("settings.languageZh")}</option>
            <option value="en">{t("settings.languageEn")}</option>
          </select>
        </div>

        {/* 保留天数 */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <label className="text-sm font-medium">{t("settings.retentionDays")}</label>
            <span className="text-sm text-primary-500">
              {settings.retentionDays} {t("settings.days")}
            </span>
          </div>
          <input
            type="range"
            min="7"
            max="365"
            value={settings.retentionDays}
            onChange={(e) => updateSetting("retentionDays", parseInt(e.target.value))}
            className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
          />
          <div className="flex justify-between text-xs text-gray-500 mt-1">
            <span>7{t("settings.days")}</span>
            <span>365{t("settings.days")}</span>
          </div>
        </div>

        {/* 最大项目数 */}
        <div>
          <label className="block text-sm font-medium mb-2">{t("settings.maxItems")}</label>
          <select
            value={settings.maxItems}
            onChange={(e) => updateSetting("maxItems", parseInt(e.target.value))}
            className="w-full px-3 py-2.5 bg-gray-100 dark:bg-gray-800 rounded-xl text-sm"
          >
            <option value={1000}>1,000</option>
            <option value={3000}>3,000</option>
            <option value={5000}>5,000</option>
            <option value={10000}>10,000</option>
          </select>
        </div>

        {/* 主题 */}
        <div>
          <label className="block text-sm font-medium mb-2">{t("settings.theme")}</label>
          <select
            value={settings.theme}
            onChange={(e) => updateSetting("theme", e.target.value as "system" | "light" | "dark")}
            className="w-full px-3 py-2.5 bg-gray-100 dark:bg-gray-800 rounded-xl text-sm"
          >
            <option value="system">{t("settings.themeSystem")}</option>
            <option value="light">{t("settings.themeLight")}</option>
            <option value="dark">{t("settings.themeDark")}</option>
          </select>
        </div>

        {/* 清除所有 */}
        <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
          <button
            onClick={handleClearAll}
            className="flex items-center justify-center gap-2 w-full px-4 py-3 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-xl"
          >
            <Trash2 className="w-4 h-4" />
            {t("settings.clearAll")}
          </button>
        </div>
      </div>
    </div>
  );
}
