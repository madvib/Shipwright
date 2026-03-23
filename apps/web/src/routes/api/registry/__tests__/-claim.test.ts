import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({ env: { AUTH_DB: {}, REGISTRY_DB: {} } }))

vi.mock('#/lib/session-auth', () => ({
  requireSession: vi.fn(),
}))

vi.mock('#/lib/github-token', () => ({
  getGitHubToken: vi.fn(),
}))

vi.mock('#/lib/github-app', () => ({
  getUser: vi.fn(),
}))

vi.mock('#/db/registry-repositories', () => ({
  createRegistryRepositories: vi.fn(),
}))

vi.mock('#/lib/d1', () => ({
  getRegistryDb: vi.fn(),
  nanoid: vi.fn(() => 'test-id'),
}))

vi.mock('#/lib/rate-limit', () => ({
  checkRateLimit: vi.fn().mockResolvedValue({ allowed: true, retryAfter: 0 }),
  rateLimitResponse: vi.fn(),
}))

import { Route } from '../claim'
import * as sessionAuth from '#/lib/session-auth'
import * as ghToken from '#/lib/github-token'
import * as githubApp from '#/lib/github-app'
import * as registryRepositories from '#/db/registry-repositories'
import * as d1Lib from '#/lib/d1'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const POST = (Route.options.server!.handlers as any).POST!

const VALID_PACKAGE = {
  id: 'pkg-1',
  path: 'github.com/testowner/testrepo',
  repoUrl: 'https://github.com/testowner/testrepo',
  claimedBy: null,
  scope: 'community',
  name: 'test-pack',
  description: null,
  latestVersion: '1.0.0',
  stars: 0,
  installs: 0,
  deprecatedBy: null,
  defaultBranch: 'main',
  contentHash: null,
  sourceType: 'native',
  indexedAt: Date.now(),
  updatedAt: Date.now(),
}

function makeRepos(overrides: Record<string, unknown> = {}) {
  return {
    getPackage: vi.fn().mockResolvedValue(VALID_PACKAGE),
    upsertPackage: vi.fn().mockResolvedValue(VALID_PACKAGE),
    searchPackages: vi.fn(),
    getLatestVersion: vi.fn().mockResolvedValue(null),
    getPackageVersions: vi.fn(),
    getPackageSkills: vi.fn(),
    createPackageVersion: vi.fn(),
    createPackageSkill: vi.fn(),
    incrementInstalls: vi.fn(),
    incrementStars: vi.fn(),
    deprecatePackage: vi.fn(),
    deletePackageSkillsByVersion: vi.fn(),
    ...overrides,
  }
}

function makeRequest(body: unknown): Request {
  return new Request('http://localhost/api/registry/claim', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
  })
}

beforeEach(() => {
  vi.mocked(d1Lib.getRegistryDb).mockReturnValue({} as D1Database)
  vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'user-1', org: 'user-1' })
  vi.mocked(ghToken.getGitHubToken).mockResolvedValue('gh-token-abc')
  vi.mocked(githubApp.getUser).mockResolvedValue({ login: 'testowner', avatar_url: '' })

  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
    ok: true,
    json: () => Promise.resolve({ permission: 'admin', role_name: 'admin' }),
  }))

  vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
    makeRepos() as ReturnType<typeof registryRepositories.createRegistryRepositories>
  )
})

describe('POST /api/registry/claim', () => {
  it('happy path — authenticated admin/owner returns 200', async () => {
    const req = makeRequest({ package_path: 'github.com/testowner/testrepo' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = await res.json() as Record<string, unknown>
    expect(body.claimed).toBe(true)
  })

  it('returns 401 when not authenticated (requireSession returns Response)', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 })
    )
    const req = makeRequest({ package_path: 'github.com/testowner/testrepo' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(401)
  })

  it('returns 401 when no GitHub token is available', async () => {
    vi.mocked(ghToken.getGitHubToken).mockResolvedValue(null)
    const req = makeRequest({ package_path: 'github.com/testowner/testrepo' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(401)
    const body = await res.json() as Record<string, unknown>
    expect(String(body.error)).toMatch(/github/i)
  })

  it('returns 404 when package is not found', async () => {
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      makeRepos({ getPackage: vi.fn().mockResolvedValue(null) }) as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest({ package_path: 'github.com/nobody/notexist' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(404)
    const body = await res.json() as Record<string, unknown>
    expect(String(body.error)).toMatch(/not found/i)
  })

  it('returns 403 when user lacks admin/write permission on repo', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ permission: 'read', role_name: 'read' }),
    }))
    const req = makeRequest({ package_path: 'github.com/testowner/testrepo' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(403)
    const body = await res.json() as Record<string, unknown>
    expect(String(body.error)).toMatch(/admin|maintainer/i)
  })

  it('returns 409 when package is claimed by another user', async () => {
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      makeRepos({
        getPackage: vi.fn().mockResolvedValue({ ...VALID_PACKAGE, claimedBy: 'other-user' }),
      }) as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest({ package_path: 'github.com/testowner/testrepo' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(409)
  })

  it('transitions scope from unofficial to community on claim', async () => {
    const unofficialPkg = { ...VALID_PACKAGE, scope: 'unofficial', claimedBy: null }
    const repos = makeRepos({ getPackage: vi.fn().mockResolvedValue(unofficialPkg) })
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      repos as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest({ package_path: 'github.com/testowner/testrepo' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    expect(repos.upsertPackage).toHaveBeenCalledWith(
      expect.objectContaining({ scope: 'community', claimedBy: 'user-1' })
    )
  })

  it('returns 200 idempotently when already claimed by same user', async () => {
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      makeRepos({
        getPackage: vi.fn().mockResolvedValue({ ...VALID_PACKAGE, claimedBy: 'user-1' }),
      }) as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest({ package_path: 'github.com/testowner/testrepo' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = await res.json() as Record<string, unknown>
    expect(body.claimed).toBe(true)
  })
})
