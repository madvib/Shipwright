import { describe, it, expect } from 'vitest'
import { parseGithubUrl, extractLibrary } from '#/lib/github-import'

describe('parseGithubUrl', () => {
  it('parses owner/repo', () => {
    expect(parseGithubUrl('https://github.com/anthropics/claude-code')).toEqual({
      owner: 'anthropics',
      repo: 'claude-code',
    })
  })

  it('strips .git suffix', () => {
    expect(parseGithubUrl('https://github.com/foo/bar.git')).toEqual({
      owner: 'foo',
      repo: 'bar',
    })
  })

  it('handles trailing slash', () => {
    expect(parseGithubUrl('https://github.com/foo/bar/')).toMatchObject({
      owner: 'foo',
      repo: 'bar',
    })
  })

  it('returns null for non-github host', () => {
    expect(parseGithubUrl('https://gitlab.com/foo/bar')).toBeNull()
  })

  it('returns null for unparseable string', () => {
    expect(parseGithubUrl('not-a-url')).toBeNull()
  })

  it('returns null when repo segment is missing', () => {
    expect(parseGithubUrl('https://github.com/foo')).toBeNull()
  })
})

describe('extractLibrary', () => {
  it('extracts CLAUDE.md as a rule', () => {
    const lib = extractLibrary({ 'CLAUDE.md': '# Rules\nBe helpful.' })
    expect(lib).not.toBeNull()
    expect(lib!.rules).toEqual([{ file_name: 'CLAUDE.md', content: '# Rules\nBe helpful.' }])
  })

  it('extracts .mcp.json mcp_servers (mcpServers key)', () => {
    const mcp = JSON.stringify({ mcpServers: { 'my-server': { command: 'npx', args: ['my-pkg'] } } })
    const lib = extractLibrary({ '.mcp.json': mcp })
    expect(lib!.mcp_servers).toHaveLength(1)
    expect((lib!.mcp_servers ?? [])[0]).toMatchObject({ name: 'my-server', command: 'npx', args: ['my-pkg'] })
  })

  it('extracts .mcp.json mcp_servers (mcp_servers key)', () => {
    const mcp = JSON.stringify({ mcp_servers: { 'srv': { command: 'node', args: [] } } })
    const lib = extractLibrary({ '.mcp.json': mcp })
    expect((lib!.mcp_servers ?? [])[0].name).toBe('srv')
  })

  it('extracts .cursor/rules/*.mdc as rules', () => {
    const lib = extractLibrary({ '.cursor/rules/react.mdc': '# React' })
    expect((lib!.rules ?? [])[0]).toEqual({ file_name: 'react.mdc', content: '# React' })
  })

  it('extracts AGENTS.md as a rule', () => {
    const lib = extractLibrary({ 'AGENTS.md': 'Be an agent.' })
    expect((lib!.rules ?? [])[0].file_name).toBe('AGENTS.md')
  })

  it('extracts .gemini/GEMINI.md as a rule with filename GEMINI.md', () => {
    const lib = extractLibrary({ '.gemini/GEMINI.md': '# Gemini' })
    expect((lib!.rules ?? [])[0].file_name).toBe('GEMINI.md')
  })

  it('returns null when nothing extractable', () => {
    expect(extractLibrary({ 'README.md': 'hello', 'src/index.ts': 'export {}' })).toBeNull()
  })

  it('returns null for empty files map', () => {
    expect(extractLibrary({})).toBeNull()
  })

  it('detects ship project from .ship/agents/ and extracts rules', () => {
    const lib = extractLibrary({
      '.ship/agents/rules/base.md': '# Base rules',
      '.ship/agents/skills/ship-coordination/SKILL.md': '# Skill content',
    })
    expect(lib).not.toBeNull()
    expect(lib!.rules).toEqual([{ file_name: 'base.md', content: '# Base rules' }])
    expect(lib!.skills).toHaveLength(1)
    expect((lib!.skills ?? [])[0]).toMatchObject({ id: 'ship-coordination', name: 'ship-coordination' })
  })

  it('ship project takes priority over other files', () => {
    const lib = extractLibrary({
      '.ship/agents/rules/base.md': '# Ship rule',
      'CLAUDE.md': '# Claude rule',
    })
    expect(lib!.rules).toHaveLength(1)
    expect((lib!.rules ?? [])[0].file_name).toBe('base.md')
  })

  it('parses mcp.toml [[servers]] from ship project', () => {
    const toml = `
[[servers]]
id = "my-tool"
command = "npx"
args = ["@my/tool"]
`
    const lib = extractLibrary({ '.ship/agents/mcp.toml': toml })
    expect(lib!.mcp_servers).toHaveLength(1)
    expect((lib!.mcp_servers ?? [])[0]).toMatchObject({ name: 'my-tool', command: 'npx', args: ['@my/tool'] })
  })

  it('collects multiple rules in priority order', () => {
    const lib = extractLibrary({
      'CLAUDE.md': 'claude',
      '.cursor/rules/a.mdc': 'cursor-a',
      '.cursor/rules/b.mdc': 'cursor-b',
      'AGENTS.md': 'agents',
    })
    expect((lib!.rules ?? []).map(r => r.file_name)).toEqual(['CLAUDE.md', 'a.mdc', 'b.mdc', 'AGENTS.md'])
  })
})
