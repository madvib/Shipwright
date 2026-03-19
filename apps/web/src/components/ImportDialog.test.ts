import { describe, it, expect } from 'vitest'
import { filterLibraryBySelection } from './ImportDialog'
import type { ProjectLibrary } from '../features/compiler/types'

function makeLibrary(overrides: Partial<ProjectLibrary> = {}): ProjectLibrary {
  return {
    modes: [],
    active_mode: null,
    skills: [],
    rules: [],
    mcp_servers: [],
    agent_profiles: [],
    claude_team_agents: [],
    env: {},
    available_models: [],
    ...overrides,
  }
}

describe('filterLibraryBySelection', () => {
  it('returns empty library when nothing selected', () => {
    const lib = makeLibrary({
      skills: [{ id: 'foo', name: 'Foo', content: '# Foo' }],
      rules: [{ file_name: 'CLAUDE.md', content: '# Rules' }],
      mcp_servers: [{ name: 'github', command: 'npx', url: null, timeout_secs: null }],
    })
    const result = filterLibraryBySelection(lib, {
      skills: new Set(),
      rules: new Set(),
      mcp_servers: new Set(),
    })
    expect(result.skills).toHaveLength(0)
    expect(result.rules).toHaveLength(0)
    expect(result.mcp_servers).toHaveLength(0)
  })

  it('returns all items when all selected', () => {
    const lib = makeLibrary({
      skills: [{ id: 'foo', name: 'Foo', content: '# Foo' }, { id: 'bar', name: 'Bar', content: '# Bar' }],
      rules: [{ file_name: 'CLAUDE.md', content: '# Rules' }],
      mcp_servers: [{ name: 'github', command: 'npx', url: null, timeout_secs: null }],
    })
    const result = filterLibraryBySelection(lib, {
      skills: new Set(['foo', 'bar']),
      rules: new Set(['CLAUDE.md']),
      mcp_servers: new Set(['github']),
    })
    expect(result.skills).toHaveLength(2)
    expect(result.rules).toHaveLength(1)
    expect(result.mcp_servers).toHaveLength(1)
  })

  it('filters skills by id', () => {
    const lib = makeLibrary({
      skills: [
        { id: 'foo', name: 'Foo', content: '# Foo' },
        { id: 'bar', name: 'Bar', content: '# Bar' },
      ],
    })
    const result = filterLibraryBySelection(lib, {
      skills: new Set(['foo']),
      rules: new Set(),
      mcp_servers: new Set(),
    })
    expect(result.skills).toHaveLength(1)
    expect(result.skills?.[0].id).toBe('foo')
  })

  it('filters rules by file_name', () => {
    const lib = makeLibrary({
      rules: [
        { file_name: 'CLAUDE.md', content: '# Claude' },
        { file_name: 'AGENTS.md', content: '# Agents' },
      ],
    })
    const result = filterLibraryBySelection(lib, {
      skills: new Set(),
      rules: new Set(['AGENTS.md']),
      mcp_servers: new Set(),
    })
    expect(result.rules).toHaveLength(1)
    expect(result.rules?.[0].file_name).toBe('AGENTS.md')
  })

  it('filters mcp_servers by name', () => {
    const lib = makeLibrary({
      mcp_servers: [
        { name: 'github', command: 'npx', url: null, timeout_secs: null },
        { name: 'filesystem', command: 'npx', url: null, timeout_secs: null },
      ],
    })
    const result = filterLibraryBySelection(lib, {
      skills: new Set(),
      rules: new Set(),
      mcp_servers: new Set(['filesystem']),
    })
    expect(result.mcp_servers).toHaveLength(1)
    expect(result.mcp_servers?.[0].name).toBe('filesystem')
  })

  it('preserves non-filtered fields on the library', () => {
    const lib = makeLibrary({
      modes: [{ id: 'test-mode', name: 'default', description: '', mcp_servers: [], skills: [], rules: [] }],
      active_mode: 'default',
    })
    const result = filterLibraryBySelection(lib, {
      skills: new Set(),
      rules: new Set(),
      mcp_servers: new Set(),
    })
    expect(result.modes).toHaveLength(1)
    expect(result.active_mode).toBe('default')
  })

  it('handles empty library without error', () => {
    const lib = makeLibrary()
    const result = filterLibraryBySelection(lib, {
      skills: new Set(),
      rules: new Set(),
      mcp_servers: new Set(),
    })
    expect(result.skills).toHaveLength(0)
    expect(result.rules).toHaveLength(0)
    expect(result.mcp_servers).toHaveLength(0)
  })
})
