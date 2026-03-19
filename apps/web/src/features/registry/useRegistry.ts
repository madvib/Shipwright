import { useQuery } from '@tanstack/react-query'
import type { RegistrySearchResponse, PackageDetailResponse, ScopeFilter } from './types'
import { MOCK_PACKAGES, MOCK_VERSIONS, MOCK_SKILLS, DEFAULT_MOCK_VERSIONS, DEFAULT_MOCK_SKILLS } from './mock-data'

const ITEMS_PER_PAGE = 12

/** Query key factory for registry cache management. */
export const registryKeys = {
  all: ['registry'] as const,
  search: (q: string, scope: ScopeFilter, page: number) =>
    ['registry', 'search', q, scope, page] as const,
  detail: (path: string) => ['registry', 'detail', path] as const,
}

/**
 * Mock search — filters MOCK_PACKAGES by query and scope.
 * Will be replaced with GET /api/registry/search when the API is deployed.
 */
function mockSearch(query: string, scope: ScopeFilter, page: number): RegistrySearchResponse {
  const q = query.toLowerCase()
  let filtered = MOCK_PACKAGES

  if (scope !== 'all') {
    filtered = filtered.filter((p) => p.scope === scope)
  }

  if (q) {
    filtered = filtered.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q) ||
        p.path.toLowerCase().includes(q),
    )
  }

  // Sort by installs descending
  filtered = [...filtered].sort((a, b) => b.installs - a.installs)

  const start = (page - 1) * ITEMS_PER_PAGE
  const paged = filtered.slice(start, start + ITEMS_PER_PAGE)

  return { packages: paged, total: filtered.length, page }
}

/**
 * Mock package detail — finds a package by path and returns versions + skills.
 * Will be replaced with GET /api/registry/packages/:path when the API is deployed.
 */
function mockDetail(path: string): PackageDetailResponse | null {
  const pkg = MOCK_PACKAGES.find((p) => p.path === path)
  if (!pkg) return null

  const versions = MOCK_VERSIONS[pkg.id] ?? DEFAULT_MOCK_VERSIONS.map((v) => ({ ...v, package_id: pkg.id }))
  const skills = MOCK_SKILLS[pkg.id] ?? DEFAULT_MOCK_SKILLS.map((s) => ({ ...s, package_id: pkg.id }))

  return { package: pkg, versions, skills }
}

/**
 * Search the registry for packages.
 * Uses mock data until the API is deployed, then falls back on fetch error.
 */
export function useRegistrySearch(query: string, scope: ScopeFilter, page: number) {
  return useQuery({
    queryKey: registryKeys.search(query, scope, page),
    queryFn: async (): Promise<RegistrySearchResponse> => {
      try {
        const params = new URLSearchParams()
        if (query) params.set('q', query)
        if (scope !== 'all') params.set('scope', scope)
        params.set('page', String(page))
        params.set('limit', String(ITEMS_PER_PAGE))

        const res = await fetch(`/api/registry/search?${params}`)
        if (!res.ok) throw new Error(`Search failed: ${res.status}`)
        return await res.json() as RegistrySearchResponse
      } catch {
        // Fallback to mock data when API is not deployed
        return mockSearch(query, scope, page)
      }
    },
    staleTime: 30_000,
    placeholderData: (prev) => prev,
  })
}

/**
 * Get full package detail by path.
 * Uses mock data until the API is deployed, then falls back on fetch error.
 */
export function usePackageDetail(path: string) {
  return useQuery({
    queryKey: registryKeys.detail(path),
    queryFn: async (): Promise<PackageDetailResponse | null> => {
      try {
        const encoded = encodeURIComponent(path)
        const res = await fetch(`/api/registry/packages/${encoded}`)
        if (res.status === 404) return null
        if (!res.ok) throw new Error(`Detail failed: ${res.status}`)
        return await res.json() as PackageDetailResponse
      } catch {
        // Fallback to mock data when API is not deployed
        return mockDetail(path)
      }
    },
    staleTime: 60_000,
    enabled: !!path,
  })
}
