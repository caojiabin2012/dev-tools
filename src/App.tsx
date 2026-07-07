import { useState, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { JsonFormatter } from '@/components/json-formatter'
import { ClipboardManager } from '@/components/clipboard'
import { Calculator } from '@/components/calculator'
import { IdCardTool } from '@/components/id-card'
import { EncodingTool } from '@/components/encoding'
import { GeneratorTool } from '@/components/generator'
import { DevTool } from '@/components/dev-tool'
import { LifeKnowledgeTool } from '@/components/life-knowledge'
import { QrcodeTool } from '@/components/qrcode'
import { ToastContainer } from '@/lib/toast'

import { Settings, type SettingsTab } from '@/components/settings'
import { Sidebar } from '@/components/sidebar'
import { Home } from '@/components/home'
import { useTheme } from '@/lib/use-theme'
import { useUpdate } from '@/lib/use-update'

export type ToolId =
  | 'home'
  | 'json-formatter' | 'clipboard' | 'calculator' | 'calendar'
  | 'id-card' | 'encoding' | 'generator' | 'dev-tool' | 'qrcode' | 'life-knowledge'
  | 'settings'

interface ToolGroup {
  name: string
  tools: { id: ToolId; name: string; icon: string }[]
}

const toolGroups: ToolGroup[] = [
  {
    name: '',
    tools: [
      { id: 'clipboard', name: '剪切板', icon: '📋' },
      { id: 'json-formatter', name: 'JSON 格式化', icon: '📝' },
      { id: 'calculator', name: '计算器', icon: '🧮' },
      { id: 'calendar', name: '日历', icon: '📅' },
      { id: 'encoding', name: '编码转换', icon: '🔄' },
      { id: 'generator', name: '生成器', icon: '⚡' },
      { id: 'qrcode', name: '二维码', icon: '🔳' },
      { id: 'dev-tool', name: '表达式', icon: '🛠️' },
      { id: 'id-card', name: '身份证', icon: '🪪' },
      { id: 'life-knowledge', name: '生活常识', icon: '📖' },
    ],
  },
]

/** 暂不展示的工具入口，功能代码保留 */
const hiddenTools: ToolId[] = ['calendar']

function filterToolGroups(groups: ToolGroup[], hidden: ToolId[]): ToolGroup[] {
  return groups
    .map((group) => ({
      ...group,
      tools: group.tools.filter((tool) => !hidden.includes(tool.id)),
    }))
    .filter((group) => group.tools.length > 0)
}

const visibleToolGroups = filterToolGroups(toolGroups, hiddenTools)
const allTools = visibleToolGroups.flatMap((g) => g.tools)

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
        toolGroups={visibleToolGroups}
        activeTool={activeTool}
        onSelect={setActiveTool}
        onOpenSettings={() => openSettings('general')}
        hasUpdate={hasUpdate}
        onUpdateClick={() => openSettings('about')}
      />
      <main className="flex-1 overflow-hidden">
        {activeTool === 'home' && (
          <Home toolGroups={visibleToolGroups} onSelect={setActiveTool} />
        )}
        {activeTool === 'json-formatter' && (
          <JsonFormatter key={jsonFormatterInput ?? 'empty'} initialInput={jsonFormatterInput} />
        )}
        {activeTool === 'clipboard' && (
          <ClipboardManager onFormatJson={openJsonFormatter} />
        )}
        {activeTool === 'calculator' && <Calculator />}
        <div className={activeTool === 'id-card' ? 'h-full' : 'h-0 overflow-hidden'}>
          <IdCardTool />
        </div>
        <div className={activeTool === 'encoding' ? 'h-full' : 'h-0 overflow-hidden'}>
          <EncodingTool />
        </div>
        <div className={activeTool === 'generator' ? 'h-full' : 'h-0 overflow-hidden'}>
          <GeneratorTool />
        </div>
        <div className={activeTool === 'life-knowledge' ? 'h-full' : 'h-0 overflow-hidden'}>
          <LifeKnowledgeTool />
        </div>
        <div className={activeTool === 'qrcode' ? 'h-full' : 'h-0 overflow-hidden'}>
          <QrcodeTool />
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
