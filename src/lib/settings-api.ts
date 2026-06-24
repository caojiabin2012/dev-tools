import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  auto_start: boolean;
  close_to_tray: boolean;
}

export interface UpdateInfo {
  current_version: string;
  latest_version: string;
  download_url: string | null;
  release_notes: string | null;
}

export async function getSettings(): Promise<AppSettings> {
  return invoke('get_settings');
}

export async function saveSettings(autoStart: boolean, closeToTray: boolean): Promise<AppSettings> {
  return invoke('save_settings', { autoStart, closeToTray });
}

export async function getAppVersion(): Promise<string> {
  return invoke('get_app_version');
}

export async function checkForUpdate(): Promise<UpdateInfo | null> {
  return invoke('check_for_update');
}
