import type { ClipboardItem as ClipboardItemType } from "../../types";
import { Pin, Star } from "lucide-react";

interface Props {
  item: ClipboardItemType;
  isSelected: boolean;
  onClick: () => void;
  onLongPress?: () => void;
}

export function ClipboardItemCard({
  item,
  isSelected,
  onClick,
  onLongPress,
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

  let longPressTimer: ReturnType<typeof setTimeout> | null = null;

  const handleTouchStart = () => {
    longPressTimer = setTimeout(() => {
      onLongPress?.();
    }, 500);
  };

  const handleTouchEnd = () => {
    if (longPressTimer) {
      clearTimeout(longPressTimer);
    }
  };

  return (
    <div
      className={`
        flex items-center gap-3 p-4 cursor-pointer transition-colors
        ${isSelected
          ? "bg-primary-100 dark:bg-primary-900/30"
          : "hover:bg-gray-100 dark:hover:bg-gray-800"
        }
      `}
      onClick={onClick}
      onTouchStart={handleTouchStart}
      onTouchEnd={handleTouchEnd}
    >
      <div className="text-2xl">{getTypeIcon()}</div>

      <div className="flex-1 min-w-0">
        <p className="text-sm truncate text-gray-900 dark:text-gray-100">
          {item.preview || item.fileName || "空"}
        </p>
        <p className="text-xs text-gray-500 dark:text-gray-400">
          {new Date(item.createdAt).toLocaleString()}
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
    </div>
  );
}
