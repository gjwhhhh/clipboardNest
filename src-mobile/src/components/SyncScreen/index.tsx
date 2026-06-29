import { useState, useEffect } from "react";
import { ChevronLeft, Wifi, RefreshCw, Smartphone } from "lucide-react";
import { useTranslation } from "../../hooks/useTranslation";
import { invoke } from "@tauri-apps/api/core";
import { useClipboardStore } from "../../stores/clipboardStore";
import * as api from "../../utils/tauri";

interface Props {
  onBack: () => void;
}

export function SyncScreen({ onBack }: Props) {
  const { t } = useTranslation();
  const { fetchItems, showToast } = useClipboardStore();
  const [isEnabled, setIsEnabled] = useState(false);
  const [localDevice, setLocalDevice] = useState<api.SyncDevice | null>(null);
  const [devices, setDevices] = useState<api.SyncDevice[]>([]);
  const [isSyncing, setIsSyncing] = useState(false);
  const [lastSync, setLastSync] = useState<string | null>(null);

  useEffect(() => {
    loadSyncState();
  }, []);

  useEffect(() => {
    if (!isEnabled) return;

    let cancelled = false;
    const refreshDevices = async () => {
      try {
        const discovered = await api.getDiscoveredDevices();
        if (!cancelled) {
          setDevices(discovered);
        }
      } catch (error) {
        console.error("刷新设备列表失败:", error);
      }
    };

    refreshDevices();
    const timer = window.setInterval(refreshDevices, 5000);

    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [isEnabled]);

  const loadSyncState = async () => {
    try {
      const settings = await invoke<Record<string, string>>("get_settings");
      setIsEnabled(settings.sync_enabled === "true");
      if (settings.last_sync) {
        setLastSync(new Date(parseInt(settings.last_sync)).toLocaleString());
      }
      if (settings.sync_enabled === "true") {
        const device = await api.startSync();
        setLocalDevice(device);
        setDevices(await api.getDiscoveredDevices());
      } else {
        setLocalDevice(await api.getLocalSyncDevice());
      }
    } catch (error) {
      console.error("加载同步状态失败:", error);
    }
  };

  const handleToggleSync = async (enabled: boolean) => {
    setIsEnabled(enabled);
    try {
      if (enabled) {
        const device = await api.startSync();
        setLocalDevice(device);
        setDevices(await api.getDiscoveredDevices());
      } else {
        await api.stopSync();
        setDevices([]);
      }
      await invoke("update_setting", {
        key: "sync_enabled",
        value: String(enabled),
      });
    } catch (error) {
      console.error("更新同步设置失败:", error);
      setIsEnabled(!enabled);
      showToast("error", t("sync.toggleFailed"));
    }
  };

  const handleSyncNow = async () => {
    if (devices.length === 0) {
      showToast("error", t("sync.noDevices"));
      return;
    }

    setIsSyncing(true);
    try {
      const report = await api.syncAllDevices();
      await fetchItems();
      const now = new Date().toLocaleString();
      setLastSync(now);
      await invoke("update_setting", {
        key: "last_sync",
        value: String(Date.now()),
      });
      showToast(
        "success",
        t("sync.syncComplete", {
          inserted: report.inserted,
          updated: report.updated,
        })
      );
    } catch (error) {
      console.error("同步失败:", error);
      showToast("error", t("sync.syncFailed"));
    } finally {
      setIsSyncing(false);
    }
  };

  return (
    <div className="min-h-screen bg-[rgb(var(--bg-primary))] safe-top">
      {/* 头部 */}
      <div className="flex items-center gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <button onClick={onBack} className="p-2">
          <ChevronLeft className="w-5 h-5" />
        </button>
        <h1 className="text-lg font-semibold">{t("sync.title")}</h1>
      </div>

      {/* 内容 */}
      <div className="p-4 space-y-6">
        {/* 启用同步 */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Wifi className="w-5 h-5 text-primary-500" />
            <span className="font-medium">{t("sync.enable")}</span>
          </div>
          <button
            onClick={() => handleToggleSync(!isEnabled)}
            className={`w-12 h-6 rounded-full transition-colors ${
              isEnabled ? "bg-primary-500" : "bg-gray-300 dark:bg-gray-600"
            }`}
          >
            <div
              className={`w-5 h-5 bg-white rounded-full shadow transition-transform ${
                isEnabled ? "translate-x-6" : "translate-x-0.5"
              }`}
            />
          </button>
        </div>

        {isEnabled && (
          <>
            {localDevice && (
              <div className="text-xs text-gray-500 bg-gray-100 dark:bg-gray-800 rounded-lg p-3">
                <div>{t("sync.deviceName")}: {localDevice.name}</div>
                <div>{localDevice.address}:{localDevice.port}</div>
              </div>
            )}

            {/* 发现的设备 */}
            <div>
              <h3 className="font-medium mb-3">{t("sync.discoveredDevices")}</h3>
              {devices.length === 0 ? (
                <div className="text-center py-8 text-gray-500">
                  <Smartphone className="w-12 h-12 mx-auto mb-2 opacity-50" />
                  <p>{t("sync.noDevices")}</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {devices.map((device) => (
                    <div
                      key={device.id}
                      className="flex items-center gap-3 p-3 bg-gray-100 dark:bg-gray-800 rounded-xl"
                    >
                      <Smartphone className="w-5 h-5 text-gray-400" />
                      <div className="flex-1">
                        <p className="font-medium">{device.name}</p>
                        <p className="text-xs text-gray-500">
                          {device.address}:{device.port}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* 立即同步 */}
            <button
              onClick={handleSyncNow}
              disabled={isSyncing}
              className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-500 text-white rounded-xl hover:bg-primary-600 transition-colors disabled:opacity-50"
            >
              <RefreshCw className={`w-4 h-4 ${isSyncing ? "animate-spin" : ""}`} />
              <span>{isSyncing ? t("sync.syncing") : t("sync.syncNow")}</span>
            </button>

            {/* 上次同步时间 */}
            <div className="text-sm text-gray-500 text-center">
              {t("sync.lastSync")}: {lastSync || t("sync.never")}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
