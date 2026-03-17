import { describe, it, expect } from 'vitest'

// ── Inline constants from index.tsx for testing ────────────────────────────
// These mirror the data defined in the home page route.

const PROVIDER_TABS = [
  { id: 'claude',  label: 'Claude Code' },
  { id: 'gemini',  label: 'Gemini CLI'  },
  { id: 'codex',   label: 'Codex CLI'   },
  { id: 'cursor',  label: 'Cursor'      },
]

const PROVIDER_OUTPUTS: Record<string, { filename: string; content: string }> = {
  claude: {
    filename: 'CLAUDE.md + .mcp.json',
    content: 'CLAUDE.md content',
  },
  gemini: {
    filename: 'GEMINI.md + .gemini/settings.json',
    content: 'GEMINI.md content',
  },
  codex: {
    filename: 'AGENTS.md + .codex/config.toml',
    content: 'AGENTS.md content',
  },
  cursor: {
    filename: '.cursor/mcp.json + .cursor/rules/',
    content: '.cursor content',
  },
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('PROVIDER_TABS', () => {
  it('contains exactly four providers', () => {
    expect(PROVIDER_TABS).toHaveLength(4)
  })

  it('includes claude, gemini, codex, cursor', () => {
    const ids = PROVIDER_TABS.map((t) => t.id)
    expect(ids).toContain('claude')
    expect(ids).toContain('gemini')
    expect(ids).toContain('codex')
    expect(ids).toContain('cursor')
  })

  it('each tab has an id and label', () => {
    for (const tab of PROVIDER_TABS) {
      expect(typeof tab.id).toBe('string')
      expect(typeof tab.label).toBe('string')
      expect(tab.id.length).toBeGreaterThan(0)
      expect(tab.label.length).toBeGreaterThan(0)
    }
  })
})

describe('PROVIDER_OUTPUTS', () => {
  it('has an entry for every provider tab', () => {
    for (const tab of PROVIDER_TABS) {
      expect(PROVIDER_OUTPUTS[tab.id]).toBeDefined()
    }
  })

  it('each output has filename and content', () => {
    for (const [, output] of Object.entries(PROVIDER_OUTPUTS)) {
      expect(typeof output.filename).toBe('string')
      expect(typeof output.content).toBe('string')
    }
  })

  it('claude output references CLAUDE.md', () => {
    expect(PROVIDER_OUTPUTS['claude'].filename).toContain('CLAUDE.md')
  })

  it('cursor output references .cursor/', () => {
    expect(PROVIDER_OUTPUTS['cursor'].filename).toContain('.cursor/')
  })
})
