import { describe, it, expect } from 'vitest'
import {
  isDraftDirty,
  deriveUnsavedIds,
  deriveLocalFiles,
  type SkillDraft,
} from '../skill-drafts'

function draft(content: string, original: string, isNew = false): SkillDraft {
  return { content, originalContent: original, isNew }
}

describe('isDraftDirty', () => {
  it('returns false when content matches original', () => {
    expect(isDraftDirty(draft('hello', 'hello'))).toBe(false)
  })

  it('returns true when content differs from original', () => {
    expect(isDraftDirty(draft('hello world', 'hello'))).toBe(true)
  })

  it('returns true for new files even when content matches original', () => {
    expect(isDraftDirty(draft('# New', '# New', true))).toBe(true)
  })
})

describe('deriveUnsavedIds', () => {
  it('returns empty set for empty drafts', () => {
    expect(deriveUnsavedIds({})).toEqual(new Set())
  })

  it('includes dirty drafts', () => {
    const drafts: Record<string, SkillDraft> = {
      'skill-a::SKILL.md': draft('changed', 'original'),
      'skill-b::SKILL.md': draft('same', 'same'),
    }
    expect(deriveUnsavedIds(drafts)).toEqual(new Set(['skill-a::SKILL.md']))
  })

  it('includes new files as unsaved', () => {
    const drafts: Record<string, SkillDraft> = {
      'skill-a::assets/vars.json': draft('{}', '{}', true),
    }
    expect(deriveUnsavedIds(drafts)).toEqual(new Set(['skill-a::assets/vars.json']))
  })
})

describe('deriveLocalFiles', () => {
  it('returns empty record for no new files', () => {
    const drafts: Record<string, SkillDraft> = {
      'skill-a::SKILL.md': draft('x', 'y', false),
    }
    expect(deriveLocalFiles(drafts)).toEqual({})
  })

  it('groups new files by skillId', () => {
    const drafts: Record<string, SkillDraft> = {
      'skill-a::assets/vars.json': draft('{}', '{}', true),
      'skill-a::references/docs/index.md': draft('# Doc', '', true),
      'skill-b::evals/evals.json': draft('[]', '', true),
    }
    const result = deriveLocalFiles(drafts)
    expect(result).toEqual({
      'skill-a': ['assets/vars.json', 'references/docs/index.md'],
      'skill-b': ['evals/evals.json'],
    })
  })

  it('excludes non-new drafts from local files', () => {
    const drafts: Record<string, SkillDraft> = {
      'skill-a::SKILL.md': draft('changed', 'original', false),
      'skill-a::assets/vars.json': draft('{}', '{}', true),
    }
    const result = deriveLocalFiles(drafts)
    expect(result).toEqual({
      'skill-a': ['assets/vars.json'],
    })
  })
})
