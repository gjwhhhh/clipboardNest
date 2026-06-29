import { useEffect, useRef } from "react";
import { Search, X } from "lucide-react";
import { useClipboardStore } from "../../stores/clipboardStore";
import { useTranslation } from "../../hooks/useTranslation";
import type { FilterType } from "../../types";

interface Props {
  autoFocus?: boolean;
}

export function SearchBar({ autoFocus = false }: Props) {
  const { searchQuery, search, activeFilter, setFilter } = useClipboardStore();
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (autoFocus) {
      inputRef.current?.focus();
    }
  }, [autoFocus]);

  const filters: { label: string; value: FilterType }[] = [
    { label: t("search.all"), value: "all" },
    { label: t("search.text"), value: "text" },
    { label: t("search.image"), value: "image" },
    { label: t("search.file"), value: "file" },
  ];

  return (
    <div className="p-3 border-b border-gray-200 dark:border-gray-700">
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          ref={inputRef}
          type="text"
          value={searchQuery}
          onChange={(e) => search(e.target.value)}
          placeholder={t("search.placeholder")}
          className="w-full pl-9 pr-8 py-2.5 bg-gray-100 dark:bg-gray-800 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
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

      <div className="flex gap-2 mt-3 overflow-x-auto pb-1">
        {filters.map((f) => (
          <button
            key={f.value}
            onClick={() => setFilter(f.value)}
            className={`
              px-4 py-1.5 text-xs rounded-full transition-colors whitespace-nowrap
              ${activeFilter === f.value
                ? "bg-primary-500 text-white"
                : "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
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
