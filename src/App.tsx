import { useState, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { JsonFormatter } from '@/components/json-formatter'
import { ClipboardManager } from '@/components/clipboard'
import { Calculator } from '@/components/calculator'
import { Calendar } from '@/components/calendar'
import { IdCardTool } from '@/components/id-card'
import { EncodingTool } from '@/components/encoding'
import { GeneratorTool } from '@/components/generator'
import { ToastContainer } from '@/lib/toast'
import { DevTool } from '@/components/dev-tool'
import { Settings, type SettingsTab } from '@/components/settings'
import { Sidebar } from '@/components/sidebar'
import { Home } from '@/components/home'
import { useTheme } from '@/lib/use-theme'
import { useUpdate } from '@/lib/use-update'

export type ToolId =
  | 'home'
  | 'json-formatter' | 'clipboard' | 'calculator' | 'calendar'
  | 'id-card' | 'encoding' | 'generator' | 'dev-tool'
  | 'settings'

interface ToolGroup {
  name: string
  tools: { id: ToolId; name: string; icon: string }[]
}

const toolGroups: ToolGroup[] = [
  {
    name: '常用工具',
    tools: [
      { id: 'json-formatter', name: 'JSON 格式化', icon: '📝' },
      { id: 'clipboard', name: '剪切板', icon: '📋' },
      { id: 'calculator', name: '计算器', icon: '🧮' },
      { id: 'calendar', name: '日历', icon: '📅' },
    ],
  },
  {
    name: '编码转换',
    tools: [
      { id: 'encoding', name: '编码工具', icon: '🔄' },
    ],
  },
  {
    name: '生成器',
    tools: [
      { id: 'generator', name: '生成工具', icon: '⚡' },
    ],
  },
  {
    name: '查询解析',
    tools: [
      { id: 'id-card', name: '身份证工具', icon: '🪪' },
    ],
  },
  {
    name: '开发工具',
    tools: [
      { id: 'dev-tool', name: '开发工具', icon: '🛠️' },
    ],
  },
]

const allTools = toolGroups.flatMap(g => g.tools)

export default function App() {
  const [activeTool, setActiveTool] = useState<ToolId>('home')
  const [settingsTab, setSettingsTab] = useState<SettingsTab>('general')
  const [jsonFormatterInput, setJsonFormatterInput] = useState<string | undefined>()
  const { theme, setTheme } = useTheme()
  const { updateInfo, updateError, hasUpdate, checking, checked, checkUpdate } = useUpdate()

  const openSettings = (tab: SettingsTab = 'general') => {
    setSettingsTab(tab)
    setActiveTool('settings')
  }

  const openJsonFormatter = (text: string) => {
    setJsonFormatterInput(text)
    setActiveTool('json-formatter')
  }

  useEffect(() => {
    const unlisten = listen<string>('shortcut-triggered', (event) => {
      const toolId = event.payload as ToolId
      const toolIds = allTools.map(t => t.id)
      if (toolId && toolIds.includes(toolId)) {
        setActiveTool(toolId)
      }
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  return (
    <div className="flex h-screen bg-background">
      <Sidebar
        toolGroups={toolGroups}
        activeTool={activeTool}
        onSelect={setActiveTool}
        onOpenSettings={() => openSettings('general')}
        hasUpdate={hasUpdate}
        onUpdateClick={() => openSettings('about')}
      />
      <main className="flex-1 overflow-hidden">
        {activeTool === 'home' && (
          <Home toolGroups={toolGroups} onSelect={setActiveTool} />
        )}
        {activeTool === 'json-formatter' && (
          <JsonFormatter key={jsonFormatterInput ?? 'empty'} initialInput={jsonFormatterInput} />
        )}
        {activeTool === 'clipboard' && (
          <ClipboardManager onFormatJson={openJsonFormatter} />
        )}
        {activeTool === 'calculator' && <Calculator />}
        <div className={activeTool === 'calendar' ? 'h-full' : 'h-0 overflow-hidden'}>
          <Calendar />
        </div>
        <div className={activeTool === 'id-card' ? 'h-full' : 'h-0 overflow-hidden'}>
          <IdCardTool />
        </div>
        <div className={activeTool === 'encoding' ? 'h-full' : 'h-0 overflow-hidden'}>
          <EncodingTool />
        </div>
        <div className={activeTool === 'generator' ? 'h-full' : 'h-0 overflow-hidden'}>
          <GeneratorTool />
        </div>
        <div className={activeTool === 'dev-tool' ? 'h-full' : 'h-0 overflow-hidden'}>
          <DevTool />
        </div>
        {activeTool === 'settings' && (
          <Settings
            activeTab={settingsTab}
            onTabChange={setSettingsTab}
            theme={theme}
            onThemeChange={setTheme}
            updateInfo={updateInfo}
            updateError={updateError}
            checkingUpdate={checking}
            updateChecked={checked}
            onCheckUpdate={checkUpdate}
          />
        )}
      </main>
      <ToastContainer />
    </div>
  )
}
