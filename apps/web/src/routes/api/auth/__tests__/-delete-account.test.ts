import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({ env: { DB: {} } }))

vi.mock('#/lib/session-auth', () => ({
  requireSession: vi.fn(),
}))

vi.mock('#/lib/d1', () => ({
  getD1: vi.fn(),
}))

vi.mock('#/lib/rate-limit', () => ({
  checkRateLimit: vi.fn().mockResolvedValue({ allowed: true, retryAfter: 0 }),
  rateLimitResponse: vi.fn(),
}))

import { Route } from '../delete-account'
import * as sessionAuth from '#/lib/session-auth'
import * as d1Lib from '#/lib/d1'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const POST = (Route.options.server!.handlers as any).POST!

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
  return new Request('http://localhost/api/auth/delete-account', {
    method: 'POST',
  })
}

beforeEach(() => {
  vi.mocked(d1Lib.getD1).mockReturnValue(makeMockD1() as unknown as D1Database)
  vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'user-1', org: 'user-1' })
})

describe('POST /api/auth/delete-account', () => {
  it('deletes user data and returns ok', async () => {
    const mockD1 = makeMockD1()
    vi.mocked(d1Lib.getD1).mockReturnValue(mockD1 as unknown as D1Database)

    const req = makeRequest()
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.ok).toBe(true)

    // Verify batch was called with 6 delete statements
    expect(mockD1.batch).toHaveBeenCalledTimes(1)
    const batchArgs = mockD1.batch.mock.calls[0][0]
    expect(batchArgs).toHaveLength(6)
  })

  it('returns 401 when not authenticated', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 }),
    )
    const req = makeRequest()
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(401)
  })

  it('returns 503 when database is unavailable', async () => {
    vi.mocked(d1Lib.getD1).mockReturnValue(null)
    const req = makeRequest()
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(503)
  })
})
