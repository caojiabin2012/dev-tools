import { invoke } from '@tauri-apps/api/core';

export interface QrGeneratePreview {
  id: number;
  text: string;
  created_at: string;
}

export interface QrGenerateDetail {
  id: number;
  text: string;
  png_data: number[];
  created_at: string;
}

export interface QrDecodeItem {
  id: number;
  text: string;
  file_name: string;
  created_at: string;
}

export interface ListQrItemsParams {
  time_filter?: string;
  search_query?: string;
}

export interface ListQrDecodeResult {
  items: QrDecodeItem[];
  total: number;
}

export interface ListQrGenerateResult {
  items: QrGeneratePreview[];
  total: number;
}

export async function decodeQrAndSave(
  imageData: number[] | Uint8Array,
  fileName: string,
): Promise<QrDecodeItem> {
  const bytes = imageData instanceof Uint8Array ? Array.from(imageData) : imageData;
  return invoke('decode_qr_and_save', { params: { image_data: bytes, file_name: fileName } });
}

export async function listQrDecodeItems(params: ListQrItemsParams = {}): Promise<ListQrDecodeResult> {
  return invoke('list_qr_decode_items', { params });
}

export async function getQrDecodeItem(id: number): Promise<QrDecodeItem | null> {
  return invoke('get_qr_decode_item', { id });
}

export async function deleteQrDecodeItem(id: number): Promise<void> {
  return invoke('delete_qr_decode_item', { id });
}

export async function clearQrDecodeItems(): Promise<void> {
  return invoke('clear_qr_decode_items');
}

export async function generateQrAndSave(text: string): Promise<QrGenerateDetail> {
  return invoke('generate_qr_and_save', { params: { text } });
}

export async function listQrGenerateItems(params: ListQrItemsParams = {}): Promise<ListQrGenerateResult> {
  return invoke('list_qr_generate_items', { params });
}

export async function getQrGenerateItem(id: number): Promise<QrGenerateDetail | null> {
  return invoke('get_qr_generate_item', { id });
}

export async function deleteQrGenerateItem(id: number): Promise<void> {
  return invoke('delete_qr_generate_item', { id });
}

export async function clearQrGenerateItems(): Promise<void> {
  return invoke('clear_qr_generate_items');
}

export async function copyQrGenerateToClipboard(id: number): Promise<void> {
  return invoke('copy_qr_generate_to_clipboard', { id });
}

export async function saveQrGenerateFile(id: number, defaultName?: string): Promise<string | null> {
  return invoke('save_qr_generate_file', { id, default_name: defaultName ?? null });
}

export function pngBytesToDataUrl(bytes: number[] | Uint8Array): string {
  const data = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  let binary = '';
  for (let i = 0; i < data.length; i += 1) {
    binary += String.fromCharCode(data[i]!);
  }
  return `data:image/png;base64,${btoa(binary)}`;
}

export function formatRelativeTime(createdAt: string): string {
  const ts = Date.parse(createdAt.includes('T') ? createdAt : createdAt.replace(' ', 'T'));
  if (Number.isNaN(ts)) return createdAt;

  const diff = Date.now() - ts;
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  if (hours < 24) return `${hours} 小时前`;
  if (days < 7) return `${days} 天前`;

  return new Date(ts).toLocaleString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}
