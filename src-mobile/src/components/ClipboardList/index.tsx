import { useClipboard } from "../../hooks/useClipboard";
import { VirtualList } from "./VirtualList";
import { PullToRefresh } from "../common/PullToRefresh";
import { Star, Search, ClipboardList as ClipboardListIcon } from "lucide-react";
import { useTranslation } from "../../hooks/useTranslation";

interface Props {
  favoritesOnly?: boolean;
}

export function ClipboardList({ favoritesOnly = false }: Props) {
  const { t } = useTranslation();
  const {
    filteredItems,
    selectedItem,
    searchQuery,
    selectItem,
    copyItem,
    deleteItem,
    favoriteItem,
    fetchItems,
  } = useClipboard();

  const handleCopyAndHide = async (item: typeof filteredItems[0]) => {
    try {
      await copyItem(item);
    } catch {
      // 错误已通过 toast 显示
    }
  };

  const visibleItems = favoritesOnly
    ? filteredItems.filter((item) => item.isFavorite)
    : filteredItems;

  const emptyState = favoritesOnly
    ? {
        icon: Star,
        title: t("clipboard.emptyFavorites"),
      }
    : searchQuery
      ? {
          icon: Search,
          title: t("clipboard.emptySearch"),
        }
      : {
          icon: ClipboardListIcon,
          title: t("clipboard.empty"),
        };

  const EmptyIcon = emptyState.icon;

  return (
    <PullToRefresh onRefresh={fetchItems}>
      {visibleItems.length === 0 ? (
        <div className="h-full flex flex-col items-center justify-center gap-3 px-6 text-center text-gray-500 dark:text-gray-400">
          <EmptyIcon className="w-10 h-10 opacity-50" />
          <p className="text-sm">{emptyState.title}</p>
        </div>
      ) : (
        <VirtualList
          items={visibleItems}
          selectedId={selectedItem?.id ?? null}
          onClick={handleCopyAndHide}
          onPreview={selectItem}
          onDelete={deleteItem}
          onFavorite={favoriteItem}
        />
      )}
    </PullToRefresh>
  );
}
