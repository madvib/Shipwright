import { describe, it, expect } from 'vitest'
import { agentToLibrary } from '../agent-to-library'
import type { ResolvedAgentProfile } from '../types'
import { DEFAULT_LIBRARY } from '@ship/ui'
import type { ProjectLibrary } from '@ship/ui'

// ── Fixtures ─────────────────────────────────────────────────────────────────

function makeTestAgent(overrides?: Partial<ResolvedAgentProfile>): ResolvedAgentProfile {
  return {
    profile: {
      id: 'test-agent',
      name: 'Test Agent',
      description: 'A test agent',
      providers: ['claude', 'gemini'],
      version: '0.1.0',
    },
    skills: [
      { id: 'skill-a', name: 'skill-a', content: 'Skill A content', source: 'custom', vars: {}, artifacts: [] },
      { id: 'skill-b', name: 'skill-b', content: 'Skill B content', source: 'community', vars: {}, artifacts: [] },
    ],
    mcpServers: [
      { name: 'github', command: 'npx', args: ['-y', '@mcp/github'], server_type: 'stdio', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
    ],
    permissions: {
      preset: 'ship-guarded',
      tools_allow: ['Read', 'Glob'],
      tools_deny: ['Bash(rm -rf *)'],
    },
    hooks: [
      { id: 'hook-1', trigger: 'PreToolUse', command: './check.sh' },
    ],
    rules: [
      { file_name: 'no-compat.md', content: 'No backward compat' },
    ],
    ...overrides,
  }
}

function makeBaseLibrary(overrides?: Partial<ProjectLibrary>): ProjectLibrary {
  return {
    ...DEFAULT_LIBRARY,
    skills: [
      { id: 'base-skill', name: 'base-skill', content: 'Base skill', source: 'builtin', vars: {}, artifacts: [] },
    ],
    mcp_servers: [
      { name: 'filesystem', command: 'npx', args: ['-y', '@mcp/fs'], server_type: 'stdio', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
    ],
    rules: [
      { file_name: 'base-rule.md', content: 'Base rule' },
    ],
    ...overrides,
  }
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe('agentToLibrary', () => {
  it('returns a new object without mutating the base library', () => {
    const agent = makeTestAgent()
    const base = makeBaseLibrary()
    const baseCopy = JSON.parse(JSON.stringify(base))

    const result = agentToLibrary(agent, base)

    expect(result).not.toBe(base)
    expect(base).toEqual(baseCopy)
  })

  it('sets active_agent to the agent mode id', () => {
    const agent = makeTestAgent()
    const result = agentToLibrary(agent, makeBaseLibrary())

    expect(result.active_agent).toBe('agent-test-agent')
  })

  it('creates a mode referencing the agent skills, servers, and rules', () => {
    const agent = makeTestAgent()
    const result = agentToLibrary(agent, makeBaseLibrary())

    const mode = result.modes?.find((m) => m.id === 'agent-test-agent')
    expect(mode).toBeDefined()
    expect(mode!.name).toBe('Test Agent')
    expect(mode!.target_agents).toEqual(['claude', 'gemini', 'codex', 'cursor'])
    expect(mode!.skills).toEqual(['skill-a', 'skill-b'])
    expect(mode!.mcp_servers).toEqual(['github'])
    expect(mode!.rules).toEqual(['no-compat.md'])
  })

  it('merges agent skills with base skills (deduplicated)', () => {
    const agent = makeTestAgent()
    const result = agentToLibrary(agent, makeBaseLibrary())

    const skillIds = result.skills?.map((s) => s.id) ?? []
    expect(skillIds).toContain('base-skill')
    expect(skillIds).toContain('skill-a')
    expect(skillIds).toContain('skill-b')
    expect(skillIds).toHaveLength(3)
  })

  it('deduplicates skills by id when agent has same skill as base', () => {
    const agent = makeTestAgent({
      skills: [{ id: 'base-skill', name: 'base-skill', content: 'Agent version', source: 'custom', vars: {}, artifacts: [] }],
    })
    const result = agentToLibrary(agent, makeBaseLibrary())

    const skillIds = result.skills?.map((s) => s.id) ?? []
    expect(skillIds).toHaveLength(1)
    // Base version is kept (dedup keeps base first)
    expect(result.skills?.[0].content).toBe('Base skill')
  })

  it('merges MCP servers with base servers (deduplicated by name)', () => {
    const agent = makeTestAgent()
    const result = agentToLibrary(agent, makeBaseLibrary())

    const names = result.mcp_servers?.map((s) => s.name) ?? []
    expect(names).toContain('filesystem')
    expect(names).toContain('github')
    expect(names).toHaveLength(2)
  })

  it('merges rules with base rules (deduplicated by file_name)', () => {
    const agent = makeTestAgent()
    const result = agentToLibrary(agent, makeBaseLibrary())

    const ruleNames = result.rules?.map((r) => r.file_name) ?? []
    expect(ruleNames).toContain('base-rule.md')
    expect(ruleNames).toContain('no-compat.md')
    expect(ruleNames).toHaveLength(2)
  })

  it('converts and appends agent hooks', () => {
    const agent = makeTestAgent()
    const result = agentToLibrary(agent, makeBaseLibrary())

    const hooks = result.hooks ?? []
    expect(hooks).toHaveLength(1)
    expect(hooks[0].trigger).toBe('PreToolUse')
    expect(hooks[0].command).toBe('./check.sh')
    expect(hooks[0].id).toBe('agent-test-agent-hook-0')
  })

  it('adds agent to agent_profiles in Rust format', () => {
    const agent = makeTestAgent()
    const result = agentToLibrary(agent, makeBaseLibrary())

    const profiles = result.agent_profiles ?? []
    expect(profiles).toHaveLength(1)
    expect(profiles[0].profile.id).toBe('test-agent')
    expect(profiles[0].profile.name).toBe('Test Agent')
    expect(profiles[0].profile.providers).toEqual(['claude', 'gemini'])
    expect(profiles[0].skills?.refs).toEqual(['skill-a', 'skill-b'])
    expect(profiles[0].mcp?.servers).toEqual(['github'])
  })

  it('deduplicates agent_profiles by profile.id', () => {
    const agent = makeTestAgent()
    const base = makeBaseLibrary({
      agent_profiles: [{
        profile: { id: 'test-agent', name: 'Old Name' },
      }],
    })

    const result = agentToLibrary(agent, base)

    expect(result.agent_profiles).toHaveLength(1)
    expect(result.agent_profiles[0].profile.name).toBe('Test Agent')
  })

  it('preserves base library fields not related to agent', () => {
    const base = makeBaseLibrary({ env: { API_KEY: 'test' } })
    const result = agentToLibrary(makeTestAgent(), base)

    expect(result.env).toEqual({ API_KEY: 'test' })
  })

  it('works with an agent that has no skills, servers, or rules', () => {
    const agent = makeTestAgent({
      skills: [],
      mcpServers: [],
      rules: [],
      hooks: [],
    })
    const result = agentToLibrary(agent, makeBaseLibrary())

    expect(result.active_agent).toBe('agent-test-agent')
    const mode = result.modes?.find((m) => m.id === 'agent-test-agent')
    expect(mode!.skills).toEqual([])
    expect(mode!.mcp_servers).toEqual([])
    expect(mode!.rules).toEqual([])
    // Base assets still present
    expect(result.skills).toHaveLength(1)
    expect(result.mcp_servers).toHaveLength(1)
  })

  it('works with DEFAULT_LIBRARY as base', () => {
    const agent = makeTestAgent()
    const result = agentToLibrary(agent, DEFAULT_LIBRARY)

    expect(result.active_agent).toBe('agent-test-agent')
    expect(result.skills).toHaveLength(2)
    expect(result.mcp_servers).toHaveLength(1)
  })

  it('preserves advanced MCP fields (env, disabled, timeout, server_type, url)', () => {
    const agent = makeTestAgent({
      mcpServers: [
        {
          name: 'remote-api',
          command: '',
          args: [],
          server_type: 'sse',
          url: 'https://example.com/mcp',
          timeout_secs: 60,
          disabled: true,
          env: { API_KEY: 'secret-123', DEBUG: 'true' },
          codex_enabled_tools: [],
          codex_disabled_tools: [],
          gemini_include_tools: [],
          gemini_exclude_tools: [],
        },
      ],
    })
    const result = agentToLibrary(agent, makeBaseLibrary())

    const server = result.mcp_servers?.find((s) => s.name === 'remote-api')
    expect(server).toBeDefined()
    expect(server!.server_type).toBe('sse')
    expect(server!.url).toBe('https://example.com/mcp')
    expect(server!.timeout_secs).toBe(60)
    expect(server!.disabled).toBe(true)
    expect(server!.env).toEqual({ API_KEY: 'secret-123', DEBUG: 'true' })
  })
})
