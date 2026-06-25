import { useClipboard } from "../../hooks/useClipboard";
import { VirtualList } from "./VirtualList";
import { PullToRefresh } from "../common/PullToRefresh";

export function ClipboardList() {
  const {
    filteredItems,
    selectedItem,
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

  return (
    <PullToRefresh onRefresh={fetchItems}>
      <VirtualList
        items={filteredItems}
        selectedId={selectedItem?.id ?? null}
        onClick={handleCopyAndHide}
        onDelete={deleteItem}
        onFavorite={favoriteItem}
      />
    </PullToRefresh>
  );
}
