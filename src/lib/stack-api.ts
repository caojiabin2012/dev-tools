import { invoke } from '@tauri-apps/api/core'

export type DbEngine = 'mysql' | 'mariadb'
export type ServiceStatus = 'not_installed' | 'stopped' | 'running' | 'error'

export interface VersionOption {
  id: string
  label: string
  engine: string | null
}

export interface StackSettings {
  www_subdir: string
}

export interface StackEnvInfo {
  site_url: string | null
  www_root: string | null
  mysql_host: string
  mysql_port: number | null
  mysql_user: string
  mysql_password: string
  php_fastcgi: string | null
  redis_addr: string | null
}

export interface ComponentView {
  id: string
  name: string
  selected_version_id: string
  selected_version_label: string
  available_versions: VersionOption[]
  default_port: number
  downloaded: boolean
  download_path: string | null
  installed: boolean
  status: ServiceStatus
  port: number | null
  home_dir: string | null
  pid: number | null
  hint: string | null
  config_path: string | null
  log_path: string | null
}

export interface StackState {
  install_root: string | null
  settings: StackSettings
  env_info: StackEnvInfo
  components: ComponentView[]
}

export interface DownloadProgress {
  component: string
  downloaded: number
  total: number | null
  percent: number | null
  phase: string
}

export interface InstallComponentParams {
  component: string
  source_path?: string
  port?: number
  version_id?: string
  engine?: DbEngine
}

export interface UpdateStackSettingsParams {
  www_subdir?: string
}

export async function getStackState(): Promise<StackState> {
  return invoke('stack_get_state')
}

export async function setInstallRoot(path: string): Promise<StackState> {
  return invoke('stack_set_install_root', { path })
}

export async function setComponentVersion(component: string, versionId: string): Promise<StackState> {
  return invoke('stack_set_component_version', { params: { component, version_id: versionId } })
}

export async function pickInstallRoot(): Promise<string | null> {
  return invoke('stack_pick_install_root')
}

export async function pickWwwSubdir(): Promise<string | null> {
  return invoke('stack_pick_www_subdir')
}

export async function pickComponentSource(): Promise<string | null> {
  return invoke('stack_pick_component_source')
}

export async function downloadComponent(component: string, versionId?: string): Promise<StackState> {
  return invoke('stack_download_component', { component, versionId })
}

export async function installComponent(params: InstallComponentParams): Promise<StackState> {
  return invoke('stack_install_component', { params })
}

export async function startComponent(component: string): Promise<StackState> {
  return invoke('stack_start_component', { component })
}

export async function stopComponent(component: string): Promise<StackState> {
  return invoke('stack_stop_component', { component })
}

export async function uninstallComponent(component: string): Promise<StackState> {
  return invoke('stack_uninstall_component', { component })
}

export async function startAllComponents(): Promise<StackState> {
  return invoke('stack_start_all')
}

export async function stopAllComponents(): Promise<StackState> {
  return invoke('stack_stop_all')
}

export async function updateStackSettings(params: UpdateStackSettingsParams): Promise<StackState> {
  return invoke('stack_update_settings', { params })
}

export async function setComponentPort(component: string, port: number): Promise<StackState> {
  return invoke('stack_set_component_port', { params: { component, port } })
}

export async function regenerateConfigs(): Promise<StackState> {
  return invoke('stack_regenerate_configs')
}

export async function openComponentConfig(component: string): Promise<void> {
  return invoke('stack_open_component_config', { component })
}

export async function openComponentLog(component: string): Promise<void> {
  return invoke('stack_open_component_log', { component })
}

export async function openSite(): Promise<void> {
  return invoke('stack_open_site')
}

export async function openInstallRoot(): Promise<void> {
  return invoke('stack_open_install_root')
}

export async function openWwwRoot(): Promise<void> {
  return invoke('stack_open_www_root')
}
