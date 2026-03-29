/** Lightweight markdown-to-HTML renderer using regex transforms. No external dependencies. */

import { useMemo } from 'react'

interface Props {
  content: string
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

/** Convert markdown to HTML using regex-based transforms. */
function markdownToHtml(md: string): string {
  let html = escapeHtml(md)

  // Code blocks: ```lang\n...\n``` -> <pre><code>
  html = html.replace(
    /^```(\w*)\n([\s\S]*?)^```/gm,
    (_, lang, code) => {
      const cls = lang ? ` class="language-${lang}"` : ''
      return `<pre class="md-pre"><code${cls}>${code.trimEnd()}</code></pre>`
    },
  )

  // Tables: detect lines with |---|
  html = html.replace(
    /^(\|.+\|)\n(\|[\s:|-]+\|)\n((?:\|.+\|\n?)+)/gm,
    (_, header: string, _sep: string, body: string) => {
      const heads = header.split('|').filter(Boolean).map((c: string) => c.trim())
      const rows = body.trim().split('\n').map((row: string) =>
        row.split('|').filter(Boolean).map((c: string) => c.trim()),
      )
      const thRow = heads.map((h: string) => `<th>${h}</th>`).join('')
      const bodyRows = rows
        .map((cols: string[]) => `<tr>${cols.map((c: string) => `<td>${c}</td>`).join('')}</tr>`)
        .join('\n')
      return `<table class="md-table"><thead><tr>${thRow}</tr></thead><tbody>${bodyRows}</tbody></table>`
    },
  )

  // Headings: # -> h1, ## -> h2, etc.
  html = html.replace(/^######\s+(.+)$/gm, '<h6 class="md-h6">$1</h6>')
  html = html.replace(/^#####\s+(.+)$/gm, '<h5 class="md-h5">$1</h5>')
  html = html.replace(/^####\s+(.+)$/gm, '<h4 class="md-h4">$1</h4>')
  html = html.replace(/^###\s+(.+)$/gm, '<h3 class="md-h3">$1</h3>')
  html = html.replace(/^##\s+(.+)$/gm, '<h2 class="md-h2">$1</h2>')
  html = html.replace(/^#\s+(.+)$/gm, '<h1 class="md-h1">$1</h1>')

  // Horizontal rules
  html = html.replace(/^---$/gm, '<hr class="md-hr" />')

  // Unordered lists (consecutive - or * lines), with TODO checkbox support
  html = html.replace(
    /^(?:[-*]\s+.+\n?)+/gm,
    (block) => {
      const items = block.trim().split('\n').map((line) => {
        const text = line.replace(/^[-*]\s+/, '')
        // TODO checkboxes: - [ ] unchecked, - [x] checked
        if (text.startsWith('[ ] ')) {
          return `<li class="md-todo"><input type="checkbox" disabled />${text.slice(4)}</li>`
        }
        if (text.startsWith('[x] ') || text.startsWith('[X] ')) {
          return `<li class="md-todo md-todo-done"><input type="checkbox" checked disabled />${text.slice(4)}</li>`
        }
        return `<li>${text}</li>`
      })
      return `<ul class="md-ul">${items.join('\n')}</ul>`
    },
  )

  // Ordered lists
  html = html.replace(
    /^(?:\d+\.\s+.+\n?)+/gm,
    (block) => {
      const items = block.trim().split('\n').map((line) =>
        `<li>${line.replace(/^\d+\.\s+/, '')}</li>`,
      )
      return `<ol class="md-ol">${items.join('\n')}</ol>`
    },
  )

  // Bold: **text** -> <strong>
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
  // Italic: *text* -> <em>
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>')

  // Inline code: `code` -> <code>
  html = html.replace(/`([^`]+)`/g, '<code class="md-code">$1</code>')

  // Links: [text](url)
  html = html.replace(
    /\[([^\]]+)\]\(([^)]+)\)/g,
    '<a href="$2" class="md-link" target="_blank" rel="noopener noreferrer">$1</a>',
  )

  // Wrap remaining plain-text lines in paragraphs
  html = html
    .split('\n\n')
    .map((block) => {
      const trimmed = block.trim()
      if (!trimmed) return ''
      if (/^<(?:h[1-6]|pre|ul|ol|table|hr|blockquote)/.test(trimmed)) return trimmed
      return `<p class="md-p">${trimmed}</p>`
    })
    .join('\n')

  return html
}

export function MarkdownPreview({ content }: Props) {
  const html = useMemo(() => markdownToHtml(content), [content])

  return (
    <div
      className="md-preview px-6 py-4 text-sm text-foreground leading-relaxed overflow-auto"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}
