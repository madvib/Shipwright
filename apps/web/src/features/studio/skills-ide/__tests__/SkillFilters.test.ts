import { describe, it, expect } from 'vitest'
import { computeFilterCounts, applyFilters } from '../SkillFilters'
import type { LibrarySkill } from '../useSkillsLibrary'

function makeSkill(overrides: Partial<LibrarySkill> & { id: string }): LibrarySkill {
  return {
    name: overrides.id, description: null, content: '', source: 'custom',
    vars: {}, origin: 'project', usedBy: [], stableId: null,
    tags: [], authors: [], varsSchema: null, files: ['SKILL.md'],
    referenceDocs: {}, evals: null,
    ...overrides,
  }
}

describe('computeFilterCounts', () => {
  it('counts all skills', () => {
    const skills = [makeSkill({ id: 'a' }), makeSkill({ id: 'b' })]
    const counts = computeFilterCounts(skills)
    expect(counts.all).toBe(2)
    expect(counts.smart).toBe(0)
    expect(counts.documented).toBe(0)
  })

  it('counts smart skills by varsSchema', () => {
    const skills = [
      makeSkill({ id: 'a', varsSchema: { type: 'object' } }),
      makeSkill({ id: 'b' }),
    ]
    const counts = computeFilterCounts(skills)
    expect(counts.smart).toBe(1)
  })

  it('counts documented skills by referenceDocs', () => {
    const skills = [
      makeSkill({ id: 'a', referenceDocs: { 'references/docs/index.md': 'content' } }),
      makeSkill({ id: 'b' }),
    ]
    const counts = computeFilterCounts(skills)
    expect(counts.documented).toBe(1)
  })

  it('aggregates tags across skills', () => {
    const skills = [
      makeSkill({ id: 'a', tags: ['testing', 'ci'] }),
      makeSkill({ id: 'b', tags: ['testing'] }),
    ]
    const counts = computeFilterCounts(skills)
    expect(counts.tags.get('testing')).toBe(2)
    expect(counts.tags.get('ci')).toBe(1)
  })
})

describe('applyFilters', () => {
  const skills = [
    makeSkill({ id: 'smart-one', varsSchema: { type: 'object' }, tags: ['ai'] }),
    makeSkill({ id: 'documented-one', referenceDocs: { 'refs/doc.md': 'text' }, tags: ['docs'] }),
    makeSkill({ id: 'plain' }),
  ]

  it('all filter returns everything', () => {
    expect(applyFilters(skills, 'all', new Set())).toHaveLength(3)
  })

  it('smart filter returns only skills with varsSchema', () => {
    const result = applyFilters(skills, 'smart', new Set())
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('smart-one')
  })

  it('documented filter returns only skills with referenceDocs', () => {
    const result = applyFilters(skills, 'documented', new Set())
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('documented-one')
  })

  it('tag filter applies AND logic with other filters', () => {
    const result = applyFilters(skills, 'all', new Set(['ai']))
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('smart-one')
  })

  it('multiple tags require all to match', () => {
    const extSkills = [
      makeSkill({ id: 'both', tags: ['ai', 'docs'] }),
      makeSkill({ id: 'only-ai', tags: ['ai'] }),
    ]
    const result = applyFilters(extSkills, 'all', new Set(['ai', 'docs']))
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('both')
  })
})
