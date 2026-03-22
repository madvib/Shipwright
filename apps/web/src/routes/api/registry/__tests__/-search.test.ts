import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({ env: { AUTH_DB: {}, REGISTRY_DB: {} } }))

vi.mock('#/db/registry-repositories', () => ({
  createRegistryRepositories: vi.fn(),
}))

vi.mock('#/lib/d1', () => ({
  getRegistryDb: vi.fn(),
  nanoid: vi.fn(() => 'test-id'),
}))

vi.mock('#/lib/rate-limit', () => ({
  checkRateLimit: vi.fn().mockResolvedValue({ allowed: true, retryAfter: 0 }),
  rateLimitResponse: vi.fn(() => Response.json(
    { error: 'Rate limit exceeded', retryAfter: 60 },
    { status: 429 },
  )),
}))

import { Route } from '../search'
import * as registryRepositories from '#/db/registry-repositories'
import * as d1Lib from '#/lib/d1'
import * as rateLimitLib from '#/lib/rate-limit'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const GET = (Route.options.server!.handlers as any).GET!

function makePackage(overrides: Record<string, unknown> = {}) {
  return {
    id: 'pkg-1',
    path: 'github.com/owner/repo',
    name: 'my-package',
    description: 'A cool package',
    scope: 'community',
    latestVersion: '1.0.0',
    installs: 42,
    stars: 5,
    deprecatedBy: null,
    defaultBranch: 'main',
    contentHash: null,
    sourceType: 'native',
    claimedBy: null,
    repoUrl: 'https://github.com/owner/repo',
    indexedAt: Date.now(),
    updatedAt: Date.now(),
    ...overrides,
  }
}

function makeRepos(overrides: Record<string, unknown> = {}) {
  return {
    searchPackages: vi.fn().mockResolvedValue({
      packages: [makePackage()],
      total: 1,
      page: 1,
    }),
    getPackage: vi.fn(),
    upsertPackage: vi.fn(),
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

function makeRequest(params: Record<string, string> = {}): Request {
  const url = new URL('http://localhost/api/registry/search')
  for (const [k, v] of Object.entries(params)) {
    url.searchParams.set(k, v)
  }
  return new Request(url.toString(), { method: 'GET' })
}

beforeEach(() => {
  vi.mocked(d1Lib.getRegistryDb).mockReturnValue({} as D1Database)
  vi.mocked(rateLimitLib.checkRateLimit).mockResolvedValue({ allowed: true, retryAfter: 0 })
  vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
    makeRepos() as ReturnType<typeof registryRepositories.createRegistryRepositories>
  )
})

describe('GET /api/registry/search', () => {
  it('returns matching packages for a query', async () => {
    const req = makeRequest({ q: 'my-package' })
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    expect(res.status).toBe(200)
    const body = await res.json() as Record<string, unknown>
    const packages = body.packages as unknown[]
    expect(Array.isArray(packages)).toBe(true)
    expect(packages.length).toBeGreaterThan(0)
    expect(body.total).toBe(1)
  })

  it('returns empty results when no packages match', async () => {
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      makeRepos({
        searchPackages: vi.fn().mockResolvedValue({ packages: [], total: 0, page: 1 }),
      }) as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest({ q: 'nonexistent-thing' })
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    expect(res.status).toBe(200)
    const body = await res.json() as Record<string, unknown>
    expect(body.packages).toEqual([])
    expect(body.total).toBe(0)
  })

  it('passes correct page and limit for pagination', async () => {
    const repos = makeRepos()
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      repos as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest({ page: '2', limit: '12' })
    await GET({ request: req } as Parameters<typeof GET>[0])

    const searchCall = vi.mocked(repos.searchPackages).mock.calls[0]
    expect(searchCall?.[2]).toBe(2) // page
    expect(searchCall?.[3]).toBe(12) // limit
  })

  it('scope filter is forwarded to repository', async () => {
    const repos = makeRepos()
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      repos as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest({ scope: 'official' })
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    expect(res.status).toBe(200)
    const searchCall = vi.mocked(repos.searchPackages).mock.calls[0]
    expect(searchCall?.[1]).toBe('official')
  })

  it('returns 400 for invalid scope value', async () => {
    const req = makeRequest({ scope: 'invalid-scope' })
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    expect(res.status).toBe(400)
    const body = await res.json() as Record<string, unknown>
    expect(String(body.error)).toMatch(/scope/i)
  })

  it('returns 503 when database is unavailable', async () => {
    vi.mocked(d1Lib.getRegistryDb).mockReturnValue(null)
    const req = makeRequest({ q: 'test' })
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    expect(res.status).toBe(503)
  })

  it('response packages include expected fields', async () => {
    const req = makeRequest()
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    const body = await res.json() as Record<string, unknown>
    const packages = body.packages as Record<string, unknown>[]
    const pkg = packages[0]
    expect(pkg).toHaveProperty('id')
    expect(pkg).toHaveProperty('path')
    expect(pkg).toHaveProperty('name')
    expect(pkg).toHaveProperty('scope')
    expect(pkg).toHaveProperty('installs')
    expect(pkg).toHaveProperty('latestVersion')
    expect(pkg).toHaveProperty('updatedAt')
  })

  it('defaults page to 1 and limit to 20 when not provided', async () => {
    const repos = makeRepos()
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      repos as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest()
    await GET({ request: req } as Parameters<typeof GET>[0])
    const searchCall = vi.mocked(repos.searchPackages).mock.calls[0]
    expect(searchCall?.[2]).toBe(1)
    expect(searchCall?.[3]).toBe(20)
  })

  it('returns 429 when rate limited', async () => {
    vi.mocked(rateLimitLib.checkRateLimit).mockResolvedValue({ allowed: false, retryAfter: 60 })
    const req = makeRequest({ q: 'test' })
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    expect(res.status).toBe(429)
  })

  it('sort param is forwarded to repository', async () => {
    const repos = makeRepos()
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      repos as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest({ sort: 'recent' })
    await GET({ request: req } as Parameters<typeof GET>[0])
    const searchCall = vi.mocked(repos.searchPackages).mock.calls[0]
    expect(searchCall?.[4]).toBe('recent')
  })

  it('returns 400 for invalid sort value', async () => {
    const req = makeRequest({ sort: 'bogus' })
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    expect(res.status).toBe(400)
    const body = await res.json() as Record<string, unknown>
    expect(String(body.error)).toMatch(/sort/i)
  })

  it('defaults sort to installs when not provided', async () => {
    const repos = makeRepos()
    vi.mocked(registryRepositories.createRegistryRepositories).mockReturnValue(
      repos as ReturnType<typeof registryRepositories.createRegistryRepositories>
    )
    const req = makeRequest()
    await GET({ request: req } as Parameters<typeof GET>[0])
    const searchCall = vi.mocked(repos.searchPackages).mock.calls[0]
    expect(searchCall?.[4]).toBe('installs')
  })
})
