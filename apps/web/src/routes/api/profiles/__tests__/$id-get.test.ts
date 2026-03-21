import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({ env: { DB: {} } }))

vi.mock('#/lib/session-auth', () => ({
  requireSession: vi.fn(),
}))

vi.mock('#/db/repositories', () => ({
  createRepositories: vi.fn(),
}))

vi.mock('#/lib/d1', () => ({
  getD1: vi.fn(),
}))

import { Route } from '../$id'
import * as sessionAuth from '#/lib/session-auth'
import * as repositories from '#/db/repositories'
import * as d1Lib from '#/lib/d1'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const GET = (Route.options.server!.handlers as any).GET!

function makeProfile(overrides: Record<string, unknown> = {}) {
  return {
    id: 'prof-1',
    orgId: 'org-1',
    userId: 'user-1',
    name: 'Test Profile',
    content: 'profile content',
    provider: null,
    createdAt: Date.now(),
    updatedAt: Date.now(),
    ...overrides,
  }
}

function makeRepos(overrides: Record<string, unknown> = {}) {
  return {
    getProfile: vi.fn(),
    getProfiles: vi.fn(),
    upsertProfile: vi.fn(),
    deleteProfile: vi.fn(),
    getLibraries: vi.fn(),
    getLibrary: vi.fn(),
    upsertLibrary: vi.fn(),
    deleteLibrary: vi.fn(),
    getWorkflows: vi.fn(),
    getWorkflow: vi.fn(),
    upsertWorkflow: vi.fn(),
    deleteWorkflow: vi.fn(),
    ...overrides,
  }
}

function makeRequest(): Request {
  return new Request('http://localhost/api/profiles/prof-1', { method: 'GET' })
}

const AUTH = { sub: 'user-1', org: 'org-1' }

beforeEach(() => {
  vi.mocked(sessionAuth.requireSession).mockResolvedValue(AUTH)
  vi.mocked(d1Lib.getD1).mockReturnValue({} as D1Database)
  vi.mocked(repositories.createRepositories).mockReturnValue(
    makeRepos({
      getProfile: vi.fn().mockResolvedValue(makeProfile()),
    }) as ReturnType<typeof repositories.createRepositories>,
  )
})

describe('GET /api/profiles/:id', () => {
  it('returns the profile when found', async () => {
    const req = makeRequest()
    const res = await GET({ request: req, params: { id: 'prof-1' } })
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    const profile = body.profile as Record<string, unknown>
    expect(profile.id).toBe('prof-1')
    expect(profile.name).toBe('Test Profile')
  })

  it('returns 404 when profile is not found', async () => {
    vi.mocked(repositories.createRepositories).mockReturnValue(
      makeRepos({
        getProfile: vi.fn().mockResolvedValue(null),
      }) as ReturnType<typeof repositories.createRepositories>,
    )
    const req = makeRequest()
    const res = await GET({ request: req, params: { id: 'nonexistent' } })
    expect(res.status).toBe(404)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.error).toBe('Profile not found')
  })

  it('returns 401 when not authenticated', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 }),
    )
    const req = makeRequest()
    const res = await GET({ request: req, params: { id: 'prof-1' } })
    expect(res.status).toBe(401)
  })

  it('returns 503 when database is unavailable', async () => {
    vi.mocked(d1Lib.getD1).mockReturnValue(null)
    const req = makeRequest()
    const res = await GET({ request: req, params: { id: 'prof-1' } })
    expect(res.status).toBe(503)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.error).toBe('Database unavailable')
  })

  it('returns 403 when profile belongs to a different org', async () => {
    vi.mocked(repositories.createRepositories).mockReturnValue(
      makeRepos({
        getProfile: vi.fn().mockResolvedValue(makeProfile({ orgId: 'other-org' })),
      }) as ReturnType<typeof repositories.createRepositories>,
    )
    const req = makeRequest()
    const res = await GET({ request: req, params: { id: 'prof-1' } })
    expect(res.status).toBe(403)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.error).toBe('forbidden')
  })

  it('passes correct id and org to repository', async () => {
    const repos = makeRepos({
      getProfile: vi.fn().mockResolvedValue(makeProfile()),
    })
    vi.mocked(repositories.createRepositories).mockReturnValue(
      repos as ReturnType<typeof repositories.createRepositories>,
    )
    const req = makeRequest()
    await GET({ request: req, params: { id: 'prof-1' } })
    expect(repos.getProfile).toHaveBeenCalledWith('prof-1', 'org-1')
  })
})
