import { invoke } from "@tauri-apps/api/core";
import type { ClipboardItem } from "../types";

export interface SyncDevice {
  id: string;
  name: string;
  address: string;
  port: number;
  last_seen: number;
}

export interface SyncReport {
  inserted: number;
  updated: number;
  skipped: number;
}

async function writeTextWithBrowserClipboard(text: string): Promise<boolean> {
  if (navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // Fall back to the legacy path below for WebViews with partial support.
    }
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.opacity = "0";
  textarea.style.pointerEvents = "none";
  document.body.appendChild(textarea);
  textarea.select();

  try {
    return document.execCommand("copy");
  } finally {
    document.body.removeChild(textarea);
  }
}

async function readTextWithBrowserClipboard(): Promise<string | null> {
  if (!navigator.clipboard?.readText) {
    return null;
  }

  try {
    return await navigator.clipboard.readText();
  } catch {
    return null;
  }
}

async function readTextWithNativeClipboard(): Promise<string | null> {
  try {
    return await invoke<string | null>("plugin:clipboard-monitor|get_text");
  } catch {
    return null;
  }
}

export async function writeTextToClipboard(text: string): Promise<void> {
  try {
    await invoke("plugin:clipboard-monitor|set_text", { text });
    return;
  } catch {
    // Fall back to WebView clipboard APIs when the native plugin is unavailable.
  }

  const didWrite = await writeTextWithBrowserClipboard(text);
  if (!didWrite) {
    throw new Error("Clipboard write is not available");
  }
}

export async function ingestCurrentTextClipboard(): Promise<boolean> {
  const text =
    (await readTextWithNativeClipboard()) ?? (await readTextWithBrowserClipboard());
  if (!text) {
    return false;
  }

  if (!text.trim()) {
    return false;
  }

  await invoke("plugin:clipboard-monitor|update_clipboard_content", {
    content: {
      text,
      content_type: "text",
    },
  });
  return true;
}

function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });
}

export async function saveClipboardImage(dataUrl: string): Promise<void> {
  return invoke("save_clipboard_image", { dataUrl });
}

export async function getFileDataUrl(filePath: string): Promise<string> {
  return invoke("get_file_data_url", { filePath });
}

async function readImageWithNativeClipboard(): Promise<string | null> {
  try {
    return await invoke<string | null>("plugin:clipboard-monitor|get_image");
  } catch {
    return null;
  }
}

export async function ingestCurrentImageClipboard(): Promise<boolean> {
  const nativeImage = await readImageWithNativeClipboard();
  if (nativeImage) {
    await saveClipboardImage(nativeImage);
    return true;
  }

  if (!navigator.clipboard?.read) {
    return false;
  }

  let items: ClipboardItems;
  try {
    items = await navigator.clipboard.read();
  } catch {
    return false;
  }

  for (const item of items) {
    const imageType = item.types.find((type) => type.startsWith("image/"));
    if (!imageType) continue;

    const blob = await item.getType(imageType);
    const dataUrl = await blobToDataUrl(blob);
    await saveClipboardImage(dataUrl);
    return true;
  }

  return false;
}

export async function ingestCurrentClipboard(): Promise<boolean> {
  const didIngestText = await ingestCurrentTextClipboard();
  if (didIngestText) {
    return true;
  }
  return ingestCurrentImageClipboard();
}

export async function writeImageToClipboard(filePath: string): Promise<void> {
  const dataUrl = await getFileDataUrl(filePath);

  try {
    await invoke("plugin:clipboard-monitor|set_image", { dataUrl });
    return;
  } catch {
    // Fall back to WebView clipboard APIs when native image write is unavailable.
  }

  const ClipboardItemCtor = globalThis.ClipboardItem;
  if (!navigator.clipboard?.write || !ClipboardItemCtor) {
    throw new Error("Image clipboard write is not available");
  }

  const blob = await fetch(dataUrl).then((response) => response.blob());
  await navigator.clipboard.write([
    new ClipboardItemCtor({
      [blob.type || "image/png"]: blob,
    }),
  ]);
}

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

export async function updateSetting(
  key: string,
  value: string
): Promise<void> {
  return invoke("update_setting", { key, value });
}

export async function startSync(): Promise<SyncDevice> {
  return invoke("start_sync");
}

export async function stopSync(): Promise<void> {
  return invoke("stop_sync");
}

export async function getLocalSyncDevice(): Promise<SyncDevice> {
  return invoke("get_local_sync_device");
}

export async function getDiscoveredDevices(): Promise<SyncDevice[]> {
  return invoke("get_discovered_devices");
}

export async function syncWithDevice(deviceId: string): Promise<SyncReport> {
  return invoke("sync_with_device", { deviceId });
}

export async function syncAllDevices(): Promise<SyncReport> {
  return invoke("sync_all_devices");
}
