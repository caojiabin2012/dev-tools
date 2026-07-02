import { useState } from 'react'
import { EncodingTool } from '@/components/encoding'
import { GeneratorTool } from '@/components/generator'
import { IdCardTool } from '@/components/id-card'
import { DevTool } from '@/components/dev-tool'
import { toolTabBarClass, toolTabButtonInlineClass } from '@/lib/tab-styles'

type SubTool = 'encoding' | 'generator' | 'id-card' | 'dev-tool'

const tabs: { id: SubTool; name: string; icon: string }[] = [
  { id: 'encoding', name: '编码转换', icon: '🔄' },
  { id: 'generator', name: '生成器', icon: '⚡' },
  { id: 'id-card', name: '身份证', icon: '🪪' },
  { id: 'dev-tool', name: '正则/Cron', icon: '🛠️' },
]

export function DevToolsPanel() {
  const [active, setActive] = useState<SubTool>('encoding')

  return (
    <div className="h-full flex flex-col">
      <nav className={`${toolTabBarClass} px-4`}>
        <div className="flex gap-2">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActive(tab.id)}
              className={`flex items-center gap-1.5 ${toolTabButtonInlineClass(active === tab.id)}`}
            >
              <span className="text-base">{tab.icon}</span>
              {tab.name}
            </button>
          ))}
        </div>
      </nav>
      <div className="flex-1 overflow-hidden">
        <div className={active === 'encoding' ? 'h-full' : 'h-0 overflow-hidden'}>
          <EncodingTool />
        </div>
        <div className={active === 'generator' ? 'h-full' : 'h-0 overflow-hidden'}>
          <GeneratorTool />
        </div>
        <div className={active === 'id-card' ? 'h-full' : 'h-0 overflow-hidden'}>
          <IdCardTool />
        </div>
        <div className={active === 'dev-tool' ? 'h-full' : 'h-0 overflow-hidden'}>
          <DevTool />
        </div>
      </div>
    </div>
  )
}
