// Lightweight semver comparison — no external dependencies.
// Handles standard major.minor.patch versions with optional pre-release suffix.

export interface SemverParts {
  major: number
  minor: number
  patch: number
  prerelease: string | null
}

/**
 * Parse a semver string into its numeric parts.
 * Returns null if the string is not a valid semver version.
 * Accepts optional leading 'v' prefix (e.g. "v1.2.3").
 */
export function parseSemver(version: string): SemverParts | null {
  const match = version.match(
    /^v?(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9.]+))?$/,
  )
  if (!match) return null
  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    prerelease: match[4] ?? null,
  }
}

/**
 * Compare two semver strings.
 * Returns:
 *   1  if a > b
 *  -1  if a < b
 *   0  if a == b
 *
 * Pre-release versions are considered lower than the same version without
 * a pre-release tag (e.g. 1.0.0-alpha < 1.0.0).
 *
 * Returns 0 if either string is not valid semver (treat as equal / unknown).
 */
export function compareSemver(a: string, b: string): -1 | 0 | 1 {
  const pa = parseSemver(a)
  const pb = parseSemver(b)
  if (!pa || !pb) return 0

  for (const key of ['major', 'minor', 'patch'] as const) {
    if (pa[key] > pb[key]) return 1
    if (pa[key] < pb[key]) return -1
  }

  // Both have same major.minor.patch — compare pre-release
  if (pa.prerelease && !pb.prerelease) return -1
  if (!pa.prerelease && pb.prerelease) return 1
  if (pa.prerelease && pb.prerelease) {
    if (pa.prerelease < pb.prerelease) return -1
    if (pa.prerelease > pb.prerelease) return 1
  }

  return 0
}

/**
 * Returns true if `candidate` is a newer version than `current`.
 * If either string is not valid semver, returns false (safe default).
 */
export function isNewerVersion(candidate: string, current: string): boolean {
  return compareSemver(candidate, current) === 1
}
