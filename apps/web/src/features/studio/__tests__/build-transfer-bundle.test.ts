import { describe, it, expect } from 'vitest'
import { buildTransferBundle } from '../build-transfer-bundle'
import { makeAgent } from '#/features/agents/make-agent'
import type { TransferBundle } from '@ship/ui'

describe('buildTransferBundle', () => {
  it('includes all AgentBundle fields from a minimal agent', () => {
    const agent = makeAgent({ profile: { id: 'test', name: 'Test' } })
    const bundle = buildTransferBundle(agent)
    const keys = Object.keys(bundle.agent)

    // Every field on AgentBundle must be present
    const requiredKeys = [
      'id', 'name', 'description', 'version', 'providers',
      'model', 'env', 'available_models', 'agent_limits',
      'hooks', 'skill_refs', 'rule_refs', 'mcp_servers',
      'permissions', 'provider_settings', 'plugins', 'rules_inline',
    ]
    for (const key of requiredKeys) {
      expect(keys).toContain(key)
    }
  })

  it('maps agent-level fields correctly', () => {
    const agent = makeAgent({
      profile: { id: 'a', name: 'Agent A' },
      model: 'claude-sonnet-4',
      env: { API_KEY: 'secret' },
      availableModels: ['claude-sonnet-4', 'claude-opus-4'],
      agentLimits: { max_turns: 10 },
      hooks: [{ id: 'hook-1', trigger: 'PreToolUse' as const, command: 'echo hi' }],
    })
    const bundle = buildTransferBundle(agent)

    expect(bundle.agent.model).toBe('claude-sonnet-4')
    expect(bundle.agent.env).toEqual({ API_KEY: 'secret' })
    expect(bundle.agent.available_models).toEqual(['claude-sonnet-4', 'claude-opus-4'])
    expect(bundle.agent.agent_limits).toEqual({ max_turns: 10 })
    expect(bundle.agent.hooks).toEqual([{ id: 'hook-1', trigger: 'PreToolUse', command: 'echo hi' }])
  })

  it('maps profile fields: version and providers', () => {
    const agent = makeAgent({
      profile: { id: 'b', name: 'B', version: '2.0.0', providers: ['openai', 'claude'] },
    })
    const bundle = buildTransferBundle(agent)

    expect(bundle.agent.version).toBe('2.0.0')
    expect(bundle.agent.providers).toEqual(['openai', 'claude'])
  })

  it('maps skills to skill_refs and skills bundle', () => {
    const agent = makeAgent({
      skills: [
        { id: 'sk-1', name: 'Skill One', description: '', content: '# Skill 1', source: 'custom' as const, vars: {}, artifacts: [] },
        { id: 'sk-2', name: 'Skill Two', description: '', content: '# Skill 2', source: 'custom' as const, vars: {}, artifacts: [] },
      ],
    })
    const bundle = buildTransferBundle(agent)

    expect(bundle.agent.skill_refs).toEqual(['sk-1', 'sk-2'])
    expect(bundle.skills?.['sk-1']).toEqual({ files: { 'SKILL.md': '# Skill 1' } })
    expect(bundle.skills?.['sk-2']).toEqual({ files: { 'SKILL.md': '# Skill 2' } })
  })

  it('maps rules to rule_refs and rules content', () => {
    const agent = makeAgent({
      rules: [
        { file_name: 'code-style.md', content: '# Code Style' },
      ],
    })
    const bundle = buildTransferBundle(agent)

    expect(bundle.agent.rule_refs).toEqual(['code-style.md'])
    expect(bundle.rules?.['code-style.md']).toBe('# Code Style')
  })

  it('maps MCP servers to mcp_servers list', () => {
    const agent = makeAgent({
      mcpServers: [
        { name: 'my-server', command: 'npx serve', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
      ],
    })
    const bundle = buildTransferBundle(agent)

    expect(bundle.agent.mcp_servers).toEqual(['my-server'])
  })

  it('nulls optional fields when agent has none', () => {
    const agent = makeAgent()
    const bundle = buildTransferBundle(agent)

    expect(bundle.agent.model).toBeNull()
    expect(bundle.agent.env).toBeNull()
    expect(bundle.agent.available_models).toBeNull()
    expect(bundle.agent.agent_limits).toBeNull()
    expect(bundle.agent.plugins).toBeNull()
    expect(bundle.agent.rules_inline).toBeNull()
  })

  it('returns valid TransferBundle structure', () => {
    const agent = makeAgent()
    const bundle: TransferBundle = buildTransferBundle(agent)

    expect(bundle).toHaveProperty('agent')
    expect(bundle).toHaveProperty('skills')
    expect(bundle).toHaveProperty('rules')
    expect(bundle).toHaveProperty('dependencies')
  })
})
