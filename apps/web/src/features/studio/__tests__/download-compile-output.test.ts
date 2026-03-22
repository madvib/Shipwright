import { describe, it, expect } from 'vitest'
import { collectProviderFiles, collectShipSourceFiles, buildShipManifest, buildAgentConfig, buildZip, crc32 } from '../download-compile-output'
import type { CompileResult } from '#/features/compiler/types'
import type { AgentProfile } from '#/features/agents/types'
import { DEFAULT_SETTINGS } from '#/features/agents/types'
import { DEFAULT_PERMISSIONS } from '@ship/ui'
import type { ProjectLibrary } from '@ship/ui'

function makeResult(overrides: Partial<CompileResult> = {}): CompileResult {
  return {
    mcp_servers: null,
    mcp_config_path: null,
    context_content: null,
    skill_files: {},
    claude_settings_patch: null,
    codex_config_patch: null,
    gemini_settings_patch: null,
    gemini_policy_patch: null,
    cursor_hooks_patch: null,
    cursor_cli_permissions: null,
    rule_files: {},
    plugins_manifest: { install: [], scope: 'project' },
    agent_files: {},
    cursor_environment_json: null,
    ...overrides,
  }
}

describe('collectProviderFiles', () => {
  it('returns context file with correct name per provider', () => {
    const result = makeResult({ context_content: '# Hello' })

    const claude = collectProviderFiles('claude', result)
    expect(claude).toEqual([{ path: 'CLAUDE.md', content: '# Hello' }])

    const gemini = collectProviderFiles('gemini', result)
    expect(gemini).toEqual([{ path: 'GEMINI.md', content: '# Hello' }])

    const codex = collectProviderFiles('codex', result)
    expect(codex).toEqual([{ path: 'AGENTS.md', content: '# Hello' }])
  })

  it('returns mcp config with correct path per provider', () => {
    const servers = { test: { command: 'test' } }
    const result = makeResult({ mcp_servers: servers })

    const claude = collectProviderFiles('claude', result)
    expect(claude[0].path).toBe('.mcp.json')

    const cursor = collectProviderFiles('cursor', result)
    expect(cursor[0].path).toBe('.cursor/mcp.json')

    const gemini = collectProviderFiles('gemini', result)
    expect(gemini[0].path).toBe('.gemini/settings.json')
  })

  it('includes skill_files, agent_files, and rule_files', () => {
    const result = makeResult({
      skill_files: { '.claude/skills/foo/SKILL.md': 'skill content' },
      agent_files: { '.claude/agents/monitor.md': 'agent content' },
      rule_files: { '.cursor/rules/style.mdc': 'rule content' },
    })

    const files = collectProviderFiles('claude', result)
    expect(files).toContainEqual({ path: '.cursor/rules/style.mdc', content: 'rule content' })
    expect(files).toContainEqual({ path: '.claude/skills/foo/SKILL.md', content: 'skill content' })
    expect(files).toContainEqual({ path: '.claude/agents/monitor.md', content: 'agent content' })
  })

  it('includes claude_settings_patch as JSON', () => {
    const patch = { permissions: { allow: ['Bash(*)'] } }
    const result = makeResult({ claude_settings_patch: patch })
    const files = collectProviderFiles('claude', result)
    expect(files).toContainEqual({
      path: '.claude/settings.json',
      content: JSON.stringify(patch, null, 2),
    })
  })

  it('includes codex_config_patch', () => {
    const toml = '[mcp_servers.test]\ncommand = "test"'
    const result = makeResult({ codex_config_patch: toml })
    const files = collectProviderFiles('codex', result)
    expect(files).toContainEqual({ path: '.codex/config.toml', content: toml })
  })

  it('includes gemini_settings_patch only for gemini provider', () => {
    const patch = { hooks: {} }
    const result = makeResult({ gemini_settings_patch: patch })

    const gemini = collectProviderFiles('gemini', result)
    const geminiSettings = gemini.filter((f) => f.path === '.gemini/settings.json')
    expect(geminiSettings.length).toBeGreaterThan(0)

    const claude = collectProviderFiles('claude', result)
    const claudeGeminiSettings = claude.filter((f) => f.path === '.gemini/settings.json')
    expect(claudeGeminiSettings.length).toBe(0)
  })

  it('includes cursor-specific files', () => {
    const result = makeResult({
      cursor_hooks_patch: { hooks: [] },
      cursor_cli_permissions: { version: 1 },
      cursor_environment_json: { env: 'test' },
    })
    const files = collectProviderFiles('cursor', result)
    expect(files).toContainEqual({
      path: '.cursor/hooks.json',
      content: JSON.stringify({ hooks: [] }, null, 2),
    })
    expect(files).toContainEqual({
      path: '.cursor/cli.json',
      content: JSON.stringify({ version: 1 }, null, 2),
    })
    expect(files).toContainEqual({
      path: '.cursor/environment.json',
      content: JSON.stringify({ env: 'test' }, null, 2),
    })
  })

  it('returns empty array when result has no content', () => {
    const result = makeResult()
    const files = collectProviderFiles('claude', result)
    expect(files).toEqual([])
  })

  it('skips null entries in skill/agent/rule files', () => {
    const result = makeResult({
      skill_files: { 'a.md': 'content', 'b.md': undefined as unknown as string },
      agent_files: { 'c.md': undefined as unknown as string },
    })
    const files = collectProviderFiles('claude', result)
    expect(files).toHaveLength(1)
    expect(files[0].path).toBe('a.md')
  })
})

describe('crc32', () => {
  it('computes correct CRC for empty data', () => {
    expect(crc32(new Uint8Array([]))).toBe(0)
  })

  it('computes correct CRC for known input', () => {
    const data = new TextEncoder().encode('hello')
    // Known CRC-32 of "hello" is 0x3610a686
    expect(crc32(data)).toBe(0x3610a686)
  })
})

describe('buildZip', () => {
  it('creates a valid zip with correct header signatures', () => {
    const encoder = new TextEncoder()
    const zip = buildZip([
      { path: 'test.txt', data: encoder.encode('hello world') },
    ])

    // Check local file header signature
    const view = new DataView(zip.buffer)
    expect(view.getUint32(0, true)).toBe(0x04034b50)

    // Find EOCD signature at end
    const eocdOffset = zip.length - 22
    expect(view.getUint32(eocdOffset, true)).toBe(0x06054b50)

    // EOCD says 1 entry
    expect(view.getUint16(eocdOffset + 8, true)).toBe(1)
  })

  it('creates a zip with multiple files', () => {
    const encoder = new TextEncoder()
    const zip = buildZip([
      { path: 'a.txt', data: encoder.encode('aaa') },
      { path: 'b.txt', data: encoder.encode('bbb') },
      { path: 'dir/c.txt', data: encoder.encode('ccc') },
    ])

    const view = new DataView(zip.buffer)
    const eocdOffset = zip.length - 22
    expect(view.getUint16(eocdOffset + 8, true)).toBe(3)
  })

  it('stores file content correctly', () => {
    const encoder = new TextEncoder()
    const content = 'test content here'
    const zip = buildZip([
      { path: 'file.txt', data: encoder.encode(content) },
    ])

    // Verify the content is stored in the zip (STORE method = uncompressed)
    const zipString = new TextDecoder().decode(zip)
    expect(zipString).toContain(content)
  })
})

function makeAgent(overrides: Partial<AgentProfile> = {}): AgentProfile {
  return {
    id: 'test-agent',
    name: 'Test Agent',
    description: 'A test agent for unit tests',
    providers: ['claude', 'gemini'],
    version: '0.1.0',
    skills: [
      { id: 'skill-a', name: 'skill-a', content: '', source: 'custom' },
      { id: 'skill-b', name: 'skill-b', content: '', source: 'community' },
    ],
    mcpServers: [
      { name: 'github', command: 'npx', args: [], server_type: 'stdio', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
      { name: 'filesystem', command: 'npx', args: [], server_type: 'stdio', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
    ],
    subagents: [],
    permissions: { ...DEFAULT_PERMISSIONS },
    permissionPreset: 'ship-guarded',
    settings: { ...DEFAULT_SETTINGS },
    hooks: [],
    rules: [],
    mcpToolStates: {},
    ...overrides,
  }
}

describe('buildShipManifest', () => {
  it('generates manifest with agent name and skills', () => {
    const agent = makeAgent()
    const raw = buildShipManifest(undefined, agent)
    const manifest = JSON.parse(raw)

    expect(manifest.$schema).toBe('https://getship.dev/schemas/ship.schema.json')
    expect(manifest.module.name).toBe('Test Agent')
    expect(manifest.module.version).toBe('0.1.0')
    expect(manifest.module.description).toBe('A test agent for unit tests')
    expect(manifest.exports.skills).toEqual(['skill-a', 'skill-b'])
    expect(manifest.exports.agents).toEqual(['agents/test-agent.jsonc'])
  })

  it('falls back to my-project when no agent is provided', () => {
    const raw = buildShipManifest()
    const manifest = JSON.parse(raw)

    expect(manifest.module.name).toBe('my-project')
    expect(manifest.module.description).toBe('')
    expect(manifest.exports.skills).toEqual([])
    expect(manifest.exports.agents).toEqual([])
  })

  it('uses library skills when no agent is provided', () => {
    const library: ProjectLibrary = {
      skills: [{ id: 'lib-skill', name: 'lib-skill', content: '', source: 'custom' as const }],
      agent_profiles: [],
      claude_team_agents: [],
      env: {},
      available_models: [],
    }
    const raw = buildShipManifest(library)
    const manifest = JSON.parse(raw)

    expect(manifest.exports.skills).toEqual(['lib-skill'])
  })

  it('prefers agent skills over library skills', () => {
    const agent = makeAgent({ skills: [{ id: 'agent-skill', name: 'agent-skill', content: '', source: 'custom' }] })
    const library: ProjectLibrary = {
      skills: [{ id: 'lib-skill', name: 'lib-skill', content: '', source: 'custom' as const }],
      agent_profiles: [],
      claude_team_agents: [],
      env: {},
      available_models: [],
    }
    const raw = buildShipManifest(library, agent)
    const manifest = JSON.parse(raw)

    expect(manifest.exports.skills).toEqual(['agent-skill'])
  })
})

describe('buildAgentConfig', () => {
  it('generates valid agent config with schema ref', () => {
    const agent = makeAgent()
    const raw = buildAgentConfig(agent)
    const config = JSON.parse(raw)

    expect(config.$schema).toBe('https://getship.dev/schemas/agent.schema.json')
    expect(config.agent.id).toBe('test-agent')
    expect(config.agent.name).toBe('Test Agent')
    expect(config.agent.providers).toEqual(['claude', 'gemini'])
    expect(config.skills.refs).toEqual(['skill-a', 'skill-b'])
    expect(config.mcp.servers).toEqual(['github', 'filesystem'])
    expect(config.permissions).toEqual(DEFAULT_PERMISSIONS)
  })

  it('handles agent with no skills or mcp servers', () => {
    const agent = makeAgent({ skills: [], mcpServers: [] })
    const raw = buildAgentConfig(agent)
    const config = JSON.parse(raw)

    expect(config.skills.refs).toEqual([])
    expect(config.mcp.servers).toEqual([])
  })
})

describe('collectShipSourceFiles', () => {
  it('always includes ship.jsonc', () => {
    const files = collectShipSourceFiles()
    expect(files).toHaveLength(1)
    expect(files[0].path).toBe('.ship/ship.jsonc')

    const manifest = JSON.parse(files[0].content)
    expect(manifest.$schema).toContain('ship.schema.json')
  })

  it('includes agent config when activeAgent is provided', () => {
    const agent = makeAgent()
    const files = collectShipSourceFiles(undefined, agent)

    expect(files).toHaveLength(2)
    expect(files[0].path).toBe('.ship/ship.jsonc')
    expect(files[1].path).toBe('.ship/agents/test-agent.jsonc')

    const agentConfig = JSON.parse(files[1].content)
    expect(agentConfig.agent.id).toBe('test-agent')
  })

  it('does not include agent config when no activeAgent', () => {
    const files = collectShipSourceFiles({ skills: [], agent_profiles: [], claude_team_agents: [], env: {}, available_models: [] })
    expect(files).toHaveLength(1)
    expect(files[0].path).toBe('.ship/ship.jsonc')
  })
})
