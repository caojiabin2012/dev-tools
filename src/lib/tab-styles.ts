/** 工具页最顶栏 Tab：与侧栏标题同高 h-14，保证分隔线对齐 */
export const toolTabBarClass =
  'flex h-14 shrink-0 items-center border-b border-border bg-background px-6'

/** 页面内二级 Tab 条：白底 */
export const subTabBarClass = 'shrink-0 border-b border-border bg-background px-6 py-3'

/** 工具页内容区：白底 */
export const toolContentClass = 'flex-1 overflow-hidden bg-background'

const tabActiveClass = 'bg-primary text-primary-foreground shadow-sm'
const tabInactiveClass = 'text-muted-foreground hover:bg-muted hover:text-foreground'

export function toolTabButtonClass(active: boolean): string {
  const base = 'rounded-lg px-4 py-2 text-sm font-medium transition-colors'
  return active ? `${base} ${tabActiveClass}` : `${base} ${tabInactiveClass}`
}

/** 顶栏内联 Tab（不撑满宽度） */
export function toolTabButtonInlineClass(active: boolean): string {
  return toolTabButtonClass(active)
}

/** 二级 Tab（白底条上的紧凑按钮） */
export function subTabButtonCompactClass(active: boolean): string {
  const base =
    'flex items-center gap-1.5 whitespace-nowrap rounded-lg px-3 py-1.5 text-sm font-medium transition-colors'
  return active ? `${base} ${tabActiveClass}` : `${base} ${tabInactiveClass}`
}

/** @deprecated 请用 subTabButtonCompactClass */
export function toolTabButtonCompactClass(active: boolean): string {
  return subTabButtonCompactClass(active)
}

export function subTabButtonClass(active: boolean): string {
  const base = 'flex-1 rounded-lg px-4 py-2.5 text-sm font-medium transition-colors'
  return active ? `${base} ${tabActiveClass}` : `${base} ${tabInactiveClass}`
}
