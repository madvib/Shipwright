import { createFileRoute } from '@tanstack/react-router'
import type { ElementType } from 'react'
import { useState, useDeferredValue, useCallback } from 'react'
import { Search, Download, Clock, Heart, TrendingUp } from 'lucide-react'
import { useRegistrySearch } from '#/features/registry/useRegistry'
import { FeaturedBanner, CardSection } from '#/features/registry/RegistryCardGrid'
import {
  REGISTRY_CARDS,
  REGISTRY_STATS,
  filterByType,
  filterBySearch,
} from '#/features/registry/registry-cards'
import type { TypeFilter, CategoryTab } from '#/features/registry/registry-cards'
import type { ScopeFilter } from '#/features/registry/types'

export const Route = createFileRoute('/studio/registry')({
  component: RegistryPage,
  ssr: false,
})

// ── Constants ────────────────────────────────────────────────────────────────

const TYPE_FILTERS: { value: TypeFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'skills', label: 'Skills' },
  { value: 'agents', label: 'Agents' },
  { value: 'mcp', label: 'MCP' },
]

const CATEGORY_TABS: {
  id: CategoryTab
  label: string
  icon: ElementType
}[] = [
  { id: 'trending', label: 'Trending', icon: TrendingUp },
  { id: 'new', label: 'New', icon: Clock },
  { id: 'most-installed', label: 'Most installed', icon: Download },
  { id: 'curated', label: 'Curated', icon: Heart },
]

/** Map type filter to scope for the search API. */
const SCOPE_MAP: Record<TypeFilter, ScopeFilter> = {
  all: 'all',
  skills: 'official',
  agents: 'community',
  mcp: 'unofficial',
}

// ── Page ─────────────────────────────────────────────────────────────────────

function RegistryPage() {
  const [query, setQuery] = useState('')
  const [typeFilter, setTypeFilter] = useState<TypeFilter>('all')
  const [category, setCategory] = useState<CategoryTab>('trending')
  const [installedIds, setInstalledIds] = useState<Set<string>>(() => {
    return new Set(
      REGISTRY_CARDS.filter((c) => c.installed).map((c) => c.id),
    )
  })

  const deferredQuery = useDeferredValue(query)

  // Wire into real API search -- falls back to mock when API is unavailable
  useRegistrySearch(deferredQuery, SCOPE_MAP[typeFilter], 1)

  // Local filtering over mock cards until API supports type-based search
  const filtered = filterBySearch(
    filterByType(REGISTRY_CARDS, typeFilter),
    deferredQuery,
  )

  const handleInstall = useCallback((id: string) => {
    setInstalledIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
        console.log(`[Registry] Install requested: ${id}`)
      }
      return next
    })
  }, [])

  // Group by type for sectioned display when showing all
  const skills = filtered.filter((c) => c.type === 'skill')
  const agents = filtered.filter((c) => c.type === 'agent')
  const mcpServers = filtered.filter((c) => c.type === 'mcp')
  const showSections = typeFilter === 'all' && !deferredQuery

  return (
    <div className="h-full flex flex-col overflow-auto">
      <div className="max-w-[960px] w-full mx-auto px-5 pb-24">
        <HeroSection
          query={query}
          onQueryChange={setQuery}
          typeFilter={typeFilter}
          onTypeFilterChange={setTypeFilter}
        />

        {/* Stats bar: hidden until registry has real data */}
        {(REGISTRY_STATS.skills > 0 || REGISTRY_STATS.agents > 0) && <StatsBar />}

        <CategoryTabs active={category} onChange={setCategory} />

        {/* Featured banner: hidden until a real pack is published */}

        {filtered.length === 0 ? (
          <EmptyResults
            query={query}
            typeFilter={typeFilter}
            onClear={() => {
              setQuery('')
              setTypeFilter('all')
            }}
          />
        ) : showSections ? (
          <>
            {skills.length > 0 && (
              <CardSection
                title="Popular Skills"
                cards={skills}
                installedIds={installedIds}
                onInstall={handleInstall}
              />
            )}
            {agents.length > 0 && (
              <CardSection
                title="Popular Agents"
                cards={agents}
                installedIds={installedIds}
                onInstall={handleInstall}
              />
            )}
            {mcpServers.length > 0 && (
              <CardSection
                title="Popular MCP Servers"
                cards={mcpServers}
                installedIds={installedIds}
                onInstall={handleInstall}
              />
            )}
          </>
        ) : (
          <CardSection
            title="Results"
            cards={filtered}
            installedIds={installedIds}
            onInstall={handleInstall}
          />
        )}
      </div>
    </div>
  )
}

// ── Sub-components ───────────────────────────────────────────────────────────

function HeroSection({
  query,
  onQueryChange,
  typeFilter,
  onTypeFilterChange,
}: {
  query: string
  onQueryChange: (q: string) => void
  typeFilter: TypeFilter
  onTypeFilterChange: (f: TypeFilter) => void
}) {
  return (
    <div className="pt-10 pb-8 text-center max-w-[680px] mx-auto">
      <h1 className="font-display text-[28px] font-extrabold text-foreground mb-2">
        Ship <span className="text-primary">Registry</span>
      </h1>
      <p className="text-sm text-muted-foreground mb-6 leading-relaxed">
        Skills, agents, and MCP servers for every coding agent. Install with
        one click.
      </p>

      <div className="flex items-center gap-0 max-w-[560px] mx-auto rounded-[10px] border border-border/60 bg-card transition-colors focus-within:border-primary">
        <div className="pl-3.5 flex items-center text-muted-foreground">
          <Search className="size-[18px]" />
        </div>
        <input
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          placeholder="Search packages..."
          className="flex-1 bg-transparent text-sm text-foreground placeholder:text-muted-foreground/50 px-3.5 py-3 focus:outline-none min-w-0"
        />
        <div className="flex items-center gap-0.5 pr-1.5">
          {TYPE_FILTERS.map((f) => (
            <button
              key={f.value}
              onClick={() => onTypeFilterChange(f.value)}
              className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                typeFilter === f.value
                  ? 'text-primary bg-primary/10 border border-primary/20'
                  : 'text-muted-foreground border border-transparent hover:text-foreground'
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}

function StatsBar() {
  return (
    <div className="flex justify-center gap-6 pb-6 max-w-[560px] mx-auto">
      <StatItem value={String(REGISTRY_STATS.skills)} label="Skills" />
      <StatItem value={String(REGISTRY_STATS.agents)} label="Agents" />
      <StatItem
        value={String(REGISTRY_STATS.mcpServers)}
        label="MCP Servers"
      />
      <StatItem value={REGISTRY_STATS.totalInstalls} label="Installs" />
    </div>
  )
}

function StatItem({ value, label }: { value: string; label: string }) {
  return (
    <div className="text-center">
      <div className="text-lg font-bold text-foreground">{value}</div>
      <div className="text-[10px] text-muted-foreground/50 uppercase tracking-wider">
        {label}
      </div>
    </div>
  )
}

function CategoryTabs({
  active,
  onChange,
}: {
  active: CategoryTab
  onChange: (tab: CategoryTab) => void
}) {
  return (
    <div className="flex justify-center gap-0.5 pb-5">
      {CATEGORY_TABS.map(({ id, label, icon: Icon }) => (
        <button
          key={id}
          onClick={() => onChange(id)}
          className={`flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-xs font-medium transition-colors ${
            active === id
              ? 'text-primary bg-primary/10'
              : 'text-muted-foreground hover:text-foreground hover:bg-muted/40'
          }`}
        >
          <Icon className="size-3.5" />
          {label}
        </button>
      ))}
    </div>
  )
}

function EmptyResults({
  query,
  typeFilter,
  onClear,
}: {
  query: string
  typeFilter: TypeFilter
  onClear: () => void
}) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      <div className="flex size-12 items-center justify-center rounded-2xl border border-border/60 bg-muted/40 text-muted-foreground/40 mb-3">
        <Search className="size-5" />
      </div>
      <p className="text-sm font-medium text-foreground">No packages found</p>
      <p className="mt-1 text-xs text-muted-foreground max-w-xs">
        {query
          ? `No results for "${query}". Try a different search term.`
          : 'No packages match the current filter.'}
      </p>
      {(query || typeFilter !== 'all') && (
        <button
          onClick={onClear}
          className="mt-3 inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
        >
          Clear filters
        </button>
      )}
    </div>
  )
}
