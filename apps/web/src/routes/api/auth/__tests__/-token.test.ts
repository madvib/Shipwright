import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({ env: {} }))

vi.mock('#/lib/cloud-auth', () => ({
  signJwt: vi.fn().mockResolvedValue('signed-jwt-token'),
  getSecret: vi.fn().mockReturnValue('test-secret-key'),
}))

vi.mock('#/lib/rate-limit', () => ({
  checkRateLimit: vi.fn().mockResolvedValue({ allowed: true, retryAfter: 0 }),
  rateLimitResponse: vi.fn((retryAfter: number) =>
    Response.json(
      { error: 'Rate limit exceeded', retryAfter },
      { status: 429, headers: { 'Retry-After': String(retryAfter) } },
    ),
  ),
}))

const mockRun = vi.fn().mockResolvedValue({ meta: { changes: 1 } })
const mockFirst = vi.fn()
const mockBind = vi.fn(() => ({ run: mockRun, first: mockFirst }))
const mockPrepare = vi.fn(() => ({ bind: mockBind }))
const mockD1 = { prepare: mockPrepare }

vi.mock('#/lib/d1', () => ({
  getD1: vi.fn(() => mockD1),
}))

import { Route } from '../token'
import * as rateLimit from '#/lib/rate-limit'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const POST = (Route.options.server!.handlers as any).POST!

function makeRequest(body: unknown): Request {
  return new Request('http://localhost/api/auth/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
}

describe('POST /api/auth/token', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(rateLimit.checkRateLimit).mockResolvedValue({ allowed: true, retryAfter: 0 })
    vi.mocked(rateLimit.rateLimitResponse).mockImplementation((retryAfter: number) =>
      Response.json(
        { error: 'Rate limit exceeded', retryAfter },
        { status: 429, headers: { 'Retry-After': String(retryAfter) } },
      ),
    )
    mockRun.mockResolvedValue({ meta: { changes: 1 } })
    mockFirst.mockResolvedValue({
      user_id: 'user-1',
      org_id: 'org-1',
      code_challenge: 'challenge',
      created_at: Date.now(),
    })
  })

  it('returns 401 when code was already consumed (race condition prevention)', async () => {
    // Simulate: UPDATE ... WHERE used = 0 affects 0 rows (already used)
    mockRun.mockResolvedValue({ meta: { changes: 0 } })

    const req = makeRequest({ code: 'used-code', verifier: 'test-verifier' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(401)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.error).toMatch(/invalid or already-used/i)
  })

  it('returns 401 when code does not exist', async () => {
    // UPDATE returns 0 changes because code does not exist
    mockRun.mockResolvedValue({ meta: { changes: 0 } })

    const req = makeRequest({ code: 'nonexistent', verifier: 'test-verifier' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(401)
  })

  it('returns 429 when rate limited', async () => {
    vi.mocked(rateLimit.checkRateLimit).mockResolvedValue({ allowed: false, retryAfter: 60 })

    const req = makeRequest({ code: 'test-code', verifier: 'test-verifier' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(429)
  })

  it('returns 400 for missing code or verifier', async () => {
    const req = makeRequest({ code: 'test-code' })
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(400)
  })
})
