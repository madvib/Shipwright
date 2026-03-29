// Pure helper functions for annotation overlay: CSS selector generation and
// element-in-rect queries used when creating annotations on the canvas iframe.

export function getCSSSelector(el: Element): string {
  if (el.id) return `#${el.id}`
  const tag = el.tagName.toLowerCase()
  if (el.className && typeof el.className === 'string') {
    const classes = el.className.trim().split(/\s+/).slice(0, 3).join('.')
    if (classes) return `${tag}.${classes}`
  }
  return tag
}

export function getElementsInRect(
  doc: Document,
  rect: [number, number, number, number],
): string[] {
  const [rx, ry, rw, rh] = rect
  const selectors: string[] = []
  const all = doc.querySelectorAll('*')
  for (const el of all) {
    const r = el.getBoundingClientRect()
    if (r.left >= rx && r.top >= ry && r.right <= rx + rw && r.bottom <= ry + rh) {
      selectors.push(getCSSSelector(el))
    }
    if (selectors.length >= 10) break
  }
  return selectors
}
