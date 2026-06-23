import { useState } from 'react'
import { JsonFormatter } from '@/components/json-formatter'
import { ClipboardManager } from '@/components/clipboard'
import { Sidebar } from '@/components/sidebar'

export type ToolId = 'json-formatter' | 'clipboard'

const tools = [
  { id: 'json-formatter' as ToolId, name: 'JSON 格式化', icon: '{ }' },
  { id: 'clipboard' as ToolId, name: '剪切板', icon: '📋' },
]

export default function App() {
  const [activeTool, setActiveTool] = useState<ToolId>('json-formatter')

  return (
    <div className="flex h-screen bg-background">
      <Sidebar
        tools={tools}
        activeTool={activeTool}
        onSelect={setActiveTool}
      />
      <main className="flex-1 overflow-hidden">
        {activeTool === 'json-formatter' && <JsonFormatter />}
        {activeTool === 'clipboard' && <ClipboardManager />}
      </main>
    </div>
  )
}
