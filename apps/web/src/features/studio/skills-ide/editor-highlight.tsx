/** Syntax highlighting utilities for the Skills IDE editor. */

interface HighlightedLine {
  text: string
  className: string
  fragments?: { text: string; className: string }[]
}

/** Split content into lines and apply simple syntax classes. */
export function highlightLines(content: string): HighlightedLine[] {
  const lines = content.split('\n')
  let inFrontmatter = false
  let fenceCount = 0

  return lines.map((line) => {
    if (line.trim() === '---') {
      fenceCount++
      inFrontmatter = fenceCount === 1
      return { text: line, className: 'text-muted-foreground' }
    }

    if (inFrontmatter && fenceCount === 1) {
      const colonIdx = line.indexOf(':')
      if (colonIdx > 0) {
        const key = line.slice(0, colonIdx)
        const val = line.slice(colonIdx + 1)
        return {
          text: line,
          className: '',
          fragments: [
            { text: key, className: 'text-sky-500 dark:text-sky-300' },
            { text: ':', className: 'text-muted-foreground' },
            { text: val, className: 'text-emerald-600 dark:text-emerald-300' },
          ],
        }
      }
      return { text: line, className: 'text-emerald-600 dark:text-emerald-300' }
    }

    if (line.startsWith('# ')) return { text: line, className: 'text-foreground font-bold text-sm' }
    if (line.startsWith('## ')) return { text: line, className: 'text-foreground font-semibold' }
    if (line.startsWith('### ')) return { text: line, className: 'text-foreground/80 font-semibold' }
    if (line.startsWith('- ') || line.startsWith('* ')) return { text: line, className: 'text-foreground/70' }
    if (/^\d+\.\s/.test(line)) return { text: line, className: 'text-foreground/70' }
    if (line.trim() === '') return { text: ' ', className: '' }
    return { text: line, className: 'text-foreground/80' }
  })
}

/** Highlight inline `code` -- no padding/size changes to avoid cursor drift. */
export function renderInlineCode(text: string): React.ReactNode {
  const parts = text.split(/(`[^`]+`)/)
  if (parts.length === 1) return text
  return parts.map((part, i) => {
    if (part.startsWith('`') && part.endsWith('`')) {
      return (
        <span key={i} className="text-primary">{part}</span>
      )
    }
    return part
  })
}
