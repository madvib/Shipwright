/** Syntax-colored JSON viewer for the Skills IDE. */

import { useMemo } from 'react'

interface Props {
  content: string
}

interface JsonToken {
  type: 'key' | 'string' | 'number' | 'boolean' | 'null' | 'punctuation'
  text: string
}

const TOKEN_CLASSES: Record<JsonToken['type'], string> = {
  key: 'text-sky-400',
  string: 'text-emerald-400',
  number: 'text-amber-400',
  boolean: 'text-violet-400',
  null: 'text-violet-400',
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

      // Whitespace
      if (ch === ' ' || ch === '\t') {
        let ws = ''
        while (i < line.length && (line[i] === ' ' || line[i] === '\t')) {
          ws += line[i]
          i++
        }
        tokens.push({ type: 'punctuation', text: ws })
        continue
      }

      // Punctuation: { } [ ] , :
      if ('{[}],: '.includes(ch) && ch !== ' ') {
        tokens.push({ type: 'punctuation', text: ch })
        i++
        continue
      }

      // Strings (keys or values)
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

        // Determine if this is a key (followed by optional whitespace then colon)
        let lookAhead = i
        while (lookAhead < line.length && line[lookAhead] === ' ') lookAhead++
        const isKey = lookAhead < line.length && line[lookAhead] === ':'

        tokens.push({ type: isKey ? 'key' : 'string', text: str })
        continue
      }

      // Numbers
      if (ch === '-' || (ch >= '0' && ch <= '9')) {
        let num = ''
        while (i < line.length && /[\d.eE+-]/.test(line[i])) {
          num += line[i]
          i++
        }
        tokens.push({ type: 'number', text: num })
        continue
      }

      // Booleans and null
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

      // Fallback
      tokens.push({ type: 'punctuation', text: ch })
      i++
    }

    return tokens
  })
}

export function JsonViewer({ content }: Props) {
  const { lines, error } = useMemo(() => {
    try {
      const parsed = JSON.parse(content)
      const formatted = JSON.stringify(parsed, null, 2)
      return { lines: tokenizeJson(formatted), error: null }
    } catch (e) {
      // If invalid JSON, tokenize the raw content
      return { lines: tokenizeJson(content), error: (e as Error).message }
    }
  }, [content])

  return (
    <div className="flex-1 min-h-0 overflow-auto">
      {error && (
        <div className="px-4 py-1.5 text-[11px] text-amber-500 bg-amber-500/10 border-b border-amber-500/20">
          Invalid JSON: {error}
        </div>
      )}
      <div className="flex min-h-full">
        <div className="shrink-0 w-10 pt-4 pb-4 text-right pr-2 font-mono text-[11px] leading-[1.7] text-muted-foreground select-none border-r border-border sticky left-0 bg-background/80">
          {lines.map((_, i) => <div key={i}>{i + 1}</div>)}
        </div>
        <pre className="flex-1 px-4 pt-4 pb-4 font-mono text-xs leading-[1.7] whitespace-pre-wrap break-words">
          {lines.map((tokens, li) => (
            <div key={li}>
              {tokens.length === 0
                ? ' '
                : tokens.map((t, ti) => (
                    <span key={ti} className={TOKEN_CLASSES[t.type]}>{t.text}</span>
                  ))}
            </div>
          ))}
        </pre>
      </div>
    </div>
  )
}
