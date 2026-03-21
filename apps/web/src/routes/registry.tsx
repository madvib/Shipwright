import { createFileRoute } from '@tanstack/react-router'
import type { ElementType } from 'react'
import { useState, useDeferredValue, useCallback } from 'react'
import { Search, Download, Clock, Heart, TrendingUp, AlertTriangle, RefreshCw } from 'lucide-react'
import { useRegistrySearch } from '#/features/registry/useRegistry'
import type { SortParam } from '#/features/registry/useRegistry'
import { PackageCard } from '#/features/registry/PackageCard'
import { Pagination } from '#/features/registry/Pagination'
import type { TypeFilter, CategoryTab } from '#/features/registry/registry-cards'
import type { ScopeFilter } from '#/features/registry/types'

export const Route = createFileRoute('/registry')({
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

/** Map category tab to sort order and optional scope override. */
const CATEGORY_CONFIG: Record<CategoryTab, { sort: SortParam; scopeOverride?: ScopeFilter }> = {
  trending: { sort: 'installs' },
  new: { sort: 'recent' },
  'most-installed': { sort: 'installs' },
  curated: { sort: 'installs', scopeOverride: 'official' },
}

const ITEMS_PER_PAGE = 12

// ── Page ─────────────────────────────────────────────────────────────────────

function RegistryPage() {
  const [query, setQuery] = useState('')
  const [typeFilter, setTypeFilter] = useState<TypeFilter>('all')
  const [category, setCategory] = useState<CategoryTab>('trending')
  const [page, setPage] = useState(1)

  const deferredQuery = useDeferredValue(query)

  const handleQueryChange = useCallback((q: string) => {
    setQuery(q)
    setPage(1)
  }, [])

  const handleTypeFilterChange = useCallback((f: TypeFilter) => {
    setTypeFilter(f)
    setPage(1)
  }, [])

  const handleCategoryChange = useCallback((tab: CategoryTab) => {
    setCategory(tab)
    setPage(1)
  }, [])

  const config = CATEGORY_CONFIG[category]
  const scope = config.scopeOverride ?? SCOPE_MAP[typeFilter]

  const { data, error, isLoading, refetch } = useRegistrySearch(
    deferredQuery,
    scope,
    page,
    config.sort,
  )

  const packages = data?.packages ?? []
  const total = data?.total ?? 0
  const totalPages = Math.max(1, Math.ceil(total / ITEMS_PER_PAGE))

  return (
    <div className="h-full flex flex-col overflow-auto">
      <div className="max-w-[960px] w-full mx-auto px-5 pb-24">
        <HeroSection
          query={query}
          onQueryChange={handleQueryChange}
          typeFilter={typeFilter}
          onTypeFilterChange={handleTypeFilterChange}
        />

        <CategoryTabs active={category} onChange={handleCategoryChange} />

        {error ? (
          <ErrorState error={error} onRetry={() => void refetch()} />
        ) : isLoading ? (
          <LoadingState />
        ) : packages.length === 0 ? (
          <EmptyResults
            query={query}
            typeFilter={typeFilter}
            onClear={() => {
              setQuery('')
              setTypeFilter('all')
              setPage(1)
            }}
          />
        ) : (
          <div className="mb-7">
            <div className="text-[11px] font-semibold text-muted-foreground/50 uppercase tracking-wider mb-3 pl-1">
              {deferredQuery ? 'Results' : 'Packages'}{' '}
              <span className="text-muted-foreground/30">({total})</span>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2.5">
              {packages.map((pkg) => (
                <PackageCard key={pkg.id} pkg={pkg} />
              ))}
            </div>
            {totalPages > 1 && (
              <Pagination page={page} totalPages={totalPages} onPageChange={setPage} />
            )}
          </div>
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

function ErrorState({ error, onRetry }: { error: Error; onRetry: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      <div className="flex size-12 items-center justify-center rounded-2xl border border-destructive/20 bg-destructive/5 text-destructive mb-3">
        <AlertTriangle className="size-5" />
      </div>
      <p className="text-sm font-medium text-foreground">Failed to load registry</p>
      <p className="mt-1 text-xs text-muted-foreground max-w-xs">
        {error.message || 'An unexpected error occurred while fetching packages.'}
      </p>
      <button
        onClick={onRetry}
        className="mt-3 inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
      >
        <RefreshCw className="size-3" />
        Retry
      </button>
    </div>
  )
}

function LoadingState() {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2.5">
      {Array.from({ length: 6 }).map((_, i) => (
        <div
          key={i}
          className="animate-pulse rounded-xl border border-border/60 bg-card p-4"
        >
          <div className="h-4 bg-muted/50 rounded w-2/3 mb-3" />
          <div className="h-3 bg-muted/30 rounded w-full mb-2" />
          <div className="h-3 bg-muted/30 rounded w-1/2" />
        </div>
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
