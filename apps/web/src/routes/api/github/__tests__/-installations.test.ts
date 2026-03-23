import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({ env: { DB: {} } }))

vi.mock('#/lib/session-auth', () => ({
  requireSession: vi.fn(),
}))

vi.mock('#/lib/d1', () => ({
  getRegistryDb: vi.fn(),
}))

import { Route } from '../installations'
import * as sessionAuth from '#/lib/session-auth'
import * as d1Lib from '#/lib/d1'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const GET = (Route.options.server!.handlers as any).GET!

function makeMockD1() {
  const preparedStatement = {
    bind: vi.fn().mockReturnThis(),
    run: vi.fn().mockResolvedValue({}),
    first: vi.fn().mockResolvedValue(null),
    all: vi.fn().mockResolvedValue({ results: [] }),
  }
  return {
    prepare: vi.fn().mockReturnValue(preparedStatement),
    batch: vi.fn().mockResolvedValue([]),
    exec: vi.fn(),
    dump: vi.fn(),
  }
}

function makeRequest(): Request {
  return new Request('http://localhost/api/github/installations')
}

beforeEach(() => {
  vi.restoreAllMocks()
  vi.mocked(d1Lib.getRegistryDb).mockReturnValue(makeMockD1() as unknown as D1Database)
  vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'user-1', org: 'user-1' })
})

describe('GET /api/github/installations', () => {
  it('returns 401 when not authenticated', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 }),
    )
    const res = await GET({ request: makeRequest() } as Parameters<typeof GET>[0])
    expect(res.status).toBe(401)
  })

  it('returns installations when authenticated', async () => {
    const mockD1 = makeMockD1()
    vi.mocked(d1Lib.getRegistryDb).mockReturnValue(mockD1 as unknown as D1Database)

    const res = await GET({ request: makeRequest() } as Parameters<typeof GET>[0])
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.installations).toBeDefined()
  })

  it('returns 503 when database is unavailable', async () => {
    vi.mocked(d1Lib.getRegistryDb).mockReturnValue(null)
    const res = await GET({ request: makeRequest() } as Parameters<typeof GET>[0])
    expect(res.status).toBe(503)
  })
})
