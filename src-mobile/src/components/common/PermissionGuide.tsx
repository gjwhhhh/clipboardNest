import { useState, useEffect } from "react";
import { Info, X } from "lucide-react";
import { useTranslation } from "../../hooks/useTranslation";

interface Props {
  onDismiss: () => void;
}

export function PermissionGuide({ onDismiss }: Props) {
  const [isVisible, setIsVisible] = useState(false);
  const { t } = useTranslation();

  useEffect(() => {
    const hasShown = localStorage.getItem("permission_guide_shown");
    if (!hasShown) {
      setIsVisible(true);
    }
  }, []);

  const handleDismiss = () => {
    setIsVisible(false);
    localStorage.setItem("permission_guide_shown", "true");
    onDismiss();
  };

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4">
      <div className="bg-white dark:bg-gray-900 rounded-2xl max-w-sm w-full overflow-hidden">
        {/* 头部 */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-2">
            <Info className="w-5 h-5 text-primary-500" />
            <h2 className="text-lg font-semibold">{t("permission.title")}</h2>
          </div>
          <button onClick={handleDismiss} className="p-1">
            <X className="w-5 h-5 text-gray-400" />
          </button>
        </div>

        {/* 内容 */}
        <div className="p-4 space-y-4">
          <div className="space-y-2">
            <h3 className="font-medium">{t("permission.why")}</h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {t("permission.whyDesc")}
            </p>
          </div>

          <div className="space-y-2">
            <h3 className="font-medium">{t("permission.android12")}</h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {t("permission.android12Desc")}
            </p>
          </div>

          <div className="space-y-2">
            <h3 className="font-medium">{t("permission.reduceTips")}</h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {t("permission.reduceTipsDesc")}
            </p>
          </div>
        </div>

        {/* 底部按钮 */}
        <div className="p-4 border-t border-gray-200 dark:border-gray-700">
          <button
            onClick={handleDismiss}
            className="w-full py-2 bg-primary-500 text-white rounded-xl font-medium"
          >
            {t("permission.gotIt")}
          </button>
        </div>
      </div>
    </div>
  );
}
