import { describe, it, expect } from 'vitest'
import {
  isDraftDirty,
  deriveUnsavedIds,
  makeTabId,
  parseTabId,
  type SkillDraft,
} from '../skill-drafts'

function draft(content: string, original: string, isNew = false): SkillDraft {
  return { content, originalContent: original, isNew }
}

describe('isDraftDirty — content matching originalContent', () => {
  it('returns false when content equals originalContent for existing file', () => {
    expect(isDraftDirty(draft('hello world', 'hello world', false))).toBe(false)
  })

  it('returns false when both content and originalContent are empty', () => {
    expect(isDraftDirty(draft('', '', false))).toBe(false)
  })

  it('returns false for whitespace-identical content', () => {
    const ws = '  \n\t  '
    expect(isDraftDirty(draft(ws, ws, false))).toBe(false)
  })

  it('treats trailing newline difference as dirty', () => {
    expect(isDraftDirty(draft('hello\n', 'hello', false))).toBe(true)
  })
})

describe('isDraftDirty — isNew files always dirty', () => {
  it('returns true for isNew even when content matches originalContent', () => {
    expect(isDraftDirty(draft('same', 'same', true))).toBe(true)
  })

  it('returns true for isNew with empty content', () => {
    expect(isDraftDirty(draft('', '', true))).toBe(true)
  })

  it('returns true for isNew even with identical multiline content', () => {
    const big = 'line1\nline2\nline3\n'.repeat(100)
    expect(isDraftDirty(draft(big, big, true))).toBe(true)
  })
})

describe('deriveUnsavedIds with isNew drafts', () => {
  it('does not include non-new draft with matching content', () => {
    const drafts: Record<string, SkillDraft> = {
      'skill-a::SKILL.md': draft('content', 'content', false),
      'skill-b::SKILL.md': draft('content', 'content', false),
    }
    expect(deriveUnsavedIds(drafts)).toEqual(new Set())
  })

  it('includes all isNew drafts regardless of content match', () => {
    const drafts: Record<string, SkillDraft> = {
      'skill-a::SKILL.md': draft('abc', 'abc', true),
      'skill-b::vars.json': draft('{}', '{}', true),
      'skill-c::SKILL.md': draft('same', 'same', false),
    }
    const unsaved = deriveUnsavedIds(drafts)
    expect(unsaved.has('skill-a::SKILL.md')).toBe(true)
    expect(unsaved.has('skill-b::vars.json')).toBe(true)
    expect(unsaved.has('skill-c::SKILL.md')).toBe(false)
  })
})

describe('makeTabId and parseTabId roundtrip', () => {
  it('roundtrips skillId and filePath', () => {
    const tabId = makeTabId('my-skill', 'assets/vars.json')
    expect(tabId).toBe('my-skill::assets/vars.json')
    const { skillId, filePath } = parseTabId(tabId)
    expect(skillId).toBe('my-skill')
    expect(filePath).toBe('assets/vars.json')
  })

  it('parseTabId defaults to SKILL.md when no separator present', () => {
    const { skillId, filePath } = parseTabId('my-skill')
    expect(skillId).toBe('my-skill')
    expect(filePath).toBe('SKILL.md')
  })

  it('handles filePath containing :: characters', () => {
    const tabId = makeTabId('skill', 'path::with::colons')
    const { skillId, filePath } = parseTabId(tabId)
    expect(skillId).toBe('skill')
    expect(filePath).toBe('path::with::colons')
  })
})
