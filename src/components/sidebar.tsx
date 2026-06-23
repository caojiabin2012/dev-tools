import type { ToolId } from '@/App'

interface SidebarProps {
  tools: { id: ToolId; name: string; icon: string }[]
  activeTool: ToolId
  onSelect: (id: ToolId) => void
}

export function Sidebar({ tools, activeTool, onSelect }: SidebarProps) {
  return (
    <aside className="w-56 border-r border-border bg-card flex flex-col">
      <div className="p-4 border-b border-border">
        <h1 className="text-lg font-semibold text-foreground flex items-center gap-2">
          <span className="text-xl">🧰</span>
          Tool Kit
        </h1>
        <p className="text-xs text-muted-foreground mt-1">开发者工具箱</p>
      </div>
      <nav className="flex-1 p-2 space-y-1">
        {tools.map((tool) => (
          <button
            key={tool.id}
            onClick={() => onSelect(tool.id)}
            className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
              activeTool === tool.id
                ? 'bg-accent text-accent-foreground font-medium'
                : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'
            }`}
          >
            <span className="font-mono text-base w-6 text-center">{tool.icon}</span>
            {tool.name}
          </button>
        ))}
      </nav>
      <div className="p-4 border-t border-border">
        <p className="text-xs text-muted-foreground text-center">v0.1.0</p>
      </div>
    </aside>
  )
}
