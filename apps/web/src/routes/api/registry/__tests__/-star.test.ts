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

import { Route } from '../$path.star'
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
  stars: 5,
  installs: 10,
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
    incrementStars: vi.fn().mockResolvedValue(6),
    upsertPackage: vi.fn(),
    searchPackages: vi.fn(),
    getLatestVersion: vi.fn().mockResolvedValue(null),
    getPackageVersions: vi.fn(),
    getPackageSkills: vi.fn(),
    createPackageVersion: vi.fn(),
    createPackageSkill: vi.fn(),
    incrementInstalls: vi.fn(),
    deletePackageSkillsByVersion: vi.fn(),
    deprecatePackage: vi.fn(),
    ...overrides,
  }
}

function makeRequest(): Request {
  return new Request('http://localhost/api/registry/github.com%2Ftestowner%2Ftestrepo/star', {
    method: 'POST',
  })
}

beforeEach(() => {
  vi.mocked(d1Lib.getD1).mockReturnValue({} as D1Database)
  vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'user-1', org: 'user-1' })
  vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
    makeRepos() as ReturnType<typeof registryRepositories.createRegistryRepositories>,
  )
})

describe('POST /api/registry/:path/star', () => {
  it('increments stars and returns new count', async () => {
    const req = makeRequest()
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Ftestowner%2Ftestrepo' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.stars).toBe(6)
    expect(body.starred).toBe(true)
  })

  it('returns 401 when not authenticated', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 }),
    )
    const req = makeRequest()
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Ftestowner%2Ftestrepo' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(401)
  })

  it('returns 404 when package not found', async () => {
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      makeRepos({ getPackage: vi.fn().mockResolvedValue(null) }) as ReturnType<
        typeof registryRepositories.createRegistryRepositories
      >,
    )
    const req = makeRequest()
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Fnobody%2Fnotexist' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(404)
  })

  it('returns 503 when database is unavailable', async () => {
    vi.mocked(d1Lib.getD1).mockReturnValue(null)
    const req = makeRequest()
    const res = await POST({
      request: req,
      params: { path: 'github.com%2Ftestowner%2Ftestrepo' },
    } as Parameters<typeof POST>[0])
    expect(res.status).toBe(503)
  })
})
