/** Syntax-colored JSON viewer/editor for the Skills IDE. */

import { useRef, useMemo, useCallback } from 'react'
import { CircleCheck, CircleAlert, TriangleAlert } from 'lucide-react'
import { SchemaHints } from './SchemaHints'

interface Props {
  content: string
  tabId?: string
  filePath?: string
  onContentChange?: (id: string, content: string) => void
  onSave?: (id: string) => void
}

interface JsonToken {
  type: 'key' | 'string' | 'number' | 'boolean' | 'null' | 'punctuation'
  text: string
}

/* VS Code dark-theme-inspired palette with light-mode variants. */
const TOKEN_CLASSES: Record<JsonToken['type'], string> = {
  key: 'text-[#0451a5] dark:text-[#9cdcfe]',
  string: 'text-[#a31515] dark:text-[#ce9178]',
  number: 'text-[#098658] dark:text-[#b5cea8]',
  boolean: 'text-[#0000ff] dark:text-[#569cd6]',
  null: 'text-[#0000ff] dark:text-[#569cd6]',
  punctuation: 'text-muted-foreground',
}

/** Tokenize a pretty-printed JSON string into typed segments for coloring. */
function tokenizeJson(json: string): JsonToken[][] {
  const lines = json.split('\n')
  return lines.map((line) => {
    const tokens: JsonToken[] = []
    let i = 0

    while (i < line.length) {
      const ch = line[i]

      if (ch === ' ' || ch === '\t') {
        let ws = ''
        while (i < line.length && (line[i] === ' ' || line[i] === '\t')) {
          ws += line[i]
          i++
        }
        tokens.push({ type: 'punctuation', text: ws })
        continue
      }

      if ('{[}],: '.includes(ch) && ch !== ' ') {
        tokens.push({ type: 'punctuation', text: ch })
        i++
        continue
      }

      if (ch === '"') {
        let str = '"'
        i++
        while (i < line.length) {
          if (line[i] === '\\' && i + 1 < line.length) {
            str += line[i] + line[i + 1]
            i += 2
            continue
          }
          str += line[i]
          if (line[i] === '"') { i++; break }
          i++
        }

        let lookAhead = i
        while (lookAhead < line.length && line[lookAhead] === ' ') lookAhead++
        const isKey = lookAhead < line.length && line[lookAhead] === ':'

        tokens.push({ type: isKey ? 'key' : 'string', text: str })
        continue
      }

      if (ch === '-' || (ch >= '0' && ch <= '9')) {
        let num = ''
        while (i < line.length && /[\d.eE+-]/.test(line[i])) {
          num += line[i]
          i++
        }
        tokens.push({ type: 'number', text: num })
        continue
      }

      if (line.slice(i, i + 4) === 'true') {
        tokens.push({ type: 'boolean', text: 'true' })
        i += 4
        continue
      }
      if (line.slice(i, i + 5) === 'false') {
        tokens.push({ type: 'boolean', text: 'false' })
        i += 5
        continue
      }
      if (line.slice(i, i + 4) === 'null') {
        tokens.push({ type: 'null', text: 'null' })
        i += 4
        continue
      }

      tokens.push({ type: 'punctuation', text: ch })
      i++
    }

    return tokens
  })
}

const VALID_TYPES = new Set(['string', 'bool', 'enum', 'array', 'object'])
const VALID_STORAGE = new Set(['global', 'local', 'project'])

interface VarsWarning {
  key: string
  message: string
  line: number
  severity: 'error' | 'warning'
}

/** Find the approximate line number where a top-level key appears in raw JSON text. */
function findKeyLine(content: string, key: string): number {
  const pattern = `"${key}"`
  const idx = content.indexOf(pattern)
  if (idx === -1) return 1
  return content.slice(0, idx).split('\n').length
}

/** Validate vars.json schema structure with line numbers and severity levels. */
function validateVarsSchema(content: string): VarsWarning[] {
  const warnings: VarsWarning[] = []
  try {
    const parsed = JSON.parse(content)
    if (typeof parsed !== 'object' || parsed === null) return warnings

    for (const [key, val] of Object.entries(parsed)) {
      if (key === '$schema') continue
      if (typeof val !== 'object' || val === null) continue
      const v = val as Record<string, unknown>
      const line = findKeyLine(content, key)

      if (v.type && !VALID_TYPES.has(v.type as string)) {
        warnings.push({ key, message: `invalid type "${String(v.type)}"`, line, severity: 'error' })
      }
      if (v['storage-hint'] && !VALID_STORAGE.has(v['storage-hint'] as string)) {
        warnings.push({ key, message: `invalid storage-hint "${String(v['storage-hint'])}"`, line, severity: 'error' })
      }
      if (!v.label) {
        warnings.push({ key, message: 'missing "label"', line, severity: 'warning' })
      }
      if (!v.description) {
        warnings.push({ key, message: 'missing "description"', line, severity: 'warning' })
      }
    }
  } catch {
    // JSON parse errors handled elsewhere
  }
  return warnings
}

export function JsonViewer({ content, tabId, filePath, onContentChange, onSave }: Props) {
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const isEditable = Boolean(onContentChange && tabId)
  const isVarsJson = filePath?.endsWith('vars.json') ?? false

  const { lines, error, warnings } = useMemo(() => {
    let parseError: string | null = null
    let tokenLines: JsonToken[][]
    try {
      const parsed = JSON.parse(content)
      const formatted = JSON.stringify(parsed, null, 2)
      tokenLines = tokenizeJson(formatted)
    } catch (e) {
      tokenLines = tokenizeJson(content)
      parseError = (e as Error).message
    }
    const varsWarnings = isEditable && isVarsJson ? validateVarsSchema(content) : []
    return { lines: tokenLines, error: parseError, warnings: varsWarnings }
  }, [content, isEditable, isVarsJson])

  // For editable mode, tokenize the raw content so line positions match
  const editLines = useMemo(() => {
    if (!isEditable) return lines
    return tokenizeJson(content)
  }, [content, isEditable, lines])

  const displayLines = isEditable ? editLines : lines
  const isValid = isVarsJson && !error && warnings.length === 0

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault()
        if (tabId && onSave) onSave(tabId)
      }
    },
    [tabId, onSave],
  )

  return (
    <div className="flex flex-1 flex-col min-h-0" onKeyDown={isEditable ? handleKeyDown : undefined}>
      {error && (
        <div className="px-4 py-1.5 text-[11px] text-red-600 dark:text-red-400 bg-red-500/10 border-b border-red-500/20 flex items-center gap-1.5 shrink-0">
          <CircleAlert className="size-3 shrink-0" />
          Invalid JSON: {error}
        </div>
      )}
      {warnings.length > 0 && (
        <div className="px-4 py-1.5 text-[11px] border-b border-border shrink-0 space-y-0.5">
          {warnings.map((w, i) => (
            <div key={i} className={`flex items-center gap-1.5 ${w.severity === 'error' ? 'text-red-600 dark:text-red-400' : 'text-amber-600 dark:text-amber-400'}`}>
              {w.severity === 'error' ? <CircleAlert className="size-3 shrink-0" /> : <TriangleAlert className="size-3 shrink-0" />}
              <span className="font-mono text-muted-foreground">L{w.line}</span>
              {w.key}: {w.message}
            </div>
          ))}
        </div>
      )}
      {isValid && (
        <div className="px-4 py-1.5 text-[11px] text-emerald-600 dark:text-emerald-400 bg-emerald-500/5 border-b border-emerald-500/20 flex items-center gap-1.5 shrink-0">
          <CircleCheck className="size-3" />
          Valid vars schema
        </div>
      )}
      <div className="flex-1 min-h-0 overflow-auto">
        <div className="flex min-h-full">
          <div className="shrink-0 w-10 pt-4 pb-4 text-right pr-2 font-mono text-[11px] leading-[1.7] text-muted-foreground select-none border-r border-border sticky left-0 bg-background/80">
            {displayLines.map((_, i) => <div key={i}>{i + 1}</div>)}
          </div>
          <div className="flex-1 relative min-w-0">
            <pre className={`${isEditable ? '' : 'flex-1 '}px-4 pt-4 pb-4 font-mono text-xs leading-[1.7] whitespace-pre-wrap break-words ${isEditable ? 'pointer-events-none' : ''}`} aria-hidden={isEditable}>
              {displayLines.map((tokens, li) => (
                <div key={li}>
                  {tokens.length === 0
                    ? ' '
                    : tokens.map((t, ti) => (
                        <span key={ti} className={TOKEN_CLASSES[t.type]}>{t.text}</span>
                      ))}
                </div>
              ))}
            </pre>
            {isEditable && (
              <textarea
                ref={textareaRef}
                value={content}
                onChange={(e) => onContentChange!(tabId!, e.target.value)}
                className="absolute inset-0 w-full h-full px-4 pt-4 pb-4 font-mono text-xs leading-[1.7] text-transparent caret-foreground bg-transparent resize-none focus:outline-none whitespace-pre-wrap break-words selection:bg-primary/25 selection:text-transparent"
                spellCheck={false}
                autoComplete="off"
              />
            )}
          </div>
        </div>
      </div>
      {isEditable && isVarsJson && <SchemaHints content={content} />}
    </div>
  )
}
