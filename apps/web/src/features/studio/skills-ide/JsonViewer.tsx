/** Syntax-colored JSON viewer/editor for the Skills IDE. */

import { useRef, useMemo, useCallback } from 'react'

interface Props {
  content: string
  tabId?: string
  onContentChange?: (id: string, content: string) => void
  onSave?: (id: string) => void
}

interface JsonToken {
  type: 'key' | 'string' | 'number' | 'boolean' | 'null' | 'punctuation'
  text: string
}

const TOKEN_CLASSES: Record<JsonToken['type'], string> = {
  key: 'text-blue-400 dark:text-blue-300',
  string: 'text-green-600 dark:text-green-400',
  number: 'text-orange-500 dark:text-orange-400',
  boolean: 'text-purple-500 dark:text-purple-400',
  null: 'text-purple-500 dark:text-purple-400',
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

/** Validate vars.json schema structure. Returns warnings for missing fields. */
function validateVarsSchema(content: string): string[] {
  const warnings: string[] = []
  try {
    const parsed = JSON.parse(content)
    if (typeof parsed !== 'object' || parsed === null) return warnings

    const VALID_TYPES = new Set(['string', 'number', 'boolean', 'json'])
    const VALID_STORAGE = new Set(['global', 'local', 'secret'])

    for (const [key, val] of Object.entries(parsed)) {
      if (key === '$schema') continue
      if (typeof val !== 'object' || val === null) continue
      const v = val as Record<string, unknown>
      if (v.type && !VALID_TYPES.has(v.type as string)) {
        warnings.push(`${key}: invalid type "${String(v.type)}"`)
      }
      if (v['storage-hint'] && !VALID_STORAGE.has(v['storage-hint'] as string)) {
        warnings.push(`${key}: invalid storage-hint "${String(v['storage-hint'])}"`)
      }
      if (!v.label) warnings.push(`${key}: missing "label"`)
      if (!v.description) warnings.push(`${key}: missing "description"`)
    }
  } catch {
    // JSON parse errors handled elsewhere
  }
  return warnings
}

export function JsonViewer({ content, tabId, onContentChange, onSave }: Props) {
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const isEditable = Boolean(onContentChange && tabId)

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
    const varsWarnings = isEditable ? validateVarsSchema(content) : []
    return { lines: tokenLines, error: parseError, warnings: varsWarnings }
  }, [content, isEditable])

  // For editable mode, tokenize the raw content so line positions match
  const editLines = useMemo(() => {
    if (!isEditable) return lines
    return tokenizeJson(content)
  }, [content, isEditable, lines])

  const displayLines = isEditable ? editLines : lines

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
    <div className="flex-1 min-h-0 overflow-auto" onKeyDown={isEditable ? handleKeyDown : undefined}>
      {error && (
        <div className="px-4 py-1.5 text-[11px] text-amber-500 bg-amber-500/10 border-b border-amber-500/20">
          Invalid JSON: {error}
        </div>
      )}
      {warnings.length > 0 && (
        <div className="px-4 py-1.5 text-[11px] text-amber-500 bg-amber-500/10 border-b border-amber-500/20">
          {warnings.map((w, i) => <div key={i}>{w}</div>)}
        </div>
      )}
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
  )
}
