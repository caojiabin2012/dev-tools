import { useState } from 'react'
import { JsonFormatter } from '@/components/json-formatter'
import { Sidebar } from '@/components/sidebar'

export type ToolId = 'json-formatter'

const tools = [
  { id: 'json-formatter' as ToolId, name: 'JSON 格式化', icon: '{ }' },
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
      </main>
    </div>
  )
}
