import { describe, it, expect } from 'vitest'
import { aggregateSkills, mergeProjectSkills } from '../useSkillsLibrary'
import type { LibrarySkill } from '../useSkillsLibrary'
import type { PullSkill } from '@ship/ui'

function skill(id: string, name: string, overrides?: Partial<PullSkill>): PullSkill {
  return {
    id, name, description: `${name} desc`, content: `# ${name}`,
    source: 'custom', tags: [], authors: [], artifacts: [], files: [], reference_docs: {},
    ...overrides,
  }
}

function librarySkill(id: string, name: string, overrides?: Partial<LibrarySkill>): LibrarySkill {
  return {
    id, name, description: null, content: `# ${name}`, source: 'custom',
    vars: {}, artifacts: [], origin: 'project', usedBy: [], stableId: null,
    tags: [], authors: [], varsSchema: null, files: ['SKILL.md'],
    referenceDocs: {}, evals: null,
    ...overrides,
  }
}

describe('mergeProjectSkills deduplication', () => {
  it('prefers agent-derived skill data over project skill with same ID', () => {
    const agentSkill = librarySkill('shared', 'Agent Version', {
      usedBy: ['agent-a'],
      description: 'agent description',
    })
    const projectSkill = skill('shared', 'Project Version', {
      description: 'project description',
    })
    const result = mergeProjectSkills([agentSkill], [projectSkill])

    expect(result).toHaveLength(1)
    expect(result[0].name).toBe('Agent Version')
    expect(result[0].description).toBe('agent description')
    expect(result[0].usedBy).toEqual(['agent-a'])
  })

  it('keeps agent skill metadata intact when project duplicate exists', () => {
    const agentSkill = librarySkill('deploy', 'Deploy', {
      usedBy: ['agent-x', 'agent-y'],
      varsSchema: { type: 'object' },
      tags: ['ci', 'deploy'],
    })
    const projectSkill = skill('deploy', 'Deploy Project')
    const result = mergeProjectSkills([agentSkill], [projectSkill])

    expect(result).toHaveLength(1)
    expect(result[0].usedBy).toEqual(['agent-x', 'agent-y'])
    expect(result[0].varsSchema).toEqual({ type: 'object' })
    expect(result[0].tags).toEqual(['ci', 'deploy'])
  })
})

describe('project skills with no agent usage', () => {
  it('assigns empty usedBy for project-only skills', () => {
    const projectSkills = [
      skill('orphan-a', 'Orphan A'),
      skill('orphan-b', 'Orphan B'),
    ]
    const result = mergeProjectSkills([], projectSkills)

    expect(result).toHaveLength(2)
    expect(result[0].usedBy).toEqual([])
    expect(result[1].usedBy).toEqual([])
  })

  it('sets origin to project for project-only skills', () => {
    const result = mergeProjectSkills([], [skill('solo', 'Solo Skill')])
    expect(result[0].origin).toBe('project')
  })

  it('mixed: agent skills keep usedBy, project-only skills get empty usedBy', () => {
    const agentSkills = [
      librarySkill('used', 'Used', { usedBy: ['agent-1'] }),
    ]
    const projectSkills = [
      skill('unused', 'Unused'),
    ]
    const result = mergeProjectSkills(agentSkills, projectSkills)

    const used = result.find((s) => s.id === 'used')!
    const unused = result.find((s) => s.id === 'unused')!
    expect(used.usedBy).toEqual(['agent-1'])
    expect(unused.usedBy).toEqual([])
  })
})

describe('agentUsageMap via aggregateSkills', () => {
  it('maps skill IDs to all agent IDs that reference them', () => {
    const agents = [
      { id: 'agent-a', skills: [skill('tdd', 'TDD'), skill('lint', 'Lint')], source: 'project' },
      { id: 'agent-b', skills: [skill('tdd', 'TDD')], source: 'project' },
      { id: 'agent-c', skills: [skill('deploy', 'Deploy')], source: 'library' },
    ]
    const result = aggregateSkills(agents)

    const tdd = result.find((s) => s.id === 'tdd')!
    const lint = result.find((s) => s.id === 'lint')!
    const deploy = result.find((s) => s.id === 'deploy')!

    expect(tdd.usedBy).toEqual(['agent-a', 'agent-b'])
    expect(lint.usedBy).toEqual(['agent-a'])
    expect(deploy.usedBy).toEqual(['agent-c'])
  })

  it('does not duplicate agent ID when same agent lists a skill multiple times', () => {
    const agents = [
      { id: 'agent-a', skills: [skill('dup', 'Dup'), skill('dup', 'Dup'), skill('dup', 'Dup')], source: 'project' },
    ]
    const result = aggregateSkills(agents)
    expect(result).toHaveLength(1)
    expect(result[0].usedBy).toEqual(['agent-a'])
  })

  it('preserves skill metadata from first occurrence', () => {
    const agents = [
      {
        id: 'agent-a',
        skills: [skill('s1', 'First', { description: 'first desc', tags: ['tag-a'] })],
        source: 'project',
      },
      {
        id: 'agent-b',
        skills: [skill('s1', 'Second', { description: 'second desc', tags: ['tag-b'] })],
        source: 'library',
      },
    ]
    const result = aggregateSkills(agents)
    expect(result).toHaveLength(1)
    expect(result[0].name).toBe('First')
    expect(result[0].description).toBe('first desc')
    expect(result[0].tags).toEqual(['tag-a'])
    expect(result[0].usedBy).toEqual(['agent-a', 'agent-b'])
  })
})
