import { Virtuoso } from "react-virtuoso";
import type { ClipboardItem as ClipboardItemType } from "../../types";
import { ClipboardItemCard } from "./ClipboardItem";
import { SwipeableItem } from "../common/SwipeableItem";

interface Props {
  items: ClipboardItemType[];
  selectedId: number | null;
  onClick: (item: ClipboardItemType) => void;
  onDelete: (id: number) => void;
  onFavorite: (id: number, favorite: boolean) => void;
}

export function VirtualList({
  items,
  selectedId,
  onClick,
  onDelete,
  onFavorite,
}: Props) {
  return (
    <Virtuoso
      className="flex-1"
      totalCount={items.length}
      overscan={100}
      itemContent={(index) => {
        const item = items[index];
        return (
          <SwipeableItem
            key={item.id}
            onDelete={() => onDelete(item.id)}
            onFavorite={() => onFavorite(item.id, !item.isFavorite)}
          >
            <ClipboardItemCard
              item={item}
              isSelected={item.id === selectedId}
              onClick={() => onClick(item)}
            />
          </SwipeableItem>
        );
      }}
    />
  );
}
