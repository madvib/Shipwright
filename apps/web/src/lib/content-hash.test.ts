import { describe, it, expect } from 'vitest'
import { normalizeSkillContent, computeContentHash } from '#/lib/content-hash'

describe('normalizeSkillContent', () => {
  it('trims whitespace', () => {
    expect(normalizeSkillContent('  hello  ')).toBe('hello')
  })

  it('normalizes CRLF to LF', () => {
    expect(normalizeSkillContent('line1\r\nline2\r\n')).toBe('line1\nline2')
  })

  it('normalizes bare CR to LF', () => {
    expect(normalizeSkillContent('a\rb\r')).toBe('a\nb')
  })

  it('strips YAML frontmatter', () => {
    const input = `---
title: My Skill
version: 1.0
---
# Actual content
Do the thing.`
    expect(normalizeSkillContent(input)).toBe('# Actual content\nDo the thing.')
  })

  it('preserves content when no frontmatter', () => {
    const input = '# Just markdown\nNo frontmatter here.'
    expect(normalizeSkillContent(input)).toBe(input)
  })

  it('does not strip --- that is not frontmatter', () => {
    const input = 'Some text\n---\nMore text'
    expect(normalizeSkillContent(input)).toBe(input)
  })

  it('handles empty string', () => {
    expect(normalizeSkillContent('')).toBe('')
  })

  it('handles frontmatter at EOF without trailing newline', () => {
    const input = '---\nkey: val\n---\nbody'
    expect(normalizeSkillContent(input)).toBe('body')
  })

  it('handles frontmatter with empty body', () => {
    const input = '---\nkey: val\n---\n'
    expect(normalizeSkillContent(input)).toBe('')
  })
})

describe('computeContentHash', () => {
  it('returns sha256: prefixed hex string', async () => {
    const hash = await computeContentHash('hello world')
    expect(hash).toMatch(/^sha256:[a-f0-9]{64}$/)
  })

  it('produces consistent hashes for same content', async () => {
    const a = await computeContentHash('test content')
    const b = await computeContentHash('test content')
    expect(a).toBe(b)
  })

  it('normalizes before hashing (CRLF vs LF)', async () => {
    const a = await computeContentHash('line1\nline2')
    const b = await computeContentHash('line1\r\nline2')
    expect(a).toBe(b)
  })

  it('normalizes whitespace before hashing', async () => {
    const a = await computeContentHash('content')
    const b = await computeContentHash('  content  ')
    expect(a).toBe(b)
  })

  it('strips frontmatter before hashing', async () => {
    const withFm = '---\ntitle: test\n---\nbody text'
    const withoutFm = 'body text'
    const a = await computeContentHash(withFm)
    const b = await computeContentHash(withoutFm)
    expect(a).toBe(b)
  })

  it('produces different hashes for different content', async () => {
    const a = await computeContentHash('alpha')
    const b = await computeContentHash('beta')
    expect(a).not.toBe(b)
  })
})
