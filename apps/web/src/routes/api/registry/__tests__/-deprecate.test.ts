import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({ env: { DB: {} } }))

vi.mock('#/lib/session-auth', () => ({
  requireSession: vi.fn(),
}))

vi.mock('#/db/registry-repositories', () => ({
  createRegistryRepositories: vi.fn(),
}))

vi.mock('#/lib/d1', () => ({
  getD1: vi.fn(),
}))

vi.mock('#/lib/rate-limit', () => ({
  checkRateLimit: vi.fn().mockResolvedValue({ allowed: true, retryAfter: 0 }),
  rateLimitResponse: vi.fn(),
}))

import { Route } from '../$path.deprecate'
import * as sessionAuth from '#/lib/session-auth'
import * as registryRepositories from '#/db/registry-repositories'
import * as d1Lib from '#/lib/d1'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const POST = (Route.options.server!.handlers as any).POST!

const VALID_PACKAGE = {
  id: 'pkg-1',
  path: 'github.com/testowner/testrepo',
  repoUrl: 'https://github.com/testowner/testrepo',
  claimedBy: 'user-1',
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
    deprecatePackage: vi.fn().mockResolvedValue(undefined),
    incrementStars: vi.fn(),
    upsertPackage: vi.fn(),
    searchPackages: vi.fn(),
    getLatestVersion: vi.fn().mockResolvedValue(null),
    getPackageVersions: vi.fn(),
    getPackageSkills: vi.fn(),
    createPackageVersion: vi.fn(),
    createPackageSkill: vi.fn(),
    incrementInstalls: vi.fn(),
    deletePackageSkillsByVersion: vi.fn(),
    ...overrides,
  }
}

function makeRequest(body: unknown): Request {
  return new Request('http://localhost/api/registry/github.com%2Ftestowner%2Ftestrepo/deprecate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
}

beforeEach(() => {
  vi.mocked(d1Lib.getD1).mockReturnValue({} as D1Database)
  vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'user-1', org: 'user-1' })
  vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
    makeRepos() as ReturnType<typeof registryRepositories.createRegistryRepositories>,
  )
})

describe('POST /api/registry/:path/deprecate', () => {
  it('deprecates a package owned by the authenticated user', async () => {
    const repos = makeRepos()
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      repos as ReturnType<typeof registryRepositories.createRegistryRepositories>,
    )
    const req = makeRequest({ deprecated_by: 'Use github.com/better/package instead' })
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Ftestowner%2Ftestrepo' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.ok).toBe(true)
    expect(body.deprecated_by).toBe('Use github.com/better/package instead')
    expect(repos.deprecatePackage).toHaveBeenCalledWith(
      'pkg-1',
      'Use github.com/better/package instead',
    )
  })

  it('returns 401 when not authenticated', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 }),
    )
    const req = makeRequest({ deprecated_by: 'replacement' })
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Ftestowner%2Ftestrepo' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(401)
  })

  it('returns 403 when user is not the package owner', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'other-user', org: 'other-user' })
    const req = makeRequest({ deprecated_by: 'replacement' })
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Ftestowner%2Ftestrepo' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(403)
    const body = (await res.json()) as Record<string, unknown>
    expect(String(body.error)).toMatch(/owner/i)
  })

  it('returns 404 when package not found', async () => {
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      makeRepos({ getPackage: vi.fn().mockResolvedValue(null) }) as ReturnType<
        typeof registryRepositories.createRegistryRepositories
      >,
    )
    const req = makeRequest({ deprecated_by: 'replacement' })
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Fnobody%2Fnotexist' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(404)
  })

  it('returns 400 when deprecated_by field is missing', async () => {
    const req = makeRequest({})
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Ftestowner%2Ftestrepo' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(400)
    const body = (await res.json()) as Record<string, unknown>
    expect(String(body.error)).toMatch(/deprecated_by/i)
  })

  it('returns 400 when body is not valid JSON', async () => {
    const req = new Request('http://localhost/api/registry/test/deprecate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: 'not json',
    })
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Ftestowner%2Ftestrepo' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(400)
  })

  it('returns 503 when database is unavailable', async () => {
    vi.mocked(d1Lib.getD1).mockReturnValue(null)
    const req = makeRequest({ deprecated_by: 'replacement' })
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Ftestowner%2Ftestrepo' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(503)
  })
})
