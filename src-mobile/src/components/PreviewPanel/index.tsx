import { useEffect, useState } from "react";
import { useClipboardStore } from "../../stores/clipboardStore";
import { useTranslation } from "../../hooks/useTranslation";
import { Copy, FileText, Image, File, X } from "lucide-react";
import { getFileDataUrl } from "../../utils/tauri";

export function PreviewPanel() {
  const { selectedItem, copyItem, selectItem } = useClipboardStore();
  const { t } = useTranslation();
  const [imageUrl, setImageUrl] = useState<string | null>(null);

  useEffect(() => {
    if (selectedItem?.contentType !== "image") {
      setImageUrl(null);
      return;
    }

    const imagePath = selectedItem.thumbnailPath || selectedItem.filePath;
    if (!imagePath) {
      setImageUrl(null);
      return;
    }

    let cancelled = false;
    getFileDataUrl(imagePath)
      .then((url) => {
        if (!cancelled) {
          setImageUrl(url);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setImageUrl(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedItem?.id, selectedItem?.contentType, selectedItem?.filePath, selectedItem?.thumbnailPath]);

  if (!selectedItem) {
    return null;
  }

  const renderContent = () => {
    switch (selectedItem.contentType) {
      case "image":
        return imageUrl ? (
          <img
            src={imageUrl}
            alt="clipboard"
            className="max-w-full max-h-[300px] object-contain rounded-lg"
          />
        ) : (
          <div className="flex items-center gap-2 text-gray-500 p-4">
            <Image className="w-8 h-8" />
            <span>{t("preview.imageNotAvailable")}</span>
          </div>
        );

      case "file":
        return (
          <div className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <File className="w-10 h-10 text-blue-500" />
            <div>
              <p className="font-medium">{selectedItem.fileName}</p>
              <p className="text-sm text-gray-500">
                {selectedItem.fileSize
                  ? `${(selectedItem.fileSize / 1024).toFixed(1)} KB`
                  : t("preview.unknownSize")}
              </p>
            </div>
          </div>
        );

      default:
        return (
          <pre className="whitespace-pre-wrap break-words text-sm font-mono p-4 bg-gray-50 dark:bg-gray-800 rounded-lg overflow-auto max-h-[300px]">
            {selectedItem.content}
          </pre>
        );
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 z-40 flex items-end">
      <div className="bg-white dark:bg-gray-900 w-full max-h-[70vh] rounded-t-2xl overflow-hidden">
        {/* 头部 */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-2">
            {selectedItem.contentType === "text" && <FileText className="w-4 h-4" />}
            {selectedItem.contentType === "image" && <Image className="w-4 h-4" />}
            {selectedItem.contentType === "file" && <File className="w-4 h-4" />}
            <span className="text-sm font-medium capitalize">
              {selectedItem.contentType}
            </span>
          </div>
          <button
            onClick={() => selectItem(null)}
            className="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-full"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* 内容 */}
        <div className="p-4 overflow-auto max-h-[50vh]">
          {renderContent()}
        </div>

        {/* 底部操作 */}
        <div className="p-4 border-t border-gray-200 dark:border-gray-700 safe-bottom">
          <button
            onClick={() => copyItem(selectedItem)}
            className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-500 text-white rounded-xl hover:bg-primary-600 transition-colors"
          >
            <Copy className="w-4 h-4" />
            <span>{t("preview.copy")}</span>
          </button>
        </div>
      </div>
    </div>
  );
}
