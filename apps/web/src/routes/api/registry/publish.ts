// POST /api/registry/publish
//
// Body: { repo_url: string, tag?: string }
//
// Fetches .ship/ship.toml from the repo at the given tag (or HEAD).
// Validates [module] section, reads [exports], indexes skill metadata.
// Auth optional: authenticated users get claimed_by set.

import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod/v4'
import { createRegistryRepositories } from '#/db/registry-repositories'
import { getD1, nanoid } from '#/lib/d1'
import { optionalSession } from '#/lib/session-auth'
import {
  parseGithubUrl,
  fetchFileFromGitHub,
  parseShipToml,
} from '#/lib/registry-github'

const PublishInput = z.object({
  repo_url: z.string().min(1, 'repo_url is required'),
  tag: z.string().optional(),
})

export const Route = createFileRoute('/api/registry/publish')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        let body: unknown
        try {
          body = await request.json()
        } catch {
          return Response.json(
            { error: 'Invalid JSON body' },
            { status: 400 },
          )
        }

        const parsed = PublishInput.safeParse(body)
        if (!parsed.success) {
          const msg = parsed.error.issues.map((i) => i.message).join('; ')
          return Response.json({ error: msg }, { status: 400 })
        }

        const { repo_url, tag } = parsed.data
        const ghParsed = parseGithubUrl(repo_url)
        if (!ghParsed) {
          return Response.json(
            { error: 'Malformed GitHub URL — expected https://github.com/owner/repo' },
            { status: 400 },
          )
        }

        const d1 = getD1()
        if (!d1)
          return Response.json(
            { error: 'Database unavailable' },
            { status: 503 },
          )

        const session = await optionalSession(request)
        const ref = tag || 'HEAD'

        // Fetch .ship/ship.toml
        const tomlContent = await fetchFileFromGitHub(
          ghParsed.owner,
          ghParsed.repo,
          '.ship/ship.toml',
          ref,
        )
        if (!tomlContent) {
          return Response.json(
            {
              error:
                'No .ship/ship.toml found in this repository. ' +
                'Create a .ship/ship.toml with [module] and [exports] sections to publish.',
            },
            { status: 422 },
          )
        }

        const toml = parseShipToml(tomlContent)
        if (!toml.module) {
          return Response.json(
            { error: '.ship/ship.toml missing [module] section with name and version' },
            { status: 422 },
          )
        }

        const repos = createRegistryRepositories(d1)
        const packagePath = `github.com/${ghParsed.owner}/${ghParsed.repo}`

        // Check for ownership conflict
        const existing = await repos.getPackage(packagePath)
        if (existing?.claimedBy && session && existing.claimedBy !== session.sub) {
          return Response.json(
            { error: 'Package already claimed by another user' },
            { status: 409 },
          )
        }

        const now = Date.now()
        const pkg = await repos.upsertPackage({
          id: existing?.id || nanoid(),
          path: packagePath,
          scope: 'community',
          name: toml.module.name,
          description: toml.module.description || null,
          repoUrl: repo_url,
          defaultBranch: 'main',
          latestVersion: toml.module.version || null,
          contentHash: null,
          sourceType: 'native',
          claimedBy: session?.sub || existing?.claimedBy || null,
          deprecatedBy: existing?.deprecatedBy || null,
          stars: existing?.stars ?? 0,
          installs: existing?.installs ?? 0,
          indexedAt: existing?.indexedAt ?? now,
          updatedAt: now,
        })

        // Create version record
        const versionId = nanoid()
        const skillIds: string[] = []
        const agentNames: string[] = []

        if (toml.exports) {
          if (toml.exports.skills) skillIds.push(...toml.exports.skills)
          if (toml.exports.agents) agentNames.push(...toml.exports.agents)
        }

        const version = await repos.createPackageVersion({
          id: versionId,
          packageId: pkg.id,
          version: toml.module.version || '0.0.0',
          gitTag: tag || 'HEAD',
          commitSha: '', // Populated by webhook flow with actual SHA
          contentHash: null,
          skillsJson: JSON.stringify(skillIds),
          agentsJson: JSON.stringify(agentNames),
          indexedAt: now,
        })

        // Index exported skills
        let skillsIndexed = 0
        for (const skillId of skillIds) {
          const skillPath = `.ship/skills/${skillId}.md`
          const content = await fetchFileFromGitHub(
            ghParsed.owner,
            ghParsed.repo,
            skillPath,
            ref,
          )
          if (!content) continue

          const hash = await computeContentHash(content)
          await repos.createPackageSkill({
            id: nanoid(),
            packageId: pkg.id,
            versionId: version.id,
            skillId,
            name: skillId,
            description: extractSkillDescription(content),
            contentHash: hash,
            contentLength: new TextEncoder().encode(content).length,
          })
          skillsIndexed++
        }

        return Response.json({
          package_id: pkg.id,
          version: version.version,
          skills_indexed: skillsIndexed,
        })
      },
    },
  },
})

async function computeContentHash(content: string): Promise<string> {
  const normalized = content.replace(/\r\n/g, '\n').trim()
  const encoded = new TextEncoder().encode(normalized)
  const hash = await crypto.subtle.digest('SHA-256', encoded)
  return Array.from(new Uint8Array(hash), (b) =>
    b.toString(16).padStart(2, '0'),
  ).join('')
}

function extractSkillDescription(content: string): string | null {
  // Extract description from YAML frontmatter if present
  const match = content.match(/^---\n([\s\S]*?)\n---/)
  if (!match) return null
  const frontmatter = match[1]
  const descLine = frontmatter
    .split('\n')
    .find((l) => l.startsWith('description:'))
  if (!descLine) return null
  return descLine.replace('description:', '').trim().replace(/^["']|["']$/g, '')
}
