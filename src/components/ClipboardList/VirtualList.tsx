// 虚拟滚动列表组件
import { Virtuoso } from "react-virtuoso";
import type { ClipboardItem as ClipboardItemType } from "../../types";
import { ClipboardItemCard } from "./ClipboardItem";

interface Props {
  items: ClipboardItemType[];
  selectedId: number | null;
  onClick: (item: ClipboardItemType) => void;
  onHover: (item: ClipboardItemType) => void;
  onDelete: (id: number) => void;
  onPin: (id: number, pinned: boolean) => void;
  onFavorite: (id: number, favorite: boolean) => void;
}

export function VirtualList({
  items,
  selectedId,
  onClick,
  onHover,
  onDelete,
  onPin,
  onFavorite,
}: Props) {
  return (
    <Virtuoso
      className="flex-1"
      totalCount={items.length}
      itemContent={(index) => {
        const item = items[index];
        return (
          <ClipboardItemCard
            key={item.id}
            item={item}
            isSelected={item.id === selectedId}
            onClick={() => onClick(item)}
            onHover={() => onHover(item)}
            onDelete={() => onDelete(item.id)}
            onPin={() => onPin(item.id, !item.isPinned)}
            onFavorite={() => onFavorite(item.id, !item.isFavorite)}
          />
        );
      }}
    />
  );
}
