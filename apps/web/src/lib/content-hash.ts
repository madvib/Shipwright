// Content hashing utilities for skill deduplication.
// Uses Web Crypto API (available in Cloudflare Workers).

/**
 * Normalize skill content for consistent hashing:
 * - Trim leading/trailing whitespace
 * - Normalize line endings to LF
 * - Strip YAML frontmatter (--- delimited block at start)
 */
export function normalizeSkillContent(content: string): string {
  let text = content.replace(/\r\n/g, '\n').replace(/\r/g, '\n').trim()

  // Strip YAML frontmatter: starts with "---\n", ends with "\n---\n" or "\n---" at EOF
  if (text.startsWith('---\n') || text.startsWith('---\r')) {
    const endIdx = text.indexOf('\n---', 4)
    if (endIdx !== -1) {
      // Skip past the closing "---" and any trailing newline
      let afterIdx = endIdx + 4
      if (text[afterIdx] === '\n') afterIdx++
      text = text.slice(afterIdx).trim()
    }
  }

  return text
}

/**
 * Compute SHA-256 content hash of normalized content.
 * Returns "sha256:<hex>" format.
 */
export async function computeContentHash(content: string): Promise<string> {
  const normalized = normalizeSkillContent(content)
  const encoded = new TextEncoder().encode(normalized)
  const hashBuffer = await crypto.subtle.digest('SHA-256', encoded)
  const hashArray = new Uint8Array(hashBuffer)
  const hex = Array.from(hashArray, (b) => b.toString(16).padStart(2, '0')).join('')
  return `sha256:${hex}`
}
