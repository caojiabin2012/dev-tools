import { useState, useRef, useCallback, useMemo } from 'react'
import { evaluate, formatResult } from './formula-parser'

interface HistoryItem {
  expr: string
  result: string
}

const OPERATORS = new Set(['÷', '×', '−', '＋'])

const calcKeyBase =
  'flex-1 select-none rounded-2xl font-medium transition-all active:scale-95'
const calcKeyDefault =
  'bg-secondary text-secondary-foreground hover:bg-accent'
const calcKeyOperator =
  'bg-primary text-primary-foreground shadow-sm hover:bg-primary/90'
const calcToolBtnClass =
  'rounded-lg px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground'

function calcKeyClass(label: string, size: 'sm' | 'lg' = 'lg'): string {
  const sizeClass = size === 'sm' ? 'py-2 text-sm' : 'py-3.5 text-xl'
  const style = OPERATORS.has(label) ? calcKeyOperator : calcKeyDefault
  return `${calcKeyBase} ${sizeClass} ${style}`
}

export function CalculatorPanel() {
  const [expression, setExpression] = useState('')
  const [angleMode, setAngleMode] = useState<'deg' | 'rad'>('deg')
  const [showScientific, setShowScientific] = useState(false)
  const [history, setHistory] = useState<HistoryItem[]>([])
  const [showHistory, setShowHistory] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  const { result, error } = useMemo(() => {
    if (!expression.trim()) {
      return { result: '', error: '' }
    }
    try {
      const value = evaluate(expression, angleMode)
      return { result: formatResult(value), error: '' }
    } catch (e) {
      return { result: '', error: (e as Error).message }
    }
  }, [expression, angleMode])

  const insertAtCursor = useCallback((text: string) => {
    const input = inputRef.current
    if (input) {
      const start = input.selectionStart || expression.length
      const end = input.selectionEnd || expression.length
      const newExpr = expression.slice(0, start) + text + expression.slice(end)
      setExpression(newExpr)
      setTimeout(() => {
        input.selectionStart = input.selectionEnd = start + text.length
        input.focus()
      }, 0)
    } else {
      setExpression(prev => prev + text)
    }
  }, [expression])

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      if (result && !error) {
        setHistory(prev => [{ expr: expression, result }, ...prev].slice(0, 50))
        setExpression(result.replace(/,/g, ''))
      }
    } else if (e.key === 'Escape') {
      setExpression('')
    }
  }, [expression, result, error])

  const clearAll = () => {
    setExpression('')
  }

  const backspace = () => {
    const input = inputRef.current
    if (input) {
      const start = input.selectionStart || expression.length
      const end = input.selectionEnd || expression.length
      if (start !== end) {
        setExpression(expression.slice(0, start) + expression.slice(end))
      } else if (start > 0) {
        setExpression(expression.slice(0, start - 1) + expression.slice(start))
        setTimeout(() => {
          input.selectionStart = input.selectionEnd = start - 1
          input.focus()
        }, 0)
      }
    } else {
      setExpression(prev => prev.slice(0, -1))
    }
  }

  const toggleAngleMode = () => {
    setAngleMode(prev => prev === 'deg' ? 'rad' : 'deg')
  }

  const handleScientificButton = (label: string) => {
    const actionMap: Record<string, () => void> = {
      '2nd': () => {},
      'π': () => insertAtCursor('π'),
      'e': () => insertAtCursor('e'),
      '%': () => insertAtCursor('%'),
      'AC': clearAll,
      'x²': () => insertAtCursor('^2'),
      'xʸ': () => insertAtCursor('^('),
      '√x': () => insertAtCursor('√('),
      'n!': () => insertAtCursor('!'),
      'EE': () => insertAtCursor('e'),
      '1/x': () => insertAtCursor('1/('),
      'ln': () => insertAtCursor('ln('),
      'log': () => insertAtCursor('log('),
      'eˣ': () => insertAtCursor('e^('),
      '10ˣ': () => insertAtCursor('10^('),
      'sin': () => insertAtCursor('sin('),
      'cos': () => insertAtCursor('cos('),
      'tan': () => insertAtCursor('tan('),
      'Deg': toggleAngleMode,
      'Rad': toggleAngleMode,
      '(': () => insertAtCursor('('),
      ')': () => insertAtCursor(')'),
      'sin⁻¹': () => insertAtCursor('asin('),
      'cos⁻¹': () => insertAtCursor('acos('),
      'tan⁻¹': () => insertAtCursor('atan('),
    }
    actionMap[label]?.()
  }

  const angleLabel = angleMode === 'deg' ? 'Deg' : 'Rad'

  const scientificRows = [
    ['2nd', 'π', 'e', '%', 'AC'],
    ['x²', 'xʸ', '√x', 'n!', 'EE'],
    ['1/x', 'ln', 'log', 'eˣ', '10ˣ'],
    ['sin', 'cos', 'tan', angleLabel, '('],
    ['sin⁻¹', 'cos⁻¹', 'tan⁻¹', '(', ')'],
  ]

  const standardButtons = [
    ['7', '8', '9', '÷'],
    ['4', '5', '6', '×'],
    ['1', '2', '3', '−'],
    ['0', '00', '.', '＋'],
  ]

  const handleStandardButton = (btn: string) => {
    switch (btn) {
      case '÷': insertAtCursor('/'); break
      case '×': insertAtCursor('*'); break
      case '−': insertAtCursor('-'); break
      case '＋': insertAtCursor('+'); break
      default: insertAtCursor(btn)
    }
  }

  return (
    <div className="flex h-full flex-col bg-background">
      <div className="flex items-center gap-2 border-b border-border bg-background p-4">
        <button
          onClick={() => setShowScientific(!showScientific)}
          className={calcToolBtnClass}
        >
          {showScientific ? '收起科学面板' : '科学面板'}
        </button>
        <button
          onClick={toggleAngleMode}
          className={calcToolBtnClass}
        >
          {angleMode === 'deg' ? 'Deg' : 'Rad'}
        </button>
        <div className="flex-1" />
        <button
          onClick={() => setShowHistory(!showHistory)}
          className={calcToolBtnClass}
        >
          历史记录
        </button>
      </div>

      <div className="border-b border-border bg-background p-4">
        <input
          ref={inputRef}
          type="text"
          value={expression}
          onChange={(e) => setExpression(e.target.value)}
          onKeyDown={handleKeyDown}
          className="w-full text-2xl font-mono bg-transparent outline-none text-foreground placeholder-muted-foreground"
          placeholder="输入公式..."
          autoFocus
        />
        <div className={`text-right text-xl mt-2 font-mono ${error ? 'text-destructive' : 'text-muted-foreground'}`}>
          {error ? error : result ? `= ${result}` : ''}
        </div>
      </div>

      {showScientific && (
        <div className="border-b border-border bg-background px-4 py-3">
          <div className="mx-auto max-w-sm space-y-2">
          {scientificRows.map((row, ri) => (
            <div key={ri} className="flex gap-2">
              {row.map((label) => (
                <button
                  key={label}
                  onClick={() => handleScientificButton(label)}
                  className={calcKeyClass(label, 'sm')}
                >
                  {label}
                </button>
              ))}
            </div>
          ))}
          </div>
        </div>
      )}

      <div className="flex-1 bg-background px-4 py-3">
        <div className="mx-auto max-w-sm space-y-2">
        {standardButtons.map((row, ri) => (
          <div key={ri} className="flex gap-2">
            {row.map((btn) => (
              <button
                key={btn}
                onClick={() => handleStandardButton(btn)}
                className={calcKeyClass(btn)}
              >
                {btn}
              </button>
            ))}
          </div>
        ))}
        <div className="flex gap-2">
          <button onClick={() => insertAtCursor('(')} className={calcKeyClass('(')}>
            (
          </button>
          <button onClick={() => insertAtCursor(')')} className={calcKeyClass(')')}>
            )
          </button>
          <button onClick={backspace} className={calcKeyClass('⌫')}>
            ⌫
          </button>
          <button
            onClick={() => {
              if (result && !error) {
                setHistory(prev => [{ expr: expression, result }, ...prev].slice(0, 50))
                setExpression(result.replace(/,/g, ''))
              }
            }}
            className={`${calcKeyBase} py-3.5 text-xl ${calcKeyOperator}`}
          >
            ＝
          </button>
        </div>
        </div>
      </div>

      {showHistory && history.length > 0 && (
        <div className="border-t border-border p-4 max-h-48 overflow-y-auto">
          <h3 className="text-sm font-medium text-muted-foreground mb-2">历史记录</h3>
          {history.map((item, i) => (
            <div
              key={i}
              onClick={() => {
                setExpression(item.result.replace(/,/g, ''))
                setShowHistory(false)
              }}
              className="py-1 cursor-pointer hover:bg-accent/50 rounded px-2 -mx-2"
            >
              <span className="text-muted-foreground">{item.expr}</span>
              <span className="text-foreground"> = {item.result}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
