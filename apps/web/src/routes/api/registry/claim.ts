// POST /api/registry/claim — claim an @unofficial package
// Requires GitHub auth. Validates the user is an admin/owner of the source repo.
//
// Body: { package_path: string }
// The package_path is the registry path (e.g. "unofficial/owner-repo").

import { createFileRoute } from '@tanstack/react-router'
import { requireSession } from '#/lib/session-auth'
import { getUser } from '#/lib/github-app'
import { getGitHubToken } from '#/lib/github-token'
import { getRegistryDb } from '#/lib/d1'
import { createRegistryRepositories } from '#/db/registry-repositories'
import { checkRateLimit, rateLimitResponse } from '#/lib/rate-limit'

interface ClaimBody {
  package_path: string
}

interface RepoPermission {
  permission: string
  role_name: string
}

/**
 * Check if a GitHub user has admin/maintain permission on a repo.
 * Uses GET /repos/:owner/:repo/collaborators/:username/permission
 */
async function checkRepoPermission(
  token: string,
  owner: string,
  repo: string,
  username: string,
): Promise<boolean> {
  const res = await fetch(
    `https://api.github.com/repos/${owner}/${repo}/collaborators/${username}/permission`,
    {
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: 'application/vnd.github.v3+json',
        'User-Agent': 'ship-studio/1.0',
      },
    },
  )
  if (!res.ok) return false

  const data = (await res.json()) as RepoPermission
  return data.permission === 'admin' || data.permission === 'write' || data.role_name === 'admin'
}

/**
 * Extract owner/repo from a GitHub URL stored in the package record.
 */
function parseRepoUrl(url: string): { owner: string; repo: string } | null {
  try {
    const parsed = new URL(url)
    if (parsed.hostname !== 'github.com') return null
    const parts = parsed.pathname.replace(/^\//, '').split('/')
    if (parts.length < 2 || !parts[0] || !parts[1]) return null
    return { owner: parts[0], repo: parts[1].replace(/\.git$/, '') }
  } catch {
    return null
  }
}

export const Route = createFileRoute('/api/registry/claim')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        const rl = await checkRateLimit(request, 'RATE_LIMITER_CLAIM', 3600)
        if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

        // Require authenticated session
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        // Require GitHub token (from Better Auth account)
        const ghToken = await getGitHubToken(request)
        if (!ghToken) {
          return Response.json(
            { error: 'GitHub authentication required. Connect your GitHub account first.' },
            { status: 401 },
          )
        }

        // Parse request body
        let body: ClaimBody
        try {
          body = (await request.json()) as ClaimBody
        } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        if (!body.package_path || typeof body.package_path !== 'string') {
          return Response.json({ error: 'Missing package_path' }, { status: 400 })
        }

        const d1 = getRegistryDb()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRegistryRepositories(d1)
        const pkg = await repos.getPackage(body.package_path)

        if (!pkg) {
          return Response.json({ error: 'Package not found' }, { status: 404 })
        }

        // Already claimed by someone else
        if (pkg.claimedBy && pkg.claimedBy !== auth.sub) {
          return Response.json(
            { error: 'Package already claimed by another user' },
            { status: 409 },
          )
        }

        // Already claimed by this user
        if (pkg.claimedBy === auth.sub) {
          return Response.json({ claimed: true, package_path: pkg.path })
        }

        // Validate GitHub repo ownership
        const repoParsed = parseRepoUrl(pkg.repoUrl)
        if (!repoParsed) {
          return Response.json(
            { error: 'Package has no valid GitHub repo URL' },
            { status: 422 },
          )
        }

        const ghUser = await getUser(ghToken)
        const hasPermission = await checkRepoPermission(
          ghToken,
          repoParsed.owner,
          repoParsed.repo,
          ghUser.login,
        )

        if (!hasPermission) {
          return Response.json(
            { error: `You must be an admin or maintainer of ${repoParsed.owner}/${repoParsed.repo} to claim this package` },
            { status: 403 },
          )
        }

        // Set claimedBy and transition scope from 'unofficial' to 'community'
        const newScope = pkg.scope === 'unofficial' ? 'community' : pkg.scope
        await repos.upsertPackage({ ...pkg, claimedBy: auth.sub, scope: newScope, updatedAt: Date.now() })

        return Response.json({ claimed: true, package_path: pkg.path })
      },
    },
  },
})
