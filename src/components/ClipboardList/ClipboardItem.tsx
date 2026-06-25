// 剪切板列表项组件
import type { ClipboardItem as ClipboardItemType } from "../../types";
import { Pin, Star, Trash2 } from "lucide-react";

interface Props {
  item: ClipboardItemType;
  isSelected: boolean;
  onClick: () => void;
  onHover: () => void;
  onDelete: () => void;
  onPin: () => void;
  onFavorite: () => void;
}

export function ClipboardItemCard({
  item,
  isSelected,
  onClick,
  onHover,
  onDelete,
  onPin,
  onFavorite,
}: Props) {
  const getTypeIcon = () => {
    switch (item.contentType) {
      case "image":
        return "🖼️";
      case "file":
        return "📁";
      case "richtext":
        return "📝";
      default:
        return "📋";
    }
  };

  return (
    <div
      className={`
        flex items-center gap-3 p-3 cursor-pointer transition-colors
        ${isSelected
          ? "bg-primary-100 dark:bg-primary-900/30 border-l-2 border-primary-500"
          : "hover:bg-gray-100 dark:hover:bg-gray-800 border-l-2 border-transparent"
        }
      `}
      onClick={onClick}
      onMouseEnter={onHover}
    >
      <div className="text-2xl flex-shrink-0">{getTypeIcon()}</div>

      <div className="flex-1 min-w-0">
        <p className="text-sm truncate text-gray-900 dark:text-gray-100">
          {item.contentType === "image"
            ? `${(item.fileName && item.fileName !== "clipboard_image.png") ? item.fileName : "图片"} ${item.preview || ""}`
            : item.preview || item.fileName || "空"
          }
        </p>
        <p className="text-xs text-gray-500 dark:text-gray-400">
          {new Date(item.createdAt).toLocaleString()}
          {item.sourceApp && ` • ${item.sourceApp}`}
        </p>
      </div>

      <div className="flex items-center gap-1">
        {item.isPinned && (
          <Pin className="w-4 h-4 text-primary-500" />
        )}
        {item.isFavorite && (
          <Star className="w-4 h-4 text-yellow-500" />
        )}
      </div>

      <div className="flex gap-1">
        <button
          onClick={(e) => { e.stopPropagation(); onPin(); }}
          className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded"
          title={item.isPinned ? "取消置顶" : "置顶"}
        >
          <Pin className={`w-4 h-4 ${item.isPinned ? "text-primary-500" : "text-gray-400"}`} />
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); onFavorite(); }}
          className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded"
          title={item.isFavorite ? "取消收藏" : "收藏"}
        >
          <Star className={`w-4 h-4 ${item.isFavorite ? "text-yellow-500" : "text-gray-400"}`} />
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); onDelete(); }}
          className="p-1 hover:bg-red-100 dark:hover:bg-red-900/30 rounded"
          title="删除"
        >
          <Trash2 className="w-4 h-4 text-gray-400 hover:text-red-500" />
        </button>
      </div>
    </div>
  );
}
