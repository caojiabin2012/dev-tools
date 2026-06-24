import { useState } from 'react'
import { JsonFormatter } from '@/components/json-formatter'
import { ClipboardManager } from '@/components/clipboard'
import { Settings, type SettingsTab } from '@/components/settings'
import { Sidebar } from '@/components/sidebar'
import { useTheme } from '@/lib/use-theme'
import { useUpdate } from '@/lib/use-update'

export type ToolId = 'json-formatter' | 'clipboard' | 'settings'

const tools = [
  { id: 'json-formatter' as ToolId, name: 'JSON 格式化', icon: '📝' },
  { id: 'clipboard' as ToolId, name: '剪切板', icon: '📋' },
]

export default function App() {
  const [activeTool, setActiveTool] = useState<ToolId>('json-formatter')
  const [settingsTab, setSettingsTab] = useState<SettingsTab>('general')
  const [jsonFormatterInput, setJsonFormatterInput] = useState<string | undefined>()
  const { theme, setTheme } = useTheme()
  const { updateInfo, hasUpdate, checking, checked, checkUpdate } = useUpdate()

  const openSettings = (tab: SettingsTab = 'general') => {
    setSettingsTab(tab)
    setActiveTool('settings')
  }

  const openJsonFormatter = (text: string) => {
    setJsonFormatterInput(text)
    setActiveTool('json-formatter')
  }

  return (
    <div className="flex h-screen bg-background">
      <Sidebar
        tools={tools}
        activeTool={activeTool}
        onSelect={setActiveTool}
        onOpenSettings={() => openSettings('general')}
        hasUpdate={hasUpdate}
        onUpdateClick={() => openSettings('about')}
      />
      <main className="flex-1 overflow-hidden">
        {activeTool === 'json-formatter' && (
          <JsonFormatter key={jsonFormatterInput ?? 'empty'} initialInput={jsonFormatterInput} />
        )}
        {activeTool === 'clipboard' && (
          <ClipboardManager onFormatJson={openJsonFormatter} />
        )}
        {activeTool === 'settings' && (
          <Settings
            activeTab={settingsTab}
            onTabChange={setSettingsTab}
            theme={theme}
            onThemeChange={setTheme}
            updateInfo={updateInfo}
            checkingUpdate={checking}
            updateChecked={checked}
            onCheckUpdate={checkUpdate}
          />
        )}
      </main>
    </div>
  )
}
