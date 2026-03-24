import { describe, it, expect } from 'vitest'
import { libraryToShipFiles } from '#/lib/ship-config'
import type { ProjectLibrary } from '@ship/ui'

const EMPTY_LIB: ProjectLibrary = {
  modes: [],
  active_agent: null,
  mcp_servers: [],
  skills: [],
  rules: [],
  agent_profiles: [],
  claude_team_agents: [],
  env: {},
  available_models: [],
  provider_defaults: {},
}

describe('libraryToShipFiles', () => {
  it('generates ship.toml with mode name', () => {
    const files = libraryToShipFiles(EMPTY_LIB, 'my-mode')
    expect(files['.ship/ship.toml']).toContain('profile = "my-mode"')
    expect(files['.ship/ship.toml']).toContain('version = "1"')
  })

  it('generates rule files under .ship/agents/rules/', () => {
    const lib: ProjectLibrary = {
      ...EMPTY_LIB,
      rules: [
        { file_name: 'CLAUDE.md', content: '# Be helpful' },
        { file_name: 'security.md', content: '# Security rules' },
      ],
    }
    const files = libraryToShipFiles(lib)
    expect(files['.ship/agents/rules/CLAUDE.md']).toBe('# Be helpful')
    expect(files['.ship/agents/rules/security.md']).toBe('# Security rules')
  })

  it('generates skill files under .ship/agents/skills/', () => {
    const lib: ProjectLibrary = {
      ...EMPTY_LIB,
      skills: [
        { id: 'commit', name: 'Commit', content: '# Smart commit' },
      ],
    }
    const files = libraryToShipFiles(lib)
    expect(files['.ship/agents/skills/commit/SKILL.md']).toBe('# Smart commit')
  })

  it('generates mcp.toml with [[servers]] blocks', () => {
    const lib: ProjectLibrary = {
      ...EMPTY_LIB,
      mcp_servers: [
        { name: 'github', command: 'npx', args: ['-y', '@mcp/server-github'], url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
        { name: 'memory', command: 'npx', args: ['-y', '@mcp/server-memory'], url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
      ],
    }
    const files = libraryToShipFiles(lib)
    const toml = files['.ship/agents/mcp.toml']
    expect(toml).toContain('[[servers]]')
    expect(toml).toContain('id = "github"')
    expect(toml).toContain('command = "npx"')
    expect(toml).toContain('id = "memory"')
  })

  it('generates mcp.toml with env variables', () => {
    const lib: ProjectLibrary = {
      ...EMPTY_LIB,
      mcp_servers: [
        { name: 'github', command: 'npx', args: [], env: { GITHUB_TOKEN: '$GITHUB_TOKEN' }, url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
      ],
    }
    const files = libraryToShipFiles(lib)
    expect(files['.ship/agents/mcp.toml']).toContain('GITHUB_TOKEN = "$GITHUB_TOKEN"')
  })

  it('generates preset toml referencing mcp and skills', () => {
    const lib: ProjectLibrary = {
      ...EMPTY_LIB,
      mcp_servers: [{ name: 'github', command: 'npx', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] }],
      skills: [{ id: 'commit', name: 'Commit', content: '...' }],
    }
    const files = libraryToShipFiles(lib, 'dev')
    const preset = files['.ship/agents/presets/dev.toml']
    expect(preset).toContain('name = "dev"')
    expect(preset).toContain('mcp_servers = ["github"]')
    expect(preset).toContain('skills = ["commit"]')
  })

  it('generates permissions.toml when permissions provided', () => {
    const lib: ProjectLibrary = {
      ...EMPTY_LIB,
      permissions: {
        tools: { allow: ['Bash(npm test)', 'Read'], deny: ['Bash(rm)'] },
        filesystem: { allow: ['**/*'], deny: [] },
        commands: { allow: [], deny: [] },
        network: { policy: 'none', allow_hosts: [] },
        agent: { require_confirmation: [] },
      },
    }
    const files = libraryToShipFiles(lib)
    expect(files['.ship/agents/permissions.toml']).toContain('allow = ["Bash(npm test)", "Read"]')
    expect(files['.ship/agents/permissions.toml']).toContain('deny = ["Bash(rm)"]')
  })

  it('skips mcp.toml when no servers', () => {
    const files = libraryToShipFiles(EMPTY_LIB)
    expect(files['.ship/agents/mcp.toml']).toBeUndefined()
  })

  it('sanitizes rule filenames', () => {
    const lib: ProjectLibrary = {
      ...EMPTY_LIB,
      rules: [{ file_name: 'some/weird/path.md', content: 'content' }],
    }
    const files = libraryToShipFiles(lib)
    expect(Object.keys(files).some(k => k.includes('some-weird-path.md'))).toBe(true)
  })
})
