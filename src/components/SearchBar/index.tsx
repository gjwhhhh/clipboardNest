// 搜索栏组件
import { Search, X } from "lucide-react";
import { useClipboardStore } from "../../stores/clipboardStore";
import type { FilterType } from "../../types";

const filters: { label: string; value: FilterType }[] = [
  { label: "全部", value: "all" },
  { label: "文本", value: "text" },
  { label: "图片", value: "image" },
  { label: "文件", value: "file" },
];

export function SearchBar() {
  const { searchQuery, search, activeFilter, setFilter } = useClipboardStore();

  return (
    <div className="p-3 border-b border-gray-200 dark:border-gray-700">
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => search(e.target.value)}
          placeholder="搜索历史..."
          className="w-full pl-9 pr-8 py-2 bg-gray-100 dark:bg-gray-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
        />
        {searchQuery && (
          <button
            onClick={() => search("")}
            className="absolute right-3 top-1/2 -translate-y-1/2"
          >
            <X className="w-4 h-4 text-gray-400 hover:text-gray-600" />
          </button>
        )}
      </div>

      <div className="flex gap-2 mt-2">
        {filters.map((f) => (
          <button
            key={f.value}
            onClick={() => setFilter(f.value)}
            className={`
              px-3 py-1 text-xs rounded-full transition-colors
              ${activeFilter === f.value
                ? "bg-primary-500 text-white"
                : "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
              }
            `}
          >
            {f.label}
          </button>
        ))}
      </div>
    </div>
  );
}
