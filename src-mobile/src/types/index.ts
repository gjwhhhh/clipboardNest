export type ContentType = "text" | "richtext" | "image" | "file";

export interface ClipboardItem {
  id: number;
  contentType: ContentType;
  content: string | null;
  preview: string | null;
  contentHash: string;
  fileName: string | null;
  fileSize: number | null;
  filePath: string | null;
  thumbnailPath: string | null;
  sourceApp: string | null;
  isPinned: boolean;
  isFavorite: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface Settings {
  retentionDays: number;
  maxItems: number;
  pollIntervalMs: number;
  theme: "system" | "light" | "dark";
}

export type FilterType = "all" | ContentType;
