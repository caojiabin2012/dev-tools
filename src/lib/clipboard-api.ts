import { invoke } from '@tauri-apps/api/core';

export interface ClipboardItemPreview {
  id: number;
  content_type: string;
  content_text: string | null;
  has_image: boolean;
  image_width: number | null;
  image_height: number | null;
  mime_type: string | null;
  is_pinned: boolean;
  created_at: string;
}

export interface ClipboardItemDetail {
  id: number;
  content_type: string;
  content_text: string | null;
  content_image: number[] | null;
  image_width: number | null;
  image_height: number | null;
  mime_type: string | null;
  is_pinned: boolean;
  created_at: string;
}

export interface GetItemsParams {
  content_type?: string;
  search_query?: string;
  time_filter?: string;
  limit?: number;
  offset?: number;
}

export interface GetItemsResult {
  items: ClipboardItemPreview[];
  total: number;
}

export async function getClipboardItems(params: GetItemsParams = {}): Promise<GetItemsResult> {
  return invoke('get_clipboard_items', { params });
}

export async function getClipboardItemDetail(id: number): Promise<ClipboardItemDetail | null> {
  return invoke('get_clipboard_item_detail', { id });
}

export async function deleteClipboardItem(id: number): Promise<void> {
  return invoke('delete_clipboard_item', { id });
}

export async function togglePinItem(id: number): Promise<boolean> {
  return invoke('toggle_pin_item', { id });
}

export async function clearClipboardHistory(): Promise<void> {
  return invoke('clear_clipboard_history');
}

export async function copyToClipboard(text: string): Promise<void> {
  return invoke('copy_to_clipboard', { text });
}

export async function pasteFromClipboard(): Promise<string> {
  return invoke('paste_from_clipboard');
}
