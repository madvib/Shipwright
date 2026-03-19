import { describe, it, expect } from 'vitest'
import { RELEVANT } from '#/lib/fetch-repo-files'

describe('RELEVANT file filter', () => {
  it('matches CLAUDE.md', () => {
    expect(RELEVANT('CLAUDE.md')).toBe(true)
  })

  it('matches AGENTS.md', () => {
    expect(RELEVANT('AGENTS.md')).toBe(true)
  })

  it('matches .mcp.json', () => {
    expect(RELEVANT('.mcp.json')).toBe(true)
  })

  it('matches .gemini/GEMINI.md', () => {
    expect(RELEVANT('.gemini/GEMINI.md')).toBe(true)
  })

  it('matches .cursor/rules/*.mdc files', () => {
    expect(RELEVANT('.cursor/rules/react.mdc')).toBe(true)
    expect(RELEVANT('.cursor/rules/testing.mdc')).toBe(true)
  })

  it('rejects .cursor/rules/*.md (non-mdc)', () => {
    expect(RELEVANT('.cursor/rules/react.md')).toBe(false)
  })

  it('matches .ship/agents/ paths', () => {
    expect(RELEVANT('.ship/agents/rules/base.md')).toBe(true)
    expect(RELEVANT('.ship/agents/skills/foo/SKILL.md')).toBe(true)
  })

  it('rejects unrelated files', () => {
    expect(RELEVANT('README.md')).toBe(false)
    expect(RELEVANT('src/index.ts')).toBe(false)
    expect(RELEVANT('package.json')).toBe(false)
  })
})
