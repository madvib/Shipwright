import { createFileRoute, Link } from '@tanstack/react-router'
import { useState, useDeferredValue } from 'react'
import { Search, Package, X, ChevronLeft, ChevronRight, Upload } from 'lucide-react'
import { useRegistrySearch } from '#/features/registry/useRegistry'
import { PackageCard } from '#/features/registry/PackageCard'
import { EmptyState } from '#/components/EmptyState'
import { SCOPE_FILTERS } from '#/features/registry/types'
import type { ScopeFilter } from '#/features/registry/types'

export const Route = createFileRoute('/registry/')({ component: RegistryBrowsePage })

function RegistryBrowsePage() {
  const [query, setQuery] = useState('')
  const [scope, setScope] = useState<ScopeFilter>('all')
  const [page, setPage] = useState(1)

  // Defer the search query to avoid blocking input
  const deferredQuery = useDeferredValue(query)
  const { data, isLoading, isFetching } = useRegistrySearch(deferredQuery, scope, page)

  const packages = data?.packages ?? []
  const total = data?.total ?? 0
  const totalPages = Math.max(1, Math.ceil(total / 12))

  // Reset to page 1 when filters change
  function handleScopeChange(s: ScopeFilter) {
    setScope(s)
    setPage(1)
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-auto p-5 pb-20">

        {/* Header */}
        <div className="mb-5 flex items-start justify-between gap-3">
          <div>
            <h1 className="text-base font-semibold text-foreground mb-1">Registry</h1>
            <p className="text-[11px] text-muted-foreground">
              Browse agent packages — skills, MCP servers, and presets for your AI workflow.
            </p>
          </div>
          <Link
            to={'/studio/registry/publish' as '/'}
            className="shrink-0 inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-3 py-2 text-[11px] font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
          >
            <Upload className="size-3" />
            Publish
          </Link>
        </div>

        {/* Search + filters */}
        <div className="flex flex-col sm:flex-row gap-3 mb-5">
          <div className="flex items-center gap-2 rounded-lg border border-border/60 bg-card px-3 py-2 flex-1 max-w-md">
            <Search className="size-3.5 text-muted-foreground shrink-0" />
            <input
              value={query}
              onChange={(e) => { setQuery(e.target.value); setPage(1) }}
              placeholder="Search packages..."
              className="flex-1 bg-transparent text-sm text-foreground placeholder:text-muted-foreground focus:outline-none min-w-0"
            />
            {query && (
              <button onClick={() => { setQuery(''); setPage(1) }} className="text-muted-foreground hover:text-foreground">
                <X className="size-3.5" />
              </button>
            )}
          </div>

          {/* Scope tabs */}
          <div className="flex items-center gap-1 rounded-lg border border-border/40 bg-muted/30 p-0.5">
            {SCOPE_FILTERS.map((f) => (
              <button
                key={f.value}
                onClick={() => handleScopeChange(f.value)}
                className={`rounded-md px-3 py-1.5 text-[11px] font-medium transition-colors ${
                  scope === f.value
                    ? 'bg-card text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {f.label}
              </button>
            ))}
          </div>
        </div>

        {/* Loading state */}
        {isLoading && (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className="rounded-xl border border-border/40 bg-card/50 p-4 animate-pulse">
                <div className="h-4 bg-muted/50 rounded w-2/3 mb-3" />
                <div className="h-3 bg-muted/30 rounded w-full mb-2" />
                <div className="h-3 bg-muted/30 rounded w-4/5" />
              </div>
            ))}
          </div>
        )}

        {/* Package grid */}
        {!isLoading && packages.length > 0 && (
          <>
            <div className="flex items-center justify-between mb-3">
              <span className="text-[10px] text-muted-foreground/60">
                {total} package{total !== 1 ? 's' : ''}
                {isFetching && ' ...'}
              </span>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              {packages.map((pkg) => (
                <PackageCard key={pkg.id} pkg={pkg} />
              ))}
            </div>
          </>
        )}

        {/* Empty state */}
        {!isLoading && packages.length === 0 && (
          <EmptyState
            icon={<Package className="size-5" />}
            title="No packages found"
            description={query ? `No results for "${query}" — try a different search term or scope.` : 'No packages match the current filter.'}
            action={
              <div className="flex items-center gap-2">
                {(query || scope !== 'all') && (
                  <button
                    onClick={() => { setQuery(''); setScope('all'); setPage(1) }}
                    className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
                  >
                    Clear filters
                  </button>
                )}
                <Link
                  to={'/studio/registry/publish' as '/'}
                  className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
                >
                  <Upload className="size-3" />
                  Publish a package
                </Link>
              </div>
            }
          />
        )}

        {/* Pagination */}
        {!isLoading && totalPages > 1 && (
          <Pagination page={page} totalPages={totalPages} onPageChange={setPage} />
        )}
      </div>
    </div>
  )
}

/** Pagination controls. */
function Pagination({ page, totalPages, onPageChange }: { page: number; totalPages: number; onPageChange: (p: number) => void }) {
  return (
    <div className="flex items-center justify-center gap-2 mt-6">
      <button
        onClick={() => onPageChange(Math.max(1, page - 1))}
        disabled={page <= 1}
        className="flex items-center gap-1 rounded-lg border border-border/40 bg-card px-3 py-1.5 text-[11px] font-medium text-muted-foreground transition hover:border-border hover:text-foreground disabled:opacity-30 disabled:cursor-not-allowed"
      >
        <ChevronLeft className="size-3" />
        Previous
      </button>
      <span className="text-[11px] text-muted-foreground/60 px-2">
        {page} of {totalPages}
      </span>
      <button
        onClick={() => onPageChange(Math.min(totalPages, page + 1))}
        disabled={page >= totalPages}
        className="flex items-center gap-1 rounded-lg border border-border/40 bg-card px-3 py-1.5 text-[11px] font-medium text-muted-foreground transition hover:border-border hover:text-foreground disabled:opacity-30 disabled:cursor-not-allowed"
      >
        Next
        <ChevronRight className="size-3" />
      </button>
    </div>
  )
}
