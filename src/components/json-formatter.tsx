import { useState, useCallback } from 'react'

type IndentSize = 2 | 4

export function JsonFormatter() {
  const [input, setInput] = useState('')
  const [output, setOutput] = useState('')
  const [error, setError] = useState('')
  const [indent, setIndent] = useState<IndentSize>(2)
  const [copied, setCopied] = useState(false)

  const format = useCallback(() => {
    setError('')
    if (!input.trim()) {
      setOutput('')
      return
    }
    try {
      const parsed = JSON.parse(input)
      setOutput(JSON.stringify(parsed, null, indent))
    } catch (e) {
      setError((e as Error).message)
      setOutput('')
    }
  }, [input, indent])

  const compress = useCallback(() => {
    setError('')
    if (!input.trim()) {
      setOutput('')
      return
    }
    try {
      const parsed = JSON.parse(input)
      setOutput(JSON.stringify(parsed))
    } catch (e) {
      setError((e as Error).message)
      setOutput('')
    }
  }, [input])

  const copyToClipboard = useCallback(async () => {
    if (!output) return
    try {
      await navigator.clipboard.writeText(output)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch {
      const textarea = document.createElement('textarea')
      textarea.value = output
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      document.body.removeChild(textarea)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }, [output])

  const clear = useCallback(() => {
    setInput('')
    setOutput('')
    setError('')
  }, [])

  const handlePaste = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText()
      setInput(text)
    } catch {
      // clipboard read failed silently
    }
  }, [])

  return (
    <div className="h-full flex flex-col p-4 gap-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-foreground">JSON 格式化</h2>
        <div className="flex items-center gap-2">
          <label className="text-sm text-muted-foreground">缩进:</label>
          <select
            value={indent}
            onChange={(e) => setIndent(Number(e.target.value) as IndentSize)}
            className="px-2 py-1 text-sm border border-input rounded-md bg-card text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          >
            <option value={2}>2 空格</option>
            <option value={4}>4 空格</option>
          </select>
        </div>
      </div>

      <div className="flex-1 grid grid-cols-2 gap-4 min-h-0">
        <div className="flex flex-col min-h-0">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-foreground">输入</span>
            <div className="flex gap-1">
              <button
                onClick={handlePaste}
                className="px-2 py-1 text-xs text-muted-foreground hover:text-foreground hover:bg-accent rounded transition-colors"
              >
                粘贴
              </button>
              <button
                onClick={clear}
                className="px-2 py-1 text-xs text-muted-foreground hover:text-foreground hover:bg-accent rounded transition-colors"
              >
                清空
              </button>
            </div>
          </div>
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="在此粘贴或输入 JSON..."
            className="flex-1 resize-none rounded-lg border border-input bg-card p-3 font-mono text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring min-h-0"
            spellCheck={false}
          />
        </div>

        <div className="flex flex-col min-h-0">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-foreground">输出</span>
            <button
              onClick={copyToClipboard}
              disabled={!output}
              className="px-2 py-1 text-xs text-muted-foreground hover:text-foreground hover:bg-accent rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {copied ? '已复制!' : '复制'}
            </button>
          </div>
          <textarea
            value={error ? `错误: ${error}` : output}
            readOnly
            placeholder="格式化结果..."
            className={`flex-1 resize-none rounded-lg border p-3 font-mono text-sm min-h-0 focus:outline-none ${
              error
                ? 'border-destructive bg-destructive/10 text-destructive'
                : 'border-input bg-muted text-foreground'
            }`}
            spellCheck={false}
          />
        </div>
      </div>

      <div className="flex gap-2">
        <button
          onClick={format}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors"
        >
          格式化
        </button>
        <button
          onClick={compress}
          className="px-4 py-2 bg-secondary text-secondary-foreground rounded-lg text-sm font-medium hover:bg-secondary/80 transition-colors"
        >
          压缩
        </button>
      </div>
    </div>
  )
}
