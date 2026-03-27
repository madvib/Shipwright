import { describe, it, expect } from 'vitest'
import { pullAgentToResolved } from '../pull-adapter'
import type { PullAgent } from '@ship/ui'

function makePullAgent(overrides?: Partial<PullAgent>): PullAgent {
  return {
    profile: {
      id: 'test-agent',
      name: 'Test Agent',
      description: 'A test agent',
      providers: ['claude'],
      version: '0.1.0',
    },
    skills: [],
    mcpServers: [],
    rules: [],
    hooks: [],
    source: 'project',
    ...overrides,
  }
}

describe('pullAgentToResolved', () => {
  it('converts a minimal PullAgent to ResolvedAgentProfile', () => {
    const pull = makePullAgent()
    const resolved = pullAgentToResolved(pull)

    expect(resolved.profile.id).toBe('test-agent')
    expect(resolved.profile.name).toBe('Test Agent')
    expect(resolved.profile.providers).toEqual(['claude'])
    expect(resolved.skills).toEqual([])
    expect(resolved.mcpServers).toEqual([])
    expect(resolved.rules).toEqual([])
    expect(resolved.hooks).toEqual([])
    expect(resolved.source).toBe('project')
  })

  it('converts skills from PullSkill to Skill', () => {
    const pull = makePullAgent({
      skills: [
        { id: 'skill-1', name: 'Skill One', description: 'Desc', content: '# Skill', source: 'custom' },
      ],
    })
    const resolved = pullAgentToResolved(pull)

    expect(resolved.skills).toHaveLength(1)
    expect(resolved.skills[0].id).toBe('skill-1')
    expect(resolved.skills[0].name).toBe('Skill One')
    expect(resolved.skills[0].content).toBe('# Skill')
  })

  it('converts MCP servers with required defaults', () => {
    const pull = makePullAgent({
      mcpServers: [{ name: 'my-server', command: 'npx serve' }],
    })
    const resolved = pullAgentToResolved(pull)

    expect(resolved.mcpServers).toHaveLength(1)
    expect(resolved.mcpServers[0].name).toBe('my-server')
    expect(resolved.mcpServers[0].command).toBe('npx serve')
    expect(resolved.mcpServers[0].url).toBeNull()
    expect(resolved.mcpServers[0].timeout_secs).toBeNull()
    expect(resolved.mcpServers[0].codex_enabled_tools).toEqual([])
  })

  it('converts rules from PullRule to Rule', () => {
    const pull = makePullAgent({
      rules: [{ file_name: 'code-style.md', content: '# Style' }],
    })
    const resolved = pullAgentToResolved(pull)

    expect(resolved.rules).toHaveLength(1)
    expect(resolved.rules[0].file_name).toBe('code-style.md')
    expect(resolved.rules[0].content).toBe('# Style')
  })

  it('preserves optional agent-level fields', () => {
    const pull = makePullAgent({
      model: 'claude-sonnet-4',
      env: { API_KEY: 'test' },
      available_models: ['claude-sonnet-4', 'claude-opus-4'],
      agent_limits: { max_turns: 10 },
    })
    const resolved = pullAgentToResolved(pull)

    expect(resolved.model).toBe('claude-sonnet-4')
    expect(resolved.env).toEqual({ API_KEY: 'test' })
    expect(resolved.availableModels).toEqual(['claude-sonnet-4', 'claude-opus-4'])
    expect(resolved.agentLimits).toEqual({ max_turns: 10 })
  })

  it('defaults null for missing optional fields', () => {
    const pull = makePullAgent()
    const resolved = pullAgentToResolved(pull)

    expect(resolved.model).toBeNull()
    expect(resolved.env).toBeNull()
    expect(resolved.availableModels).toBeNull()
    expect(resolved.agentLimits).toBeNull()
  })

  it('handles library source', () => {
    const pull = makePullAgent({ source: 'library' })
    const resolved = pullAgentToResolved(pull)
    expect(resolved.source).toBe('library')
  })
})
