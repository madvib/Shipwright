import { describe, it, expect } from 'vitest'
import { aggregateSkills, mergeProjectSkills } from '../useSkillsLibrary'
import type { LibrarySkill } from '../useSkillsLibrary'
import type { PullSkill } from '@ship/ui'

function skill(id: string, name: string, source = 'custom'): PullSkill {
  return { id, name, description: `${name} description`, content: `# ${name}`, source, tags: [], authors: [], files: [], reference_docs: {} }
}

describe('aggregateSkills', () => {
  it('returns empty array for no agents', () => {
    expect(aggregateSkills([])).toEqual([])
  })

  it('collects skills from a single agent', () => {
    const agents = [
      { id: 'agent-a', skills: [skill('tdd', 'TDD'), skill('review', 'Code Review')], source: 'project' },
    ]
    const result = aggregateSkills(agents)
    expect(result).toHaveLength(2)
    expect(result[0].id).toBe('tdd')
    expect(result[0].usedBy).toEqual(['agent-a'])
    expect(result[0].origin).toBe('project')
    expect(result[1].id).toBe('review')
  })

  it('deduplicates skills by ID across multiple agents', () => {
    const agents = [
      { id: 'agent-a', skills: [skill('tdd', 'TDD')], source: 'project' },
      { id: 'agent-b', skills: [skill('tdd', 'TDD')], source: 'project' },
      { id: 'agent-c', skills: [skill('review', 'Code Review')], source: 'project' },
    ]
    const result = aggregateSkills(agents)
    expect(result).toHaveLength(2)

    const tdd = result.find((s) => s.id === 'tdd')!
    expect(tdd.usedBy).toEqual(['agent-a', 'agent-b'])
  })

  it('does not duplicate agent ID in usedBy when same agent has skill twice', () => {
    const agents = [
      { id: 'agent-a', skills: [skill('tdd', 'TDD'), skill('tdd', 'TDD')], source: 'project' },
    ]
    const result = aggregateSkills(agents)
    expect(result).toHaveLength(1)
    expect(result[0].usedBy).toEqual(['agent-a'])
  })

  it('tracks origin from agent source', () => {
    const agents = [
      { id: 'local-agent', skills: [skill('tdd', 'TDD')], source: 'project' },
      { id: 'lib-agent', skills: [skill('deploy', 'Deploy')], source: 'library' },
    ]
    const result = aggregateSkills(agents)
    const tdd = result.find((s) => s.id === 'tdd')!
    const deploy = result.find((s) => s.id === 'deploy')!

    expect(tdd.origin).toBe('project')
    expect(deploy.origin).toBe('library')
  })

  it('preserves first-seen origin when skill appears in both project and library agents', () => {
    const agents = [
      { id: 'project-agent', skills: [skill('shared', 'Shared Skill')], source: 'project' },
      { id: 'lib-agent', skills: [skill('shared', 'Shared Skill')], source: 'library' },
    ]
    const result = aggregateSkills(agents)
    expect(result).toHaveLength(1)
    // First occurrence wins
    expect(result[0].origin).toBe('project')
    expect(result[0].usedBy).toEqual(['project-agent', 'lib-agent'])
  })

  it('maps skill content and description correctly', () => {
    const agents = [
      { id: 'a', skills: [skill('api-design', 'API Design')], source: 'project' },
    ]
    const result = aggregateSkills(agents)
    expect(result[0].content).toBe('# API Design')
    expect(result[0].description).toBe('API Design description')
    expect(result[0].name).toBe('API Design')
  })

  it('handles agents with no skills', () => {
    const agents = [
      { id: 'empty-agent', skills: [], source: 'project' },
      { id: 'full-agent', skills: [skill('tdd', 'TDD')], source: 'project' },
    ]
    const result = aggregateSkills(agents)
    expect(result).toHaveLength(1)
    expect(result[0].usedBy).toEqual(['full-agent'])
  })

  it('handles null description in PullSkill', () => {
    const ps: PullSkill = { id: 'x', name: 'X', description: null, content: '# X', source: 'custom', tags: [], authors: [], files: [], reference_docs: {} }
    const result = aggregateSkills([{ id: 'a', skills: [ps], source: 'project' }])
    expect(result[0].description).toBeNull()
  })

  it('defaults source to project when agent source is undefined', () => {
    const agents = [
      { id: 'a', skills: [skill('s1', 'Skill 1')], source: undefined },
    ]
    const result = aggregateSkills(agents)
    expect(result[0].origin).toBe('project')
  })
})

function librarySkill(id: string, name: string): LibrarySkill {
  return {
    id, name, description: null, content: `# ${name}`, source: 'custom',
    vars: {}, origin: 'project', usedBy: [], stableId: null,
    tags: [], authors: [], varsSchema: null, files: ['SKILL.md'],
    referenceDocs: {}, evals: null,
  }
}

describe('mergeProjectSkills', () => {
  it('returns agent skills when no project skills', () => {
    const agent = [librarySkill('tdd', 'TDD')]
    const result = mergeProjectSkills(agent, [])
    expect(result).toHaveLength(1)
    expect(result[0].id).toBe('tdd')
  })

  it('adds project skills not in agent list', () => {
    const agent = [librarySkill('tdd', 'TDD')]
    const project = [skill('browse', 'Browse')]
    const result = mergeProjectSkills(agent, project)
    expect(result).toHaveLength(2)
    expect(result.map((s) => s.id).sort()).toEqual(['browse', 'tdd'])
  })

  it('deduplicates by ID, preferring agent skills', () => {
    const agent = [librarySkill('tdd', 'TDD (agent)')]
    const project = [skill('tdd', 'TDD (project)')]
    const result = mergeProjectSkills(agent, project)
    expect(result).toHaveLength(1)
    expect(result[0].name).toBe('TDD (agent)')
  })

  it('returns only project skills when agent list is empty', () => {
    const project = [skill('deploy', 'Deploy'), skill('tdd', 'TDD')]
    const result = mergeProjectSkills([], project)
    expect(result).toHaveLength(2)
    expect(result.every((s) => s.origin === 'project')).toBe(true)
  })
})
