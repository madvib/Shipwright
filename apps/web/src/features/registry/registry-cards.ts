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

// ── Stats: hidden until real data exists ─────────────────────────────────────
// Per ui-integrity rule: never display fake data. Show 0 or hide.

export const REGISTRY_STATS = {
  skills: 0,
  agents: 0,
  mcpServers: 0,
  totalInstalls: '0',
}

export const FEATURED_COLLECTION = {
  title: 'Superpowers Skill Pack',
  description:
    '17 skills for brainstorming, code review, TDD, debugging, and more. The official productivity suite.',
  badge: 'Featured Collection',
}

/** Registry cards are populated by the API. Empty until connected. */
export const REGISTRY_CARDS: RegistryCard[] = []

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
