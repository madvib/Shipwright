// POST /api/registry/seed — admin endpoint to batch-import @unofficial packages
// Reads seed-repos.json (or accepts body with repo URLs), imports agent configs,
// and indexes them as @unofficial packages.

import { createFileRoute } from '@tanstack/react-router'
import { env as cloudflareEnv } from 'cloudflare:workers'
import { requireSession } from '#/lib/session-auth'
import { getRegistryDb, nanoid } from '#/lib/d1'
import { parseGithubUrl, extractLibrary } from '#/lib/github-import'
import { fetchRepoFiles } from '#/lib/fetch-repo-files'
import { computeContentHash } from '#/lib/content-hash'
import { fetchFileFromGitHub, parseShipToml } from '#/lib/registry-github'
import { createRegistryRepositories, type RegistryRepositories } from '#/db/registry-repositories'
import type { InsertPackage } from '#/db/registry-schema'
import seedReposJson from '#/lib/seed-repos.json'

/** Convert "owner/repo" to a human-friendly package name. */
function humanName(owner: string, repo: string): string {
  const capitalize = (s: string) =>
    s.replace(/[-_]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
  return `${capitalize(owner)} ${capitalize(repo)}`
}

interface SeedEntry {
  url: string
  category?: string
}

interface SeedResult {
  imported: number
  skipped: string[]
  errors: string[]
}

async function importRepo(
  entry: SeedEntry,
  token: string | undefined,
  db: RegistryRepositories,
): Promise<{ status: 'imported' | 'skipped' | 'error'; reason?: string }> {
  const parsed = parseGithubUrl(entry.url)
  if (!parsed) return { status: 'error', reason: `Invalid URL: ${entry.url}` }

  const { owner, repo } = parsed
  const files = await fetchRepoFiles(owner, repo, token)
  if (files === 'not_found') {
    return { status: 'skipped', reason: `Repo not found: ${owner}/${repo}` }
  }

  const library = extractLibrary(files)
  if (!library) {
    return { status: 'skipped', reason: `No agent config found: ${owner}/${repo}` }
  }

  const pkgPath = `unofficial/${owner}-${repo}`.toLowerCase()
  const now = Date.now()

  const pkgData: InsertPackage = {
    id: nanoid(),
    path: pkgPath,
    scope: 'unofficial',
    name: humanName(owner, repo),
    description: `Agent configuration imported from ${owner}/${repo}`,
    repoUrl: entry.url,
    defaultBranch: 'main',
    latestVersion: null,
    contentHash: null,
    sourceType: 'imported',
    claimedBy: null,
    deprecatedBy: null,
    stars: 0,
    installs: 0,
    indexedAt: now,
    updatedAt: now,
  }

  const pkg = await db.upsertPackage(pkgData)

  // Index skills: rules become skills in the package
  for (const rule of (library.rules ?? [])) {
    const hash = await computeContentHash(rule.content)
    await db.createPackageSkill({
      id: nanoid(),
      packageId: pkg.id,
      versionId: '',
      skillId: rule.file_name.replace(/\.(md|mdc)$/, ''),
      name: rule.file_name,
      description: null,
      contentHash: hash,
    })
  }

  // Index native skills from .ship/agents/ directories
  for (const skill of (library.skills ?? [])) {
    const hash = await computeContentHash(skill.content)
    await db.createPackageSkill({
      id: nanoid(),
      packageId: pkg.id,
      versionId: '',
      skillId: skill.id,
      name: skill.name,
      description: null,
      contentHash: hash,
    })
  }

  // Index native Ship skills from .ship/ship.toml [exports].skills
  await indexShipTomlSkills(owner, repo, pkg.id, db)

  return { status: 'imported' }
}

/** Fetch .ship/ship.toml and index any exported skills from .ship/skills/. */
async function indexShipTomlSkills(
  owner: string,
  repo: string,
  packageId: string,
  db: RegistryRepositories,
): Promise<void> {
  const tomlContent = await fetchFileFromGitHub(owner, repo, '.ship/ship.toml', 'HEAD')
  if (!tomlContent) return

  const toml = parseShipToml(tomlContent)
  const skillIds = toml.exports?.skills ?? []
  if (skillIds.length === 0) return

  for (const skillId of skillIds) {
    const content = await fetchFileFromGitHub(
      owner, repo, `.ship/skills/${skillId}.md`, 'HEAD',
    )
    if (!content) continue

    const hash = await computeContentHash(content)
    await db.createPackageSkill({
      id: nanoid(),
      packageId,
      versionId: '',
      skillId,
      name: skillId,
      description: null,
      contentHash: hash,
    })
  }
}

export const Route = createFileRoute('/api/registry/seed')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const seedSecret = (cloudflareEnv as Partial<Env>).SEED_SECRET
        const providedSecret = request.headers.get('X-Seed-Secret')
        if (!seedSecret || !providedSecret || providedSecret !== seedSecret) {
          return Response.json({ error: 'Unauthorized' }, { status: 401 })
        }

        const d1 = getRegistryDb()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const db = createRegistryRepositories(d1)
        const token = process.env.GITHUB_TOKEN

        // Accept body with custom repos, or fall back to seed-repos.json
        let repos: SeedEntry[]
        try {
          const body = await request.json() as { repos?: SeedEntry[] }
          repos = Array.isArray(body?.repos) ? body.repos : seedReposJson
        } catch {
          repos = seedReposJson
        }

        const result: SeedResult = { imported: 0, skipped: [], errors: [] }

        for (const entry of repos) {
          try {
            const outcome = await importRepo(entry, token || undefined, db)
            if (outcome.status === 'imported') result.imported++
            else if (outcome.status === 'skipped') {
              result.skipped.push(outcome.reason ?? entry.url)
            } else {
              result.errors.push(outcome.reason ?? entry.url)
            }
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err)
            result.errors.push(`${entry.url}: ${msg}`)
          }
        }

        return Response.json(result)
      },
    },
  },
})
