import { useQuery } from '@tanstack/react-query'
import type { RegistrySearchResponse, PackageDetailResponse, ScopeFilter } from './types'

export type SortParam = 'installs' | 'recent' | 'name'

const ITEMS_PER_PAGE = 12

/** Query key factory for registry cache management. */
export const registryKeys = {
  all: ['registry'] as const,
  search: (q: string, scope: ScopeFilter, page: number, sort: SortParam) =>
    ['registry', 'search', q, scope, page, sort] as const,
  detail: (path: string) => ['registry', 'detail', path] as const,
}

/** Search the registry for packages. Errors propagate to TanStack Query error state. */
export function useRegistrySearch(
  query: string,
  scope: ScopeFilter,
  page: number,
  sort: SortParam = 'installs',
) {
  return useQuery({
    queryKey: registryKeys.search(query, scope, page, sort),
    queryFn: async (): Promise<RegistrySearchResponse> => {
      const params = new URLSearchParams()
      if (query) params.set('q', query)
      if (scope !== 'all') params.set('scope', scope)
      params.set('page', String(page))
      params.set('limit', String(ITEMS_PER_PAGE))
      if (sort !== 'installs') params.set('sort', sort)

      const res = await fetch(`/api/registry/search?${params}`)
      if (!res.ok) throw new Error(`Registry search failed (${res.status})`)
      return await res.json() as RegistrySearchResponse
    },
    staleTime: 30_000,
    placeholderData: (prev) => prev,
    retry: 1,
  })
}

/** Get full package detail by path. Errors propagate to TanStack Query error state. */
export function usePackageDetail(path: string) {
  return useQuery({
    queryKey: registryKeys.detail(path),
    queryFn: async (): Promise<PackageDetailResponse | null> => {
      const encoded = encodeURIComponent(path)
      const res = await fetch(`/api/registry/packages/${encoded}`)
      if (res.status === 404) return null
      if (!res.ok) throw new Error(`Package detail failed (${res.status})`)
      return await res.json() as PackageDetailResponse
    },
    staleTime: 60_000,
    enabled: !!path,
    retry: 1,
  })
}
