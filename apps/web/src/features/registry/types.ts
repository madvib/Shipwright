/** Registry package scope — determines trust level and badge color. */
export type PackageScope = 'official' | 'community' | 'unofficial'

/** Source type — how the package was indexed. */
export type PackageSourceType = 'native' | 'imported'

/** A package in the Ship registry. */
export interface RegistryPackage {
  id: string
  path: string
  name: string
  description: string
  scope: PackageScope
  repo_url: string
  default_branch: string
  latest_version: string | null
  content_hash: string | null
  source_type: PackageSourceType
  claimed_by: string | null
  deprecated_by: string | null
  stars: number
  installs: number
  indexed_at: string
  updated_at: string
}

/** A specific version of a package. */
export interface PackageVersion {
  id: string
  package_id: string
  version: string
  git_tag: string
  commit_sha: string
  skills: string[]
  agents: string[]
  indexed_at: string
}

/** A skill exported by a package. */
export interface PackageSkill {
  id: string
  package_id: string
  version_id: string
  skill_id: string
  name: string
  description: string
  content_hash: string
  content_length: number
}

/** Search API response shape. */
export interface RegistrySearchResponse {
  packages: RegistryPackage[]
  total: number
  page: number
}

/** Package detail API response shape. */
export interface PackageDetailResponse {
  package: RegistryPackage
  versions: PackageVersion[]
  skills: PackageSkill[]
}

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
