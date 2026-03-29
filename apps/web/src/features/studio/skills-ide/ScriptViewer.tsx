/** Basic syntax-colored viewer for script files (.sh, .py, .js, .ts). */

import { useMemo } from 'react'

interface Props {
  content: string
  language: 'sh' | 'py' | 'js' | 'ts'
}

interface Token {
  text: string
  className: string
}

const KEYWORDS: Record<string, Set<string>> = {
  sh: new Set([
    'if', 'then', 'else', 'elif', 'fi', 'for', 'while', 'do', 'done',
    'case', 'esac', 'in', 'function', 'return', 'exit', 'local',
    'export', 'source', 'set', 'unset', 'readonly', 'shift', 'break', 'continue',
  ]),
  py: new Set([
    'def', 'class', 'if', 'elif', 'else', 'for', 'while', 'return',
    'import', 'from', 'as', 'with', 'try', 'except', 'finally', 'raise',
    'yield', 'lambda', 'pass', 'break', 'continue', 'and', 'or', 'not',
    'in', 'is', 'None', 'True', 'False', 'async', 'await',
  ]),
  js: new Set([
    'const', 'let', 'var', 'function', 'return', 'if', 'else', 'for',
    'while', 'do', 'switch', 'case', 'break', 'continue', 'new', 'this',
    'class', 'extends', 'import', 'export', 'from', 'default', 'async',
    'await', 'try', 'catch', 'finally', 'throw', 'typeof', 'instanceof',
    'yield', 'of', 'in', 'true', 'false', 'null', 'undefined',
  ]),
  ts: new Set([
    'const', 'let', 'var', 'function', 'return', 'if', 'else', 'for',
    'while', 'do', 'switch', 'case', 'break', 'continue', 'new', 'this',
    'class', 'extends', 'import', 'export', 'from', 'default', 'async',
    'await', 'try', 'catch', 'finally', 'throw', 'typeof', 'instanceof',
    'type', 'interface', 'enum', 'implements', 'abstract', 'private',
    'public', 'protected', 'readonly', 'as', 'keyof', 'never', 'void',
    'yield', 'of', 'in', 'true', 'false', 'null', 'undefined',
  ]),
}

function isCommentStart(line: string, lang: string): boolean {
  const trimmed = line.trimStart()
  if (trimmed.startsWith('#') && lang === 'sh') return true
  if (trimmed.startsWith('#') && lang === 'py') return true
  if (trimmed.startsWith('//') && (lang === 'js' || lang === 'ts')) return true
  return false
}

function tokenizeLine(line: string, lang: string): Token[] {
  if (isCommentStart(line, lang)) {
    return [{ text: line, className: 'text-muted-foreground italic' }]
  }

  const tokens: Token[] = []
  const kwSet = KEYWORDS[lang] ?? KEYWORDS.js
  let i = 0

  while (i < line.length) {
    const ch = line[i]

    // Strings
    if (ch === '"' || ch === "'" || ch === '`') {
      const quote = ch
      let str = quote
      i++
      while (i < line.length && line[i] !== quote) {
        if (line[i] === '\\' && i + 1 < line.length) {
          str += line[i] + line[i + 1]
          i += 2
          continue
        }
        str += line[i]
        i++
      }
      if (i < line.length) { str += line[i]; i++ }
      tokens.push({ text: str, className: 'text-green-600 dark:text-green-400' })
      continue
    }

    // Words (identifiers/keywords)
    if (/[a-zA-Z_$]/.test(ch)) {
      let word = ''
      while (i < line.length && /[a-zA-Z0-9_$]/.test(line[i])) {
        word += line[i]
        i++
      }
      tokens.push({
        text: word,
        className: kwSet.has(word) ? 'text-blue-400 dark:text-blue-300 font-medium' : '',
      })
      continue
    }

    // Numbers
    if (/\d/.test(ch)) {
      let num = ''
      while (i < line.length && /[\d.xXa-fA-F]/.test(line[i])) {
        num += line[i]
        i++
      }
      tokens.push({ text: num, className: 'text-orange-500 dark:text-orange-400' })
      continue
    }

    // Everything else
    tokens.push({ text: ch, className: '' })
    i++
  }

  return tokens
}

export function ScriptViewer({ content, language }: Props) {
  const lines = useMemo(
    () => content.split('\n').map((line) => tokenizeLine(line, language)),
    [content, language],
  )

  return (
    <div className="flex-1 min-h-0 overflow-auto">
      <div className="flex min-h-full">
        <div className="shrink-0 w-10 pt-4 pb-4 text-right pr-2 font-mono text-[11px] leading-[1.7] text-muted-foreground select-none border-r border-border sticky left-0 bg-background/80">
          {lines.map((_, i) => <div key={i}>{i + 1}</div>)}
        </div>
        <pre className="flex-1 px-4 pt-4 pb-4 font-mono text-xs leading-[1.7] whitespace-pre-wrap break-words text-foreground/80">
          {lines.map((tokens, li) => (
            <div key={li}>
              {tokens.length === 0
                ? ' '
                : tokens.map((t, ti) => (
                    <span key={ti} className={t.className}>{t.text}</span>
                  ))}
            </div>
          ))}
        </pre>
      </div>
    </div>
  )
}
