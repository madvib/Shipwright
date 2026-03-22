// POST /api/registry/webhook
//
// GitHub App webhook handler for auto-publish on tag push.
// Validates X-Hub-Signature-256 header against webhook secret.
// Handles 'create' event (ref_type='tag') and 'installation' events.

import { createFileRoute } from '@tanstack/react-router'
import { createRegistryRepositories } from '#/db/registry-repositories'
import { getD1, nanoid } from '#/lib/d1'
import { env as cloudflareEnv } from 'cloudflare:workers'
import {
  fetchFileFromGitHub,
  parseShipToml,
} from '#/lib/registry-github'
import { isNewerVersion } from '#/lib/semver'
import { verifySignature, getPayloadAgeMs } from '#/lib/webhook-verify'

const WEBHOOK_MAX_AGE_MS = 5 * 60 * 1000 // 5 minutes

export const Route = createFileRoute('/api/registry/webhook')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        const secret = (cloudflareEnv as Partial<Env>).GITHUB_WEBHOOK_SECRET

        if (!secret) {
          return Response.json(
            { error: 'Webhook secret not configured' },
            { status: 500 },
          )
        }

        // Validate signature
        const signature = request.headers.get('x-hub-signature-256')
        const body = await request.text()

        if (!signature || !(await verifySignature(secret, body, signature))) {
          return Response.json(
            { error: 'Invalid webhook signature' },
            { status: 401 },
          )
        }

        const event = request.headers.get('x-github-event')
        const payload = JSON.parse(body) as Record<string, unknown>

        // Reject stale payloads to prevent replay attacks (5-minute window)
        const payloadAge = getPayloadAgeMs(payload)
        if (payloadAge !== null && payloadAge > WEBHOOK_MAX_AGE_MS) {
          return Response.json(
            { error: 'Webhook payload too old — rejecting to prevent replay' },
            { status: 400 },
          )
        }

        if (event === 'create' && payload.ref_type === 'tag') {
          return handleTagCreate(payload)
        }

        if (event === 'installation') {
          return handleInstallation(payload)
        }

        if (event === 'installation_repositories') {
          return handleInstallation(payload)
        }

        // Ignore all other events
        return Response.json({ status: 'ignored' })
      },
    },
  },
})

async function handleTagCreate(
  payload: Record<string, unknown>,
): Promise<Response> {
  const repo = payload.repository as Record<string, unknown> | undefined
  if (!repo) {
    return Response.json({ error: 'Missing repository in payload' }, { status: 400 })
  }

  const fullName = repo.full_name as string
  const tag = payload.ref as string
  const defaultBranch = (repo.default_branch as string) || 'main'
  const repoUrl = `https://github.com/${fullName}`
  const [owner, repoName] = fullName.split('/')

  // Fetch .ship/ship.toml at the tag
  const tomlContent = await fetchFileFromGitHub(owner, repoName, '.ship/ship.toml', tag)
  if (!tomlContent) {
    return Response.json({
      status: 'skipped',
      reason: 'no .ship/ship.toml at tag',
    })
  }

  const toml = parseShipToml(tomlContent)
  if (!toml.module?.name) {
    return Response.json({
      status: 'skipped',
      reason: 'ship.toml missing [module] name',
    })
  }

  const d1 = getD1()
  if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

  const repos = createRegistryRepositories(d1)
  const packagePath = `github.com/${fullName}`
  const now = Date.now()

  const existing = await repos.getPackage(packagePath)
  const incomingVersion = toml.module.version || tag
  const shouldUpdateVersion =
    !existing?.latestVersion ||
    isNewerVersion(incomingVersion, existing.latestVersion)
  const latestVersion = shouldUpdateVersion
    ? incomingVersion
    : existing?.latestVersion ?? null
  const pkg = await repos.upsertPackage({
    id: existing?.id || nanoid(),
    path: packagePath,
    scope: existing?.scope || 'community',
    name: toml.module.name,
    description: toml.module.description || null,
    repoUrl,
    defaultBranch,
    latestVersion,
    contentHash: null,
    sourceType: 'native',
    claimedBy: existing?.claimedBy || null,
    deprecatedBy: existing?.deprecatedBy || null,
    stars: existing?.stars ?? 0,
    installs: existing?.installs ?? 0,
    indexedAt: existing?.indexedAt ?? now,
    updatedAt: now,
  })

  const skillIds = toml.exports?.skills || []
  const agentNames = toml.exports?.agents || []

  await repos.createPackageVersion({
    id: nanoid(),
    packageId: pkg.id,
    version: toml.module.version || tag,
    gitTag: tag,
    commitSha: '',
    contentHash: null,
    skillsJson: JSON.stringify(skillIds),
    agentsJson: JSON.stringify(agentNames),
    indexedAt: now,
  })

  return Response.json({
    status: 'indexed',
    package_id: pkg.id,
    version: toml.module.version || tag,
  })
}

async function handleInstallation(
  payload: Record<string, unknown>,
): Promise<Response> {
  const action = payload.action as string
  const installation = payload.installation as Record<string, unknown> | undefined
  if (!installation?.id) {
    return Response.json({ status: 'acknowledged', action })
  }

  const installationId = installation.id as number
  const account = installation.account as Record<string, unknown> | undefined
  const accountLogin = (account?.login as string) ?? 'unknown'
  const accountType = (account?.type as string) ?? 'User'

  const d1 = getD1()
  if (!d1) {
    // No DB — acknowledge without persisting
    return Response.json({ status: 'acknowledged', action })
  }

  const now = Date.now()

  if (action === 'created') {
    const rawRepos = payload.repositories
    const repos = Array.isArray(rawRepos) ? (rawRepos as Array<Record<string, unknown>>) : []
    const repoList = repos.map((r) => ({
      id: r.id as number,
      full_name: r.full_name as string,
      private: r.private as boolean,
    }))
    const id = nanoid()

    await d1
      .prepare(
        `INSERT INTO github_installations (id, installation_id, account_login, account_type, repos_json, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(installation_id) DO UPDATE SET
           account_login = excluded.account_login,
           repos_json = excluded.repos_json,
           updated_at = excluded.updated_at`,
      )
      .bind(id, installationId, accountLogin, accountType, JSON.stringify(repoList), now, now)
      .run()

    return Response.json({ status: 'stored', action, installation_id: installationId })
  }

  if (action === 'deleted') {
    await d1
      .prepare('DELETE FROM github_installations WHERE installation_id = ?')
      .bind(installationId)
      .run()

    return Response.json({ status: 'removed', action, installation_id: installationId })
  }

  // Handle repos added/removed to an existing installation
  if (action === 'added' || action === 'removed') {
    const existing = await d1
      .prepare('SELECT repos_json FROM github_installations WHERE installation_id = ?')
      .bind(installationId)
      .first<{ repos_json: string }>()

    if (existing) {
      let currentRepos = JSON.parse(existing.repos_json) as Array<{ id: number; full_name: string; private: boolean }>

      if (action === 'added') {
        const added = (payload.repositories_added as Array<Record<string, unknown>>) ?? []
        for (const r of added) {
          if (!currentRepos.some((cr) => cr.id === r.id)) {
            currentRepos.push({ id: r.id as number, full_name: r.full_name as string, private: r.private as boolean })
          }
        }
      } else {
        const removed = (payload.repositories_removed as Array<Record<string, unknown>>) ?? []
        const removedIds = new Set(removed.map((r) => r.id as number))
        currentRepos = currentRepos.filter((cr) => !removedIds.has(cr.id))
      }

      await d1
        .prepare('UPDATE github_installations SET repos_json = ?, updated_at = ? WHERE installation_id = ?')
        .bind(JSON.stringify(currentRepos), now, installationId)
        .run()
    }

    return Response.json({ status: 'updated', action, installation_id: installationId })
  }

  return Response.json({ status: 'acknowledged', action })
}

