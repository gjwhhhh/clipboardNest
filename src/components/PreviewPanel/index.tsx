// 预览面板组件 - 右侧实时预览，由 hover 驱动
import { useEffect, useState } from "react";
import { useClipboardStore } from "../../stores/clipboardStore";
import { FileText, Image, File } from "lucide-react";
import { getImageDataUrl } from "../../utils/tauri";

export function PreviewPanel() {
  const { selectedItem, hoveredItem } = useClipboardStore();
  const [imageUrl, setImageUrl] = useState<string | null>(null);

  // 优先显示 hover 项，其次显示选中项
  const previewItem = hoveredItem || selectedItem;

  // 加载图片 data URL
  useEffect(() => {
    if (previewItem?.contentType !== "image") {
      setImageUrl(null);
      return;
    }

    const imagePath = previewItem.thumbnailPath || previewItem.filePath;
    if (!imagePath) {
      setImageUrl(null);
      return;
    }

    setImageUrl(null);
    getImageDataUrl(imagePath)
      .then((url) => {
        setImageUrl(url);
      })
      .catch((err) => {
        console.error("图片加载失败:", err);
        setImageUrl(null);
      });
  }, [previewItem?.id, previewItem?.contentType, previewItem?.filePath, previewItem?.thumbnailPath]);

  if (!previewItem) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-400 dark:text-gray-600">
        <p>悬停或选择一个项目进行预览</p>
      </div>
    );
  }

  const renderContent = () => {
    switch (previewItem.contentType) {
      case "image":
        if (imageUrl) {
          return (
            <img
              src={imageUrl}
              alt="剪切板图片"
              className="max-w-full max-h-[400px] object-contain rounded"
            />
          );
        }
        return (
          <div className="flex items-center gap-2 text-gray-500">
            <Image className="w-8 h-8" />
            <span>{previewItem.filePath || previewItem.thumbnailPath ? "加载中..." : "图片处理中..."}</span>
          </div>
        );

      case "file":
        return (
          <div className="flex items-center gap-3 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <File className="w-10 h-10 text-blue-500" />
            <div>
              <p className="font-medium">{previewItem.fileName}</p>
              <p className="text-sm text-gray-500">
                {previewItem.fileSize
                  ? `${(previewItem.fileSize / 1024).toFixed(1)} KB`
                  : "未知大小"}
              </p>
            </div>
          </div>
        );

      default:
        return (
          <pre className="whitespace-pre-wrap break-words text-sm font-mono p-4 bg-gray-50 dark:bg-gray-800 rounded-lg overflow-auto max-h-[400px]">
            {previewItem.content}
          </pre>
        );
    }
  };

  return (
    <div className="flex-1 p-4 overflow-auto">
      <div className="flex items-center gap-2 mb-3">
        {previewItem.contentType === "text" && <FileText className="w-4 h-4" />}
        {previewItem.contentType === "image" && <Image className="w-4 h-4" />}
        {previewItem.contentType === "file" && <File className="w-4 h-4" />}
        <span className="text-sm font-medium capitalize">
          {previewItem.contentType}
          {previewItem.contentType === "image" && previewItem.preview && (
            <span className="text-gray-400 ml-2">
              {(previewItem.fileName && previewItem.fileName !== "clipboard_image.png")
                ? `${previewItem.fileName} `
                : ""}
              {previewItem.preview}
            </span>
          )}
        </span>
      </div>

      {renderContent()}

      <div className="mt-3 text-xs text-gray-500">
        <p>复制时间: {new Date(previewItem.createdAt).toLocaleString()}</p>
        {previewItem.sourceApp && <p>来源应用: {previewItem.sourceApp}</p>}
      </div>
    </div>
  );
}
