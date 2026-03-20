/** Card types and mock data for the registry browse page. */

export type TypeFilter = 'all' | 'skills' | 'agents' | 'mcp'
export type CategoryTab = 'trending' | 'new' | 'most-installed' | 'curated'

export interface RegistryCard {
  id: string
  name: string
  author: string
  description: string
  type: 'skill' | 'agent' | 'mcp'
  icon: string
  installs: number
  rating: number
  installed: boolean
  /** Agent cards: number of bundled skills */
  skillCount?: number
  /** Agent cards: number of bundled MCP servers */
  mcpCount?: number
  /** MCP cards: number of tools exposed */
  toolCount?: number
}

// ── Placeholder stats (replaced by API) ──────────────────────────────────────

export const REGISTRY_STATS = {
  skills: 342,
  agents: 89,
  mcpServers: 156,
  totalInstalls: '12.4k',
}

export const FEATURED_COLLECTION = {
  title: 'Superpowers Skill Pack',
  description:
    '17 skills for brainstorming, code review, TDD, debugging, and more. The official productivity suite.',
  badge: 'Featured Collection',
}

// ── Mock card data ───────────────────────────────────────────────────────────

export const REGISTRY_CARDS: RegistryCard[] = [
  // Skills
  {
    id: 'sk-code-review', name: 'code-review', author: '@ship/official', type: 'skill', icon: 'CR',
    description: 'Pre-landing PR review. Analyzes diff for SQL safety, trust boundaries, and structural issues.',
    installs: 2100, rating: 4.8, installed: true,
  },
  {
    id: 'sk-tdd', name: 'test-driven-dev', author: '@ship/official', type: 'skill', icon: 'TD',
    description: 'Write tests before implementation. Red-green-refactor cycle with coverage tracking.',
    installs: 1800, rating: 4.9, installed: false,
  },
  {
    id: 'sk-debug', name: 'systematic-debug', author: '@ship/official', type: 'skill', icon: 'SD',
    description: 'Structured debugging methodology. Root cause analysis before proposing fixes.',
    installs: 1400, rating: 4.7, installed: false,
  },
  {
    id: 'sk-commit', name: 'smart-commit', author: '@ship/official', type: 'skill', icon: 'SC',
    description: 'Enforces conventional commit format with scope and type validation. Atomic commits.',
    installs: 1200, rating: 4.6, installed: false,
  },
  {
    id: 'sk-brainstorm', name: 'brainstorm', author: '@ship/official', type: 'skill', icon: 'BR',
    description: 'Structured brainstorming with divergent thinking, convergence, and prioritization.',
    installs: 980, rating: 4.5, installed: false,
  },
  {
    id: 'sk-refactor', name: 'refactor-guide', author: '@community/cleancode', type: 'skill', icon: 'RG',
    description: 'Guided refactoring with safety checks. Extract, inline, rename with test verification.',
    installs: 870, rating: 4.4, installed: false,
  },
  // Agents
  {
    id: 'ag-fullstack', name: 'fullstack-dev', author: '@ship/starters', type: 'agent', icon: 'FS',
    description: 'Full-stack development agent with React, Node, and database skills. Ship-guarded permissions.',
    installs: 890, rating: 4.7, installed: false, skillCount: 5, mcpCount: 2,
  },
  {
    id: 'ag-rust', name: 'rust-developer', author: '@community/oxidetools', type: 'agent', icon: 'RD',
    description: 'Rust specialist with cargo, clippy, and systems programming skills. Strict permissions.',
    installs: 620, rating: 4.8, installed: false, skillCount: 4, mcpCount: 1,
  },
  {
    id: 'ag-qa', name: 'qa-engineer', author: '@community/testcraft', type: 'agent', icon: 'QA',
    description: 'Testing specialist. E2E, unit, integration testing with Playwright and Vitest.',
    installs: 340, rating: 4.5, installed: false, skillCount: 3, mcpCount: 2,
  },
  {
    id: 'ag-data', name: 'data-engineer', author: '@community/dataops', type: 'agent', icon: 'DE',
    description: 'ETL pipelines, SQL optimization, and data quality checks. Postgres and BigQuery ready.',
    installs: 290, rating: 4.3, installed: false, skillCount: 4, mcpCount: 3,
  },
  // MCP
  {
    id: 'mcp-github', name: 'github', author: '@modelcontextprotocol', type: 'mcp', icon: 'GH',
    description: 'GitHub API integration. PRs, issues, files, code search. 18 tools available.',
    installs: 4200, rating: 4.9, installed: true, toolCount: 18,
  },
  {
    id: 'mcp-playwright', name: 'playwright', author: '@modelcontextprotocol', type: 'mcp', icon: 'PW',
    description: 'Browser automation and testing. Navigate, interact, screenshot, assert.',
    installs: 2800, rating: 4.7, installed: false, toolCount: 12,
  },
  {
    id: 'mcp-postgres', name: 'postgres', author: '@modelcontextprotocol', type: 'mcp', icon: 'PG',
    description: 'PostgreSQL read-only access. Schema inspection, query execution, table browsing.',
    installs: 1900, rating: 4.6, installed: false, toolCount: 8,
  },
  {
    id: 'mcp-memory', name: 'memory', author: '@modelcontextprotocol', type: 'mcp', icon: 'ME',
    description: 'Persistent knowledge graph for context. Store and retrieve facts across sessions.',
    installs: 1500, rating: 4.4, installed: false, toolCount: 6,
  },
  {
    id: 'mcp-linear', name: 'linear', author: '@linear', type: 'mcp', icon: 'LN',
    description: 'Linear project management. Create issues, manage cycles, update project status.',
    installs: 1100, rating: 4.5, installed: false, toolCount: 10,
  },
]

// ── Helpers ──────────────────────────────────────────────────────────────────

export function formatInstalls(n: number): string {
  if (n >= 1_000) return `${(n / 1000).toFixed(1)}k`
  return String(n)
}

export function filterByType(cards: RegistryCard[], filter: TypeFilter): RegistryCard[] {
  if (filter === 'all') return cards
  const singular = filter.replace(/s$/, '') as RegistryCard['type']
  return cards.filter((c) => c.type === singular)
}

export function filterBySearch(cards: RegistryCard[], query: string): RegistryCard[] {
  if (!query) return cards
  const q = query.toLowerCase()
  return cards.filter(
    (c) =>
      c.name.toLowerCase().includes(q) ||
      c.description.toLowerCase().includes(q) ||
      c.author.toLowerCase().includes(q),
  )
}

export const TYPE_ICON_STYLES: Record<RegistryCard['type'], string> = {
  skill: 'bg-primary/10 text-primary',
  agent: 'bg-[oklch(0.35_0.08_300)] text-[oklch(0.75_0.15_300)]',
  mcp: 'bg-[oklch(0.25_0.06_250)] text-[oklch(0.7_0.12_250)]',
}
