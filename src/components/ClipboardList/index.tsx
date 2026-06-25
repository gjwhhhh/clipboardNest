// 剪切板列表组件
import { useClipboard } from "../../hooks/useClipboard";
import { useKeyboard } from "../../hooks/useKeyboard";
import { useClipboardStore } from "../../stores/clipboardStore";
import { VirtualList } from "./VirtualList";
import * as api from "../../utils/tauri";
import type { ClipboardItem } from "../../types";

export function ClipboardList() {
  const {
    filteredItems,
    selectedItem,
    selectItem,
    copyItem,
    deleteItem,
    pinItem,
    favoriteItem,
  } = useClipboard();

  const { setHoveredItem } = useClipboardStore();

  const selectedIndex = filteredItems.findIndex(
    (item) => item.id === selectedItem?.id
  );

  // 复制并隐藏窗口
  const handleCopyAndHide = async (item: ClipboardItem) => {
    try {
      await copyItem(item);
      await api.hideWindow();
    } catch {
      // 复制失败时不隐藏窗口，toast 已经显示了错误信息
    }
  };

  // hover 时设置预览项
  const handleHover = (item: ClipboardItem) => {
    setHoveredItem(item);
  };

  useKeyboard({
    onArrowUp: () => {
      if (selectedIndex > 0) {
        const newItem = filteredItems[selectedIndex - 1];
        selectItem(newItem);
        setHoveredItem(newItem);
      }
    },
    onArrowDown: () => {
      if (selectedIndex < filteredItems.length - 1) {
        const newItem = filteredItems[selectedIndex + 1];
        selectItem(newItem);
        setHoveredItem(newItem);
      }
    },
    onEnter: () => {
      if (selectedItem) {
        handleCopyAndHide(selectedItem);
      }
    },
    onDelete: () => {
      if (selectedItem) {
        deleteItem(selectedItem.id);
      }
    },
    onEscape: () => {
      api.hideWindow();
    },
  });

  return (
    <VirtualList
      items={filteredItems}
      selectedId={selectedItem?.id ?? null}
      onClick={handleCopyAndHide}
      onHover={handleHover}
      onDelete={deleteItem}
      onPin={pinItem}
      onFavorite={favoriteItem}
    />
  );
}
