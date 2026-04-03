import { describe, it, expect } from 'vitest'
import { computeFilterCounts, applyFilters } from '../SkillFilters'
import type { LibrarySkill } from '../useSkillsLibrary'

function makeSkill(overrides: Partial<LibrarySkill> & { id: string }): LibrarySkill {
  return {
    name: overrides.id, description: null, content: '', source: 'custom',
    vars: {}, artifacts: [], origin: 'project', usedBy: [], stableId: null,
    tags: [], authors: [], varsSchema: null, files: ['SKILL.md'],
    referenceDocs: {}, evals: null,
    ...overrides,
  }
}

describe('tag filter with multiple tags uses AND logic', () => {
  const skills = [
    makeSkill({ id: 'all-three', tags: ['ai', 'testing', 'ci'] }),
    makeSkill({ id: 'ai-testing', tags: ['ai', 'testing'] }),
    makeSkill({ id: 'ai-only', tags: ['ai'] }),
    makeSkill({ id: 'none', tags: [] }),
  ]

  it('single tag returns all skills with that tag', () => {
    const result = applyFilters(skills, 'all', new Set(['ai']))
    expect(result.map((s) => s.id).sort()).toEqual(['ai-only', 'ai-testing', 'all-three'])
  })

  it('two tags requires both to match', () => {
    const result = applyFilters(skills, 'all', new Set(['ai', 'testing']))
    expect(result.map((s) => s.id).sort()).toEqual(['ai-testing', 'all-three'])
  })

  it('three tags narrows to skills with all three', () => {
    const result = applyFilters(skills, 'all', new Set(['ai', 'testing', 'ci']))
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('all-three')
  })

  it('non-existent tag returns no results', () => {
    const result = applyFilters(skills, 'all', new Set(['nonexistent']))
    expect(result).toHaveLength(0)
  })
})

describe('smart filter identifies skills with varsSchema', () => {
  it('includes skills with non-null varsSchema', () => {
    const skills = [
      makeSkill({ id: 'smart-obj', varsSchema: { type: 'object', properties: {} } }),
      makeSkill({ id: 'smart-str', varsSchema: 'simple-schema' }),
      makeSkill({ id: 'no-vars', varsSchema: null }),
      makeSkill({ id: 'no-vars-undef' }),
    ]
    const result = applyFilters(skills, 'smart', new Set())
    expect(result.map((s) => s.id).sort()).toEqual(['smart-obj', 'smart-str'])
  })

  it('counts smart skills correctly', () => {
    const skills = [
      makeSkill({ id: 'a', varsSchema: { type: 'object' } }),
      makeSkill({ id: 'b', varsSchema: { type: 'string' } }),
      makeSkill({ id: 'c' }),
    ]
    const counts = computeFilterCounts(skills)
    expect(counts.smart).toBe(2)
  })

  it('smart filter combined with tags narrows both ways', () => {
    const skills = [
      makeSkill({ id: 'smart-tagged', varsSchema: { type: 'object' }, tags: ['deploy'] }),
      makeSkill({ id: 'smart-untagged', varsSchema: { type: 'object' }, tags: [] }),
      makeSkill({ id: 'tagged-not-smart', tags: ['deploy'] }),
    ]
    const result = applyFilters(skills, 'smart', new Set(['deploy']))
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('smart-tagged')
  })
})

describe('documented filter identifies skills with referenceDocs', () => {
  it('includes skills with non-empty referenceDocs', () => {
    const skills = [
      makeSkill({ id: 'has-docs', referenceDocs: { 'references/docs/api.md': '# API' } }),
      makeSkill({ id: 'multi-docs', referenceDocs: { 'a.md': 'a', 'b.md': 'b' } }),
      makeSkill({ id: 'empty-docs', referenceDocs: {} }),
      makeSkill({ id: 'no-docs' }),
    ]
    const result = applyFilters(skills, 'documented', new Set())
    expect(result.map((s) => s.id).sort()).toEqual(['has-docs', 'multi-docs'])
  })

  it('counts documented skills correctly', () => {
    const skills = [
      makeSkill({ id: 'a', referenceDocs: { 'x.md': 'content' } }),
      makeSkill({ id: 'b', referenceDocs: {} }),
      makeSkill({ id: 'c', referenceDocs: { 'y.md': 'y', 'z.md': 'z' } }),
    ]
    const counts = computeFilterCounts(skills)
    expect(counts.documented).toBe(2)
  })

  it('documented filter combined with tags', () => {
    const skills = [
      makeSkill({ id: 'doc-tagged', referenceDocs: { 'd.md': 'doc' }, tags: ['api'] }),
      makeSkill({ id: 'doc-untagged', referenceDocs: { 'd.md': 'doc' }, tags: [] }),
      makeSkill({ id: 'tagged-no-doc', tags: ['api'] }),
    ]
    const result = applyFilters(skills, 'documented', new Set(['api']))
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('doc-tagged')
  })
})

describe('computeFilterCounts tag aggregation', () => {
  it('aggregates tags across all skills', () => {
    const skills = [
      makeSkill({ id: 'a', tags: ['x', 'y'] }),
      makeSkill({ id: 'b', tags: ['y', 'z'] }),
      makeSkill({ id: 'c', tags: ['x', 'y', 'z'] }),
    ]
    const counts = computeFilterCounts(skills)
    expect(counts.tags.get('x')).toBe(2)
    expect(counts.tags.get('y')).toBe(3)
    expect(counts.tags.get('z')).toBe(2)
  })

  it('returns empty tags map when no skills have tags', () => {
    const skills = [makeSkill({ id: 'a' }), makeSkill({ id: 'b' })]
    const counts = computeFilterCounts(skills)
    expect(counts.tags.size).toBe(0)
  })
})
