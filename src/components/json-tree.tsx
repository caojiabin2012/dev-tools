import { useState, useCallback, useMemo } from 'react'

interface JsonTreeProps {
  value: unknown
  expandAll: boolean
  indent: number
  isLast?: boolean
  depth?: number
}

export function JsonTree({ value, expandAll, indent, isLast = true, depth = 0 }: JsonTreeProps) {
  const type = useMemo(() => {
    if (value === null) return 'null'
    if (Array.isArray(value)) return 'array'
    return typeof value
  }, [value])

  if (type === 'object' || type === 'array') {
    return (
      <CollapsibleNode
        value={value}
        type={type}
        expandAll={expandAll}
        indent={indent}
        isLast={isLast}
        depth={depth}
      />
    )
  }

  return <PrimitiveValue value={value} type={type} isLast={isLast} />
}

function CollapsibleNode({
  value,
  type,
  expandAll,
  indent,
  isLast,
  depth,
}: {
  value: unknown
  type: string
  expandAll: boolean
  indent: number
  isLast: boolean
  depth: number
}) {
  const [expanded, setExpanded] = useState(expandAll)

  const entries = useMemo(() => {
    if (type === 'array') {
      return (value as unknown[]).map((v, i) => ({ key: i, value: v }))
    }
    return Object.entries(value as Record<string, unknown>).map(([k, v]) => ({
      key: k,
      value: v,
    }))
  }, [value, type])

  const toggle = useCallback(() => setExpanded((prev) => !prev), [])

  const prefix = type === 'array' ? '[' : '{'
  const suffix = type === 'array' ? ']' : '}'
  const bracketColor = type === 'array' ? 'text-blue-400' : 'text-amber-400'

  const indentation = '  '.repeat(depth)
  const childIndentation = '  '.repeat(depth + 1)

  if (!expanded) {
    const count = entries.length
    const label = type === 'array' ? `${count} items` : `${count} keys`
    return (
      <div className="leading-relaxed">
        <span style={{ paddingLeft: depth * indent * 4 }}>{indentation}</span>
        <button
          onClick={toggle}
          className="inline-flex items-center gap-1 hover:bg-accent/50 rounded px-0.5 -ml-0.5"
        >
          <span className="text-muted-foreground text-xs">▶</span>
          <span className={`${bracketColor}`}>{prefix}</span>
          <span className="text-muted-foreground text-xs italic">... {label} ...</span>
          <span className={`${bracketColor}`}>{suffix}</span>
          {!isLast && <span className="text-muted-foreground">,</span>}
        </button>
      </div>
    )
  }

  return (
    <div className="leading-relaxed">
      <div>
        <span style={{ paddingLeft: depth * indent * 4 }}>{indentation}</span>
        <button
          onClick={toggle}
          className="inline-flex items-center gap-1 hover:bg-accent/50 rounded px-0.5 -ml-0.5"
        >
          <span className="text-muted-foreground text-xs">▼</span>
          <span className={`${bracketColor}`}>{prefix}</span>
        </button>
      </div>
      {entries.map((entry, i) => (
        <div key={entry.key}>
          <span style={{ paddingLeft: (depth + 1) * indent * 4 }}>{childIndentation}</span>
          {type === 'object' && (
            <>
              <span className="text-purple-400">"{String(entry.key)}"</span>
              <span className="text-muted-foreground">: </span>
            </>
          )}
          <JsonTree
            value={entry.value}
            expandAll={expandAll}
            indent={indent}
            isLast={i === entries.length - 1}
            depth={depth + 1}
          />
        </div>
      ))}
      <div>
        <span style={{ paddingLeft: depth * indent * 4 }}>{indentation}</span>
        <span className={`${bracketColor}`}>{suffix}</span>
        {!isLast && <span className="text-muted-foreground">,</span>}
      </div>
    </div>
  )
}

function PrimitiveValue({
  value,
  type,
  isLast,
}: {
  value: unknown
  type: string
  isLast: boolean
}) {
  const colorClass = useMemo(() => {
    switch (type) {
      case 'string':
        return 'text-green-400'
      case 'number':
        return 'text-orange-400'
      case 'boolean':
        return 'text-cyan-400'
      case 'null':
        return 'text-red-400'
      default:
        return 'text-foreground'
    }
  }, [type])

  const display = useMemo(() => {
    if (type === 'null') return 'null'
    if (type === 'string') return `"${value}"`
    return String(value)
  }, [value, type])

  return (
    <span className={`${colorClass}`}>
      {display}
      {!isLast && <span className="text-muted-foreground">,</span>}
    </span>
  )
}
