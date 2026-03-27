import type { Package } from '#/db/registry-schema'

// ── Enum narrowing ──────────────────────────────────────────────────────────

/** Registry package scope — determines trust level and badge color. */
export type PackageScope = 'official' | 'community' | 'unofficial'

/** Source type — how the package was indexed. */
export type PackageSourceType = 'native' | 'imported'

// ── Package types (derived from Drizzle schema) ─────────────────────────────

/** Full package with narrowed enum fields. Derives from D1 schema. */
export type RegistryPackage = Omit<Package, 'scope' | 'sourceType'> & {
  scope: PackageScope
  sourceType: PackageSourceType
}

/** Subset of Package fields returned by the search API. */
export type SearchPackage = Pick<RegistryPackage,
  | 'id' | 'path' | 'name' | 'description' | 'scope'
  | 'latestVersion' | 'updatedAt' | 'installs' | 'stars'
  | 'deprecatedBy' | 'repoUrl' | 'claimedBy'
>

// ── API response types (match actual API shapes) ────────────────────────────

/** Version as returned by the detail API (parsed skills/agents arrays). */
export interface PackageVersion {
  id: string
  version: string
  gitTag: string
  commitSha: string
  skills: string[]
  agents: string[]
  indexedAt: number
}

/** Skill as returned by the detail API. */
export interface PackageSkill {
  id: string
  skillId: string
  name: string
  description: string | null
  contentHash: string
}

/** Search API response shape. */
export interface RegistrySearchResponse {
  packages: SearchPackage[]
  total: number
  page: number
}

/** Package detail API response shape. */
export interface PackageDetailResponse {
  package: RegistryPackage
  versions: PackageVersion[]
  skills: PackageSkill[]
}

// ── UI constants ────────────────────────────────────────────────────────────

/** Scope filter option for the browse UI. */
export type ScopeFilter = 'all' | PackageScope

export const SCOPE_FILTERS: { value: ScopeFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'official', label: 'Official' },
  { value: 'community', label: 'Community' },
  { value: 'unofficial', label: 'Unofficial' },
]

/** Scope badge color mapping. */
export const SCOPE_COLORS: Record<PackageScope, { bg: string; text: string; border: string }> = {
  official: { bg: 'bg-blue-500/10', text: 'text-blue-500', border: 'border-blue-500/20' },
  community: { bg: 'bg-emerald-500/10', text: 'text-emerald-500', border: 'border-emerald-500/20' },
  unofficial: { bg: 'bg-muted', text: 'text-muted-foreground', border: 'border-border/60' },
}

/** Extract the GitHub owner from a repo URL. Returns null if not a GitHub URL. */
export function extractGitHubOwner(repoUrl: string): string | null {
  const match = repoUrl.match(/github\.com\/([^/]+)/)
  return match?.[1] ?? null
}
