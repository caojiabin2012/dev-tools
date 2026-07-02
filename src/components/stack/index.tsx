import { useCallback, useEffect, useMemo, useState, memo, type ReactNode } from 'react'
import { listen } from '@tauri-apps/api/event'
import {
  getStackState,
  setInstallRoot,
  pickInstallRoot,
  pickComponentSource,
  setComponentVersion,
  downloadComponent,
  installComponent,
  startComponent,
  stopComponent,
  uninstallComponent,
  startAllComponents,
  stopAllComponents,
  openInstallRoot,
  openWwwRoot,
  openSite,
  openComponentConfig,
  openComponentLog,
  pickWwwSubdir,
  updateStackSettings,
  setComponentPort,
  regenerateConfigs,
  type StackState,
  type ComponentView,
  type DownloadProgress,
  type ServiceStatus,
} from '@/lib/stack-api'

const POLL_INTERVAL_MS = 5000

function stackStateEqual(a: StackState, b: StackState): boolean {
  if (a.install_root !== b.install_root) return false
  if (a.settings?.www_subdir !== b.settings?.www_subdir) return false
  if (a.components.length !== b.components.length) return false
  for (let i = 0; i < a.components.length; i++) {
    const x = a.components[i]
    const y = b.components[i]
    if (
      x.id !== y.id ||
      x.status !== y.status ||
      x.pid !== y.pid ||
      x.port !== y.port ||
      x.installed !== y.installed ||
      x.downloaded !== y.downloaded ||
      x.selected_version_id !== y.selected_version_id
    ) {
      return false
    }
  }
  return true
}

const STATUS: Record<ServiceStatus, { label: string; dot: string; badge: string; text: string }> = {
  not_installed: {
    label: '未安装',
    dot: 'bg-muted-foreground/40',
    badge: 'bg-muted text-muted-foreground',
    text: 'text-muted-foreground',
  },
  stopped: {
    label: '已停止',
    dot: 'bg-red-500',
    badge: 'bg-red-500/10 text-red-700 dark:text-red-300 ring-1 ring-red-500/20',
    text: 'text-red-600 dark:text-red-400',
  },
  running: {
    label: '已启动',
    dot: 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]',
    badge: 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 ring-1 ring-emerald-500/20',
    text: 'text-emerald-600 dark:text-emerald-400',
  },
  error: {
    label: '异常',
    dot: 'bg-red-500',
    badge: 'bg-red-500/10 text-red-700 dark:text-red-300 ring-1 ring-red-500/20',
    text: 'text-red-600 dark:text-red-400',
  },
}

const SERVICE_THEME: Record<string, { icon: string; gradient: string; ring: string; label: string }> = {
  mysql: {
    icon: '🐬',
    gradient: 'from-sky-500/15 to-blue-600/5 dark:from-sky-500/20 dark:to-blue-900/10',
    ring: 'ring-sky-500/20',
    label: 'MySQL',
  },
  nginx: {
    icon: '🌐',
    gradient: 'from-emerald-500/15 to-green-600/5 dark:from-emerald-500/20 dark:to-green-900/10',
    ring: 'ring-emerald-500/20',
    label: 'Nginx',
  },
  php: {
    icon: '🐘',
    gradient: 'from-violet-500/15 to-indigo-600/5 dark:from-violet-500/20 dark:to-indigo-900/10',
    ring: 'ring-violet-500/20',
    label: 'PHP',
  },
  redis: {
    icon: '⚡',
    gradient: 'from-red-500/15 to-rose-600/5 dark:from-red-500/20 dark:to-rose-900/10',
    ring: 'ring-red-500/20',
    label: 'Redis',
  },
}

function StatusBadge({ status, pid }: { status: ServiceStatus; pid?: number | null }) {
  const s = STATUS[status] ?? STATUS.stopped
  return (
    <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium ${s.badge}`}>
      <span className={`w-1.5 h-1.5 rounded-full ${s.dot} ${status === 'running' ? 'animate-pulse' : ''}`} />
      {s.label}
      {pid ? <span className="opacity-70 font-mono">#{pid}</span> : null}
    </span>
  )
}

function ActionBtn({
  children,
  onClick,
  disabled,
  variant = 'default',
  className = '',
}: {
  children: ReactNode
  onClick: () => void
  disabled?: boolean
  variant?: 'primary' | 'default' | 'danger' | 'ghost'
  className?: string
}) {
  const base = 'inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all disabled:opacity-40 disabled:pointer-events-none'
  const variants = {
    primary: 'bg-primary text-primary-foreground hover:bg-primary/90 shadow-sm',
    default: 'border border-border bg-background hover:bg-accent hover:border-foreground/10',
    danger: 'border border-destructive/25 text-destructive hover:bg-destructive/10',
    ghost: 'text-muted-foreground hover:text-foreground hover:bg-accent',
  }
  return (
    <button type="button" disabled={disabled} onClick={onClick} className={`${base} ${variants[variant]} ${className}`}>
      {children}
    </button>
  )
}

const ServiceCard = memo(function ServiceCard({
  component,
  loading,
  progress,
  canInstall,
  onVersionChange,
  onDownload,
  onPickLocal,
  onInstall,
  onStart,
  onStop,
  onUninstall,
  onSetPort,
  onOpenConfig,
  onOpenLog,
  localSource,
}: {
  component: ComponentView
  loading: boolean
  progress?: DownloadProgress
  canInstall: boolean
  onVersionChange: (versionId: string) => void
  onDownload: () => void
  onPickLocal: () => void
  onInstall: () => void
  onStart: () => void
  onStop: () => void
  onUninstall: () => void
  onSetPort: (port: number) => void
  onOpenConfig: () => void
  onOpenLog?: () => void
  localSource?: string
}) {
  const theme = SERVICE_THEME[component.id] ?? SERVICE_THEME.mysql
  const downloading = progress?.component === component.id && progress.phase === 'downloading'
  const [portInput, setPortInput] = useState(String(component.port ?? component.default_port))
  const canEditPort = component.installed && component.status !== 'running'

  useEffect(() => {
    setPortInput(String(component.port ?? component.default_port))
  }, [component.port, component.default_port])

  return (
    <article
      className={`group relative overflow-hidden rounded-2xl border border-border bg-card shadow-sm hover:shadow-md transition-shadow ring-1 ${theme.ring}`}
    >
      <div className={`absolute inset-x-0 top-0 h-20 bg-gradient-to-br ${theme.gradient} pointer-events-none`} />

      <div className="relative p-4 space-y-4">
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-start gap-3 min-w-0">
            <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-background/80 border border-border/60 text-xl shadow-sm">
              {theme.icon}
            </div>
            <div className="min-w-0">
              <div className="flex items-center gap-2 flex-wrap">
                <h3 className="text-sm font-semibold text-foreground">{component.name}</h3>
                <span className="text-[10px] font-mono px-1.5 py-0.5 rounded-md bg-secondary text-muted-foreground truncate max-w-[140px]">
                  {component.installed ? component.selected_version_label : component.selected_version_label}
                </span>
              </div>
              <p className="text-xs text-muted-foreground mt-0.5 truncate">
                {component.hint ?? `默认端口 ${component.default_port}`}
              </p>
            </div>
          </div>
          {component.installed && (
            <StatusBadge status={component.status} pid={component.pid} />
          )}
        </div>

        {downloading && progress && (
          <div className="rounded-xl bg-secondary/50 p-3 space-y-2">
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">正在下载…</span>
              <span className="font-mono text-foreground">
                {progress.percent != null ? `${progress.percent.toFixed(0)}%` : '…'}
              </span>
            </div>
            <div className="h-2 rounded-full bg-background overflow-hidden">
              <div
                className="h-full rounded-full bg-primary transition-all duration-300"
                style={{ width: `${Math.min(progress.percent ?? 5, 100)}%` }}
              />
            </div>
          </div>
        )}

        {!component.installed && component.available_versions.length > 0 && (
          <div className="flex items-center gap-2 text-xs">
            <span className="text-muted-foreground shrink-0">版本</span>
            <select
              value={component.selected_version_id}
              disabled={loading}
              onChange={(e) => onVersionChange(e.target.value)}
              className="flex-1 rounded-lg border border-border bg-background px-2.5 py-1.5 text-xs focus:outline-none focus:ring-2 focus:ring-primary/30"
            >
              {component.available_versions.map((v) => (
                <option key={v.id} value={v.id}>
                  {v.label}
                </option>
              ))}
            </select>
          </div>
        )}

        {component.installed && (
          <div className="text-xs text-muted-foreground rounded-lg bg-secondary/30 px-2.5 py-1.5">
            已安装版本：<span className="text-foreground font-medium">{component.selected_version_label}</span>
          </div>
        )}

        {component.installed && (
          <div className="grid grid-cols-2 gap-2">
            <div className="rounded-xl bg-secondary/40 px-3 py-2">
              <div className="text-[10px] uppercase tracking-wide text-muted-foreground">端口</div>
              {canEditPort ? (
                <div className="flex gap-1 mt-1">
                  <input
                    type="number"
                    value={portInput}
                    onChange={(e) => setPortInput(e.target.value)}
                    className="w-full rounded-md border border-border bg-background px-2 py-1 text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary/30"
                  />
                  <ActionBtn
                    variant="default"
                    disabled={loading}
                    className="shrink-0 px-2"
                    onClick={() => {
                      const port = Number(portInput)
                      if (!Number.isInteger(port) || port < 1 || port > 65535) return
                      onSetPort(port)
                    }}
                  >
                    应用
                  </ActionBtn>
                </div>
              ) : (
                <div className="font-mono text-sm font-medium mt-0.5">{component.port}</div>
              )}
            </div>
            <div className="rounded-xl bg-secondary/40 px-3 py-2">
              <div className="text-[10px] uppercase tracking-wide text-muted-foreground">状态</div>
              <div className={`text-sm font-medium mt-0.5 ${STATUS[component.status]?.text ?? ''}`}>
                {STATUS[component.status]?.label}
              </div>
            </div>
            {component.home_dir && (
              <div className="col-span-2 rounded-xl bg-secondary/40 px-3 py-2">
                <div className="text-[10px] uppercase tracking-wide text-muted-foreground">安装路径</div>
                <div className="font-mono text-[11px] text-foreground/80 mt-0.5 break-all line-clamp-2">
                  {component.home_dir}
                </div>
              </div>
            )}
          </div>
        )}

        {localSource && !component.installed && (
          <p className="text-[11px] text-muted-foreground truncate rounded-lg bg-secondary/30 px-2 py-1.5">
            📁 {localSource}
          </p>
        )}

        <div className="flex flex-wrap gap-2 pt-1 border-t border-border/60">
          {!component.downloaded && !component.installed && (
            <>
              <ActionBtn variant="primary" disabled={loading} onClick={onDownload}>
                ⬇ 下载
              </ActionBtn>
              <ActionBtn variant="default" disabled={loading} onClick={onPickLocal}>
                📂 本地包
              </ActionBtn>
            </>
          )}
          {canInstall && (
            <ActionBtn variant="primary" disabled={loading} onClick={onInstall}>
              ✓ 安装
            </ActionBtn>
          )}
          {component.installed && (
            <>
              <ActionBtn variant="primary" disabled={loading || component.status === 'running'} onClick={onStart}>
                ▶ 启动
              </ActionBtn>
              <ActionBtn variant="default" disabled={loading || component.status !== 'running'} onClick={onStop}>
                ■ 停止
              </ActionBtn>
              <ActionBtn variant="default" disabled={loading} onClick={onOpenConfig}>
                ⚙ 配置
              </ActionBtn>
              {onOpenLog && (
                <ActionBtn variant="ghost" disabled={loading} onClick={onOpenLog}>
                  📋 日志
                </ActionBtn>
              )}
              <ActionBtn variant="danger" disabled={loading} onClick={onUninstall}>
                卸载
              </ActionBtn>
            </>
          )}
          {component.downloaded && !component.installed && !canInstall && (
            <span className="text-xs text-emerald-600 dark:text-emerald-400 self-center">✓ 安装包已就绪</span>
          )}
        </div>
      </div>
    </article>
  )
})

function EnvSettingsPanel({
  state,
  loading,
  wwwSubdir,
  onWwwSubdirChange,
  onSave,
  onPickWww,
  onOpenSite,
  onRegenerate,
  onCopyMysql,
}: {
  state: StackState
  loading: boolean
  wwwSubdir: string
  onWwwSubdirChange: (v: string) => void
  onSave: () => void
  onPickWww: () => void
  onOpenSite: () => void
  onRegenerate: () => void
  onCopyMysql: () => void
}) {
  const info = state.env_info
  const nginxRunning = state.components.find((c) => c.id === 'nginx')?.status === 'running'

  return (
    <section className="rounded-2xl border border-border bg-card shadow-sm overflow-hidden">
      <div className="px-4 py-3 border-b border-border/60 bg-secondary/30">
        <h3 className="text-sm font-medium flex items-center gap-2">
          <span className="text-base">⚙</span> 环境配置
          <span className="text-[10px] font-normal text-muted-foreground">（参照 XAMPP 常用项）</span>
        </h3>
      </div>
      <div className="p-4 space-y-4">
        <div className="space-y-2">
          <label className="text-xs text-muted-foreground">网站根目录（相对环境根目录）</label>
          <div className="flex gap-2">
            <input
              value={wwwSubdir}
              onChange={(e) => onWwwSubdirChange(e.target.value)}
              placeholder="www/default"
              className="flex-1 rounded-xl border border-border bg-background px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-primary/30"
            />
            <ActionBtn variant="default" disabled={loading || !state.install_root} onClick={onPickWww}>
              浏览
            </ActionBtn>
            <ActionBtn variant="primary" disabled={loading || !state.install_root} onClick={onSave}>
              保存
            </ActionBtn>
          </div>
          {info.www_root && (
            <p className="text-[11px] text-muted-foreground font-mono break-all">完整路径：{info.www_root}</p>
          )}
        </div>

        <div className="grid gap-3 sm:grid-cols-2">
          <div className="rounded-xl border border-border/60 bg-secondary/20 p-3 space-y-2">
            <div className="text-xs font-medium text-foreground">Web 站点</div>
            {info.site_url ? (
              <div className="flex flex-wrap items-center gap-2">
                <code className="text-xs bg-background px-2 py-1 rounded-md border border-border">{info.site_url}</code>
                <ActionBtn variant="primary" disabled={loading || !nginxRunning} onClick={onOpenSite}>
                  打开网站
                </ActionBtn>
              </div>
            ) : (
              <p className="text-xs text-muted-foreground">安装并启动 Nginx 后可访问</p>
            )}
            <p className="text-[11px] text-muted-foreground">Nginx 监听端口 · PHP 通过 FastCGI 联动</p>
          </div>

          <div className="rounded-xl border border-border/60 bg-secondary/20 p-3 space-y-2">
            <div className="text-xs font-medium text-foreground">MySQL 连接</div>
            {info.mysql_port ? (
              <>
                <div className="text-[11px] font-mono space-y-0.5 text-muted-foreground">
                  <div>host={info.mysql_host} · port={info.mysql_port}</div>
                  <div>user={info.mysql_user} · password=（空）</div>
                </div>
                <ActionBtn variant="default" disabled={loading} onClick={onCopyMysql}>
                  复制连接信息
                </ActionBtn>
              </>
            ) : (
              <p className="text-xs text-muted-foreground">安装 MySQL 后可用，默认 root 无密码</p>
            )}
          </div>
        </div>

        {(info.php_fastcgi || info.redis_addr) && (
          <div className="flex flex-wrap gap-3 text-[11px] text-muted-foreground">
            {info.php_fastcgi && <span>PHP FastCGI：<code className="font-mono">{info.php_fastcgi}</code></span>}
            {info.redis_addr && <span>Redis：<code className="font-mono">{info.redis_addr}</code></span>}
          </div>
        )}

        <div className="flex flex-wrap gap-2 pt-1 border-t border-border/60">
          <ActionBtn variant="ghost" disabled={loading || !state.install_root} onClick={onRegenerate}>
            重新生成配置文件
          </ActionBtn>
          <span className="text-[11px] text-muted-foreground self-center">
            修改端口或网站目录后会自动更新 my.ini / nginx / php.ini / redis.conf
          </span>
        </div>
      </div>
    </section>
  )
}

function StatPill({ label, value, accent }: { label: string; value: string | number; accent?: string }) {
  return (
    <div className="flex flex-col items-center justify-center rounded-xl bg-background/60 border border-border/50 px-4 py-2 min-w-[72px]">
      <span className={`text-lg font-semibold tabular-nums ${accent ?? 'text-foreground'}`}>{value}</span>
      <span className="text-[10px] text-muted-foreground mt-0.5">{label}</span>
    </div>
  )
}

export function StackPanel() {
  const [state, setState] = useState<StackState | null>(null)
  const [installRoot, setInstallRootInput] = useState('')
  const [loading, setLoading] = useState(false)
  const [message, setMessage] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({})
  const [localSources, setLocalSources] = useState<Record<string, string>>({})
  const [wwwSubdir, setWwwSubdir] = useState('www/default')

  const stats = useMemo(() => {
    const comps = state?.components ?? []
    return {
      installed: comps.filter((c) => c.installed).length,
      running: comps.filter((c) => c.status === 'running').length,
      total: comps.length,
    }
  }, [state])

  const refresh = useCallback(async () => {
    try {
      const next = await getStackState()
      setState((prev) => (prev && stackStateEqual(prev, next) ? prev : next))
      if (next.install_root) {
        setInstallRootInput((prev) => (prev === next.install_root ? prev : next.install_root!))
      }
      if (next.settings?.www_subdir) {
        setWwwSubdir((prev) => (prev === next.settings!.www_subdir ? prev : next.settings!.www_subdir))
      }
    } catch (e) {
      console.error(e)
    }
  }, [])

  useEffect(() => {
    refresh()
    let timer: ReturnType<typeof setInterval> | undefined

    const startPolling = () => {
      if (timer) return
      timer = setInterval(refresh, POLL_INTERVAL_MS)
    }
    const stopPolling = () => {
      if (!timer) return
      clearInterval(timer)
      timer = undefined
    }

    const onVisibility = () => {
      if (document.hidden) {
        stopPolling()
      } else {
        refresh()
        startPolling()
      }
    }

    startPolling()
    document.addEventListener('visibilitychange', onVisibility)
    return () => {
      stopPolling()
      document.removeEventListener('visibilitychange', onVisibility)
    }
  }, [refresh])

  useEffect(() => {
    const unlisten = listen<DownloadProgress>('stack-download-progress', (event) => {
      setProgress((prev) => ({ ...prev, [event.payload.component]: event.payload }))
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  useEffect(() => {
    if (!message && !error) return
    const t = setTimeout(() => {
      setMessage(null)
      setError(null)
    }, 4000)
    return () => clearTimeout(t)
  }, [message, error])

  const run = async (fn: () => Promise<void>) => {
    setLoading(true)
    setError(null)
    setMessage(null)
    try {
      await fn()
      await refresh()
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  const ensureRoot = () => {
    if (!installRoot.trim() && !state?.install_root) {
      throw new Error('请先设置环境安装目录')
    }
  }

  return (
    <div className="relative h-full flex flex-col bg-gradient-to-b from-secondary/30 to-background">
      {/* 页头 */}
      <header className="shrink-0 px-5 pt-5 pb-4 border-b border-border/80 bg-background">
        <div className="flex flex-wrap items-start justify-between gap-4 max-w-5xl">
          <div>
            <h2 className="text-xl font-semibold tracking-tight text-foreground">本地环境</h2>
            <p className="text-sm text-muted-foreground mt-1 max-w-lg">
              一键管理 MySQL · Nginx · PHP · Redis 绿色开发环境
            </p>
          </div>
          <div className="flex gap-2">
            <StatPill label="已安装" value={`${stats.installed}/${stats.total}`} />
            <StatPill label="已启动" value={stats.running} accent="text-emerald-600 dark:text-emerald-400" />
          </div>
        </div>
      </header>

      <div className="flex-1 overflow-y-auto">
        <div className="max-w-5xl mx-auto p-5 space-y-5 animate-fade-in">
          {/* 提示 */}
          {(error || message) && (
            <div
              className={`rounded-xl px-4 py-3 text-sm flex items-start gap-2 ${
                error
                  ? 'bg-red-500/10 text-red-700 dark:text-red-300 ring-1 ring-red-500/20'
                  : 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 ring-1 ring-emerald-500/20'
              }`}
            >
              <span>{error ? '✕' : '✓'}</span>
              <span>{error ?? message}</span>
            </div>
          )}

          {/* 全局配置 */}
          <section className="rounded-2xl border border-border bg-card shadow-sm overflow-hidden">
            <div className="px-4 py-3 border-b border-border/60 bg-secondary/30 flex flex-wrap items-center justify-between gap-2">
              <h3 className="text-sm font-medium flex items-center gap-2">
                <span className="text-base">📁</span> 环境根目录
              </h3>
              <div className="flex gap-2">
                <ActionBtn
                  variant="primary"
                  disabled={loading}
                  onClick={() => run(async () => { await startAllComponents(); setMessage('全部服务已启动') })}
                >
                  ▶ 一键启动
                </ActionBtn>
                <ActionBtn
                  variant="default"
                  disabled={loading}
                  onClick={() => run(async () => { await stopAllComponents(); setMessage('全部服务已停止') })}
                >
                  ■ 一键停止
                </ActionBtn>
              </div>
            </div>
            <div className="p-4 space-y-3">
              <div className="flex gap-2">
                <input
                  value={installRoot}
                  onChange={(e) => setInstallRootInput(e.target.value)}
                  placeholder="选择或输入目录，如 D:\dev-env"
                  className="flex-1 rounded-xl border border-border bg-background px-4 py-2.5 text-sm font-mono placeholder:font-sans focus:outline-none focus:ring-2 focus:ring-primary/30 transition-shadow"
                />
                <ActionBtn variant="default" disabled={loading} onClick={() => run(async () => {
                  const picked = await pickInstallRoot()
                  if (picked) {
                    setInstallRootInput(picked)
                    await setInstallRoot(picked)
                    setMessage('安装目录已设置')
                  }
                })} className="px-4">
                  浏览
                </ActionBtn>
                <ActionBtn variant="primary" disabled={loading} onClick={() => run(async () => {
                  await setInstallRoot(installRoot.trim())
                  setMessage('安装目录已保存')
                })} className="px-4">
                  保存
                </ActionBtn>
              </div>
              <div className="flex flex-wrap gap-4 text-xs">
                {state?.install_root && (
                  <button
                    type="button"
                    onClick={() => openInstallRoot().catch((e) => setError(String(e)))}
                    className="text-primary hover:underline inline-flex items-center gap-1"
                  >
                    打开根目录 ↗
                  </button>
                )}
                <button
                  type="button"
                  onClick={() => openWwwRoot().catch((e) => setError(String(e)))}
                  className="text-primary hover:underline inline-flex items-center gap-1"
                >
                  网站目录 www ↗
                </button>
              </div>
            </div>
          </section>

          {state && (
            <EnvSettingsPanel
              state={state}
              loading={loading}
              wwwSubdir={wwwSubdir}
              onWwwSubdirChange={setWwwSubdir}
              onSave={() => run(async () => {
                await updateStackSettings({ www_subdir: wwwSubdir.trim() })
                setMessage('网站目录已保存，Nginx 配置已更新')
              })}
              onPickWww={() => run(async () => {
                const picked = await pickWwwSubdir()
                if (picked) {
                  setWwwSubdir(picked)
                  await updateStackSettings({ www_subdir: picked })
                  setMessage('网站目录已更新')
                }
              })}
              onOpenSite={() => openSite().catch((e) => setError(String(e)))}
              onRegenerate={() => run(async () => {
                await regenerateConfigs()
                setMessage('配置文件已重新生成')
              })}
              onCopyMysql={() => {
                const info = state.env_info
                if (!info.mysql_port) return
                const text = `host=${info.mysql_host}\nport=${info.mysql_port}\nuser=${info.mysql_user}\npassword=\n`
                navigator.clipboard.writeText(text).then(() => setMessage('MySQL 连接信息已复制')).catch(() => setError('复制失败'))
              }}
            />
          )}

          {/* 服务网格 */}
          <div className="grid gap-4 sm:grid-cols-2">
            {state?.components.map((comp) => (
              <ServiceCard
                key={comp.id}
                component={comp}
                loading={loading}
                progress={progress[comp.id]}
                canInstall={!comp.installed && (comp.downloaded || Boolean(localSources[comp.id]))}
                onVersionChange={(versionId) => run(async () => {
                  await setComponentVersion(comp.id, versionId)
                })}
                localSource={localSources[comp.id]}
                onDownload={() => run(async () => {
                  ensureRoot()
                  if (!state?.install_root) await setInstallRoot(installRoot.trim())
                  await downloadComponent(comp.id, comp.selected_version_id)
                  setMessage(`${comp.name} 下载完成`)
                })}
                onPickLocal={() => run(async () => {
                  const picked = await pickComponentSource()
                  if (picked) {
                    setLocalSources((prev) => ({ ...prev, [comp.id]: picked }))
                    setMessage(`已选择 ${comp.name} 本地包`)
                  }
                })}
                onInstall={() => run(async () => {
                  ensureRoot()
                  if (!state?.install_root) await setInstallRoot(installRoot.trim())
                  await installComponent({
                    component: comp.id,
                    source_path: localSources[comp.id],
                    port: comp.default_port,
                    version_id: comp.selected_version_id,
                  })
                  setMessage(`${comp.name} 安装完成`)
                })}
                onStart={() => run(async () => {
                  await startComponent(comp.id)
                  setMessage(`${comp.name} 已启动`)
                })}
                onStop={() => run(async () => {
                  await stopComponent(comp.id)
                  setMessage(`${comp.name} 已停止`)
                })}
                onUninstall={() => run(async () => {
                  if (!confirm(`卸载 ${comp.name} 注册？不会删除已解压文件。`)) return
                  await uninstallComponent(comp.id)
                  setMessage(`${comp.name} 已卸载`)
                })}
                onSetPort={(port) => run(async () => {
                  await setComponentPort(comp.id, port)
                  setMessage(`${comp.name} 端口已更新为 ${port}`)
                })}
                onOpenConfig={() => openComponentConfig(comp.id).catch((e) => setError(String(e)))}
                onOpenLog={comp.log_path ? () => openComponentLog(comp.id).catch((e) => setError(String(e))) : undefined}
              />
            ))}
          </div>

          <p className="text-center text-[11px] text-muted-foreground pb-4">
            绿色安装 · 各组件可选版本 · 下载缓存于 downloads 目录
          </p>
        </div>
      </div>

      {loading && (
        <div className="absolute inset-0 bg-background/20 backdrop-blur-[1px] pointer-events-none flex items-start justify-center pt-32">
          <div className="rounded-full bg-background border border-border px-4 py-2 text-xs shadow-lg flex items-center gap-2">
            <span className="w-3 h-3 border-2 border-primary border-t-transparent rounded-full animate-spin" />
            处理中…
          </div>
        </div>
      )}
    </div>
  )
}
