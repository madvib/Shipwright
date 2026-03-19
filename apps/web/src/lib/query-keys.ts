// Query key factory for TanStack Query cache invalidation.
// Follows the pattern from bene: hierarchical arrays for granular invalidation.

export const studioKeys = {
  all: ['studio'] as const,
  library: (id: string) => ['studio', 'library', id] as const,
  profiles: (id: string) => ['studio', 'profiles', id] as const,
  workspaces: () => ['studio', 'workspaces'] as const,
  workspace: (id: string) => ['studio', 'workspaces', id] as const,
} as const

export const libraryKeys = {
  all: () => ['libraries'] as const,
  list: () => [...libraryKeys.all(), 'list'] as const,
  detail: (id: string) => [...libraryKeys.all(), id] as const,
}

export const authKeys = {
  all: ['auth'] as const,
  session: () => ['auth', 'session'] as const,
  me: () => ['auth', 'me'] as const,
} as const

export const githubKeys = {
  all: ['github'] as const,
  repos: (page?: number) => ['github', 'repos', page ?? 1] as const,
} as const

export const profileKeys = {
  all: () => ['profiles'] as const,
  list: () => [...profileKeys.all(), 'list'] as const,
  detail: (id: string) => [...profileKeys.all(), id] as const,
}

export const workflowKeys = {
  all: () => ['workflows'] as const,
  list: () => [...workflowKeys.all(), 'list'] as const,
  detail: (id: string) => [...workflowKeys.all(), id] as const,
}

export const registryKeys = {
  all: () => ['registry'] as const,
  search: (q: string, scope: string, page: number) => [...registryKeys.all(), 'search', q, scope, page] as const,
  detail: (path: string) => [...registryKeys.all(), 'detail', path] as const,
}
