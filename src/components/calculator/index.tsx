import { useState } from 'react'
import { CalculatorPanel } from './calculator-panel'
import { ConverterPanel } from './converter'
import { toolTabBarClass, toolTabButtonClass, toolContentClass } from '@/lib/tab-styles'

type CalculatorTab = 'calculator' | 'converter'

const tabs: { id: CalculatorTab; name: string; icon: string }[] = [
  { id: 'calculator', name: '计算器', icon: '📱' },
  { id: 'converter', name: '换算', icon: '🔄' },
]

export function Calculator() {
  const [activeTab, setActiveTab] = useState<CalculatorTab>('calculator')

  return (
    <div className="flex h-full flex-col bg-background">
      <div className={toolTabBarClass}>
        <div className="flex gap-2">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={toolTabButtonClass(activeTab === tab.id)}
            >
              <span className="mr-1.5">{tab.icon}</span>
              {tab.name}
            </button>
          ))}
        </div>
      </div>
      <div className={toolContentClass}>
        {activeTab === 'calculator' ? <CalculatorPanel /> : <ConverterPanel />}
      </div>
    </div>
  )
}
