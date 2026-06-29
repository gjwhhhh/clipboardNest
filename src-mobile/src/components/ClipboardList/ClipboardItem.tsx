import { useRef } from "react";
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
  const longPressTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const longPressTriggered = useRef(false);

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

  const handleTouchStart = () => {
    longPressTriggered.current = false;
    longPressTimer.current = setTimeout(() => {
      longPressTriggered.current = true;
      onLongPress?.();
    }, 500);
  };

  const clearLongPressTimer = () => {
    if (longPressTimer.current) {
      clearTimeout(longPressTimer.current);
      longPressTimer.current = null;
    }
  };

  const handleTouchEnd = () => {
    clearLongPressTimer();
  };

  const handleClick = () => {
    if (longPressTriggered.current) {
      longPressTriggered.current = false;
      return;
    }
    onClick();
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
      onClick={handleClick}
      onTouchStart={handleTouchStart}
      onTouchMove={clearLongPressTimer}
      onTouchEnd={handleTouchEnd}
      onTouchCancel={clearLongPressTimer}
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
