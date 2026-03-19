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

        if (event === 'create' && payload.ref_type === 'tag') {
          return handleTagCreate(payload)
        }

        if (event === 'installation') {
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
  const pkg = await repos.upsertPackage({
    id: existing?.id || nanoid(),
    path: packagePath,
    scope: existing?.scope || 'community',
    name: toml.module.name,
    description: toml.module.description || null,
    repoUrl,
    defaultBranch,
    latestVersion: toml.module.version || tag,
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
  // Log installation events for now — full storage TBD
  return Response.json({
    status: 'acknowledged',
    action,
  })
}

async function verifySignature(
  secret: string,
  body: string,
  signature: string,
): Promise<boolean> {
  const encoder = new TextEncoder()
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign'],
  )
  const sig = await crypto.subtle.sign('HMAC', key, encoder.encode(body))
  const expected =
    'sha256=' +
    Array.from(new Uint8Array(sig), (b) =>
      b.toString(16).padStart(2, '0'),
    ).join('')

  // Constant-time comparison
  if (expected.length !== signature.length) return false
  let result = 0
  for (let i = 0; i < expected.length; i++) {
    result |= expected.charCodeAt(i) ^ signature.charCodeAt(i)
  }
  return result === 0
}
