// Tauri API 封装
import { invoke } from "@tauri-apps/api/core";
import type { ClipboardItem } from "../types";

export async function getClipboardHistory(
  contentType?: string,
  limit?: number,
  offset?: number
): Promise<ClipboardItem[]> {
  return invoke("get_clipboard_history", {
    contentType: contentType || null,
    limit: limit || 100,
    offset: offset || 0,
  });
}

export async function searchClipboard(
  query: string,
  limit?: number
): Promise<ClipboardItem[]> {
  return invoke("search_clipboard", {
    query,
    limit: limit || 200,
  });
}

export async function copyToClipboard(itemId: number): Promise<void> {
  return invoke("copy_to_clipboard", { itemId });
}

export async function deleteClipboardItem(itemId: number): Promise<void> {
  return invoke("delete_clipboard_item", { itemId });
}

export async function pinClipboardItem(
  itemId: number,
  pinned: boolean
): Promise<void> {
  return invoke("pin_clipboard_item", { itemId, pinned });
}

export async function favoriteClipboardItem(
  itemId: number,
  favorite: boolean
): Promise<void> {
  return invoke("favorite_clipboard_item", { itemId, favorite });
}

export async function clearAllHistory(): Promise<void> {
  return invoke("clear_all_history");
}

export async function getSettings(): Promise<Record<string, string>> {
  return invoke("get_settings");
}

/// 隐藏主窗口
export async function hideWindow(): Promise<void> {
  await invoke("hide_window");
}

export async function updateSetting(
  key: string,
  value: string
): Promise<void> {
  return invoke("update_setting", { key, value });
}

export async function updateHotkey(newHotkey: string): Promise<void> {
  return invoke("update_hotkey", { newHotkey });
}

/// 获取图片的 base64 data URL
export async function getImageDataUrl(filePath: string): Promise<string> {
  return invoke("get_image_data_url", { filePath });
}
