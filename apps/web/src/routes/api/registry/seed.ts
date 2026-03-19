// POST /api/registry/seed — admin endpoint to batch-import @unofficial packages
// Reads seed-repos.json (or accepts body with repo URLs), imports agent configs,
// and indexes them as @unofficial packages.

import { createFileRoute } from '@tanstack/react-router'
import { requireSession } from '#/lib/session-auth'
import { getD1, nanoid } from '#/lib/d1'
import { parseGithubUrl, extractLibrary } from '#/lib/github-import'
import { fetchRepoFiles } from '#/lib/fetch-repo-files'
import { computeContentHash } from '#/lib/content-hash'
import { createRegistryRepositories, type RegistryRepositories } from '#/db/registry-repositories'
import type { InsertPackage } from '#/db/schema'
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
  skipped: number
  errors: string[]
}

async function importRepo(
  entry: SeedEntry,
  token: string | undefined,
  db: RegistryRepositories,
): Promise<'imported' | 'skipped' | string> {
  const parsed = parseGithubUrl(entry.url)
  if (!parsed) return `Invalid URL: ${entry.url}`

  const { owner, repo } = parsed
  const files = await fetchRepoFiles(owner, repo, token)
  if (files === 'not_found') return `Repo not found: ${owner}/${repo}`

  const library = extractLibrary(files)
  if (!library) return 'skipped'

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
  // Note: no version record for imported packages (no git tag)
  for (const rule of library.rules) {
    const hash = await computeContentHash(rule.content)
    await db.createPackageSkill({
      id: nanoid(),
      packageId: pkg.id,
      versionId: '',
      skillId: rule.file_name.replace(/\.(md|mdc)$/, ''),
      name: rule.file_name,
      description: null,
      contentHash: hash,
      contentLength: new TextEncoder().encode(rule.content).byteLength,
    })
  }

  // Index native skills from .ship/ projects
  for (const skill of library.skills) {
    const hash = await computeContentHash(skill.content)
    await db.createPackageSkill({
      id: nanoid(),
      packageId: pkg.id,
      versionId: '',
      skillId: skill.id,
      name: skill.name,
      description: null,
      contentHash: hash,
      contentLength: new TextEncoder().encode(skill.content).byteLength,
    })
  }

  return 'imported'
}

export const Route = createFileRoute('/api/registry/seed')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const d1 = getD1()
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

        const result: SeedResult = { imported: 0, skipped: 0, errors: [] }

        for (const entry of repos) {
          try {
            const outcome = await importRepo(entry, token || undefined, db)
            if (outcome === 'imported') result.imported++
            else if (outcome === 'skipped') result.skipped++
            else result.errors.push(outcome)
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
