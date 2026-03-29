// Pure helper functions for the session canvas iframe: theme injection,
// content wrapping, and theme resolution.

export function getResolvedTheme(): string {
  const root = document.documentElement
  if (root.classList.contains('dark')) return 'dark'
  if (root.classList.contains('light')) return 'light'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

/** Inject a theme listener script so the iframe responds to postMessage theme changes. */
export function injectThemeListener(html: string): string {
  const script = `<script>
window.addEventListener('message', function(e) {
  if (e.data && e.data.type === 'theme') {
    var root = document.documentElement;
    root.classList.remove('light', 'dark');
    root.classList.add(e.data.theme);
    root.setAttribute('data-theme', e.data.theme);
    root.style.colorScheme = e.data.theme;
  }
});
</script>`
  if (html.includes('</head>')) return html.replace('</head>', script + '</head>')
  if (html.includes('</html>')) return html.replace('</html>', script + '</html>')
  return html + script
}

/** Inject the current theme directly into srcdoc so the initial render is correct. */
export function injectThemeAttribute(html: string): string {
  const theme = typeof document !== 'undefined' ? getResolvedTheme() : 'light'
  if (html.includes('<html')) {
    return html.replace('<html', `<html class="${theme}" data-theme="${theme}" style="color-scheme:${theme}"`)
  }
  return html
}

/** Wrap non-HTML content in a styled HTML shell for iframe rendering. */
export function wrapContent(content: string, fileType?: string | null): string {
  if (!content) return ''
  if (fileType === 'image') {
    return `<!DOCTYPE html><html><head><style>
      * { margin: 0; padding: 0; box-sizing: border-box; }
      body { background: #1a1a1a; display: flex; align-items: center; justify-content: center; min-height: 100vh; overflow: auto; }
      img { max-width: 100%; cursor: zoom-in; transition: transform 0.2s; }
      img.zoomed { cursor: zoom-out; transform: scale(2); transform-origin: center; }
    </style></head><body>
      <img src="${content}" onclick="this.classList.toggle('zoomed')" />
    </body></html>`
  }
  if (fileType === 'markdown') {
    const html = content
      .replace(/^### (.+)$/gm, '<h3>$1</h3>')
      .replace(/^## (.+)$/gm, '<h2>$1</h2>')
      .replace(/^# (.+)$/gm, '<h1>$1</h1>')
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replace(/^- (.+)$/gm, '<li>$1</li>')
      .replace(/(<li>.*<\/li>)/s, '<ul>$1</ul>')
      .replace(/\n\n/g, '</p><p>')
      .replace(/^(?!<[hulo])/gm, '<p>')
    return `<!DOCTYPE html><html><head><style>
      * { margin: 0; padding: 0; box-sizing: border-box; }
      body { font-family: -apple-system, sans-serif; padding: 2rem; max-width: 48rem; margin: 0 auto; line-height: 1.6; color: #e8e0d6; background: #18140f; }
      h1, h2, h3 { margin: 1.5rem 0 0.75rem; font-weight: 700; }
      h1 { font-size: 1.75rem; } h2 { font-size: 1.35rem; } h3 { font-size: 1.1rem; }
      p { margin: 0.5rem 0; } code { background: #2a2520; padding: 0.15rem 0.4rem; border-radius: 0.25rem; font-size: 0.9em; }
      ul { padding-left: 1.5rem; margin: 0.5rem 0; } li { margin: 0.25rem 0; }
      strong { font-weight: 600; }
    </style></head><body>${html}</body></html>`
  }
  return content
}
