import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({ env: {} }))

const mockRun = vi.fn().mockResolvedValue({})
const mockFirst = vi.fn()
const mockBind = vi.fn(() => ({ run: mockRun, first: mockFirst }))
const mockPrepare = vi.fn(() => ({ bind: mockBind }))
const mockD1 = { prepare: mockPrepare }

vi.mock('#/lib/d1', () => ({
  getD1: vi.fn(() => mockD1),
  nanoid: vi.fn(() => 'test-id'),
}))

import { Route } from '../cli-callback'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const GET = (Route.options.server!.handlers as any).GET!

function makeRequest(params: Record<string, string> = {}): Request {
  const url = new URL('http://localhost/auth/cli-callback')
  for (const [k, v] of Object.entries(params)) {
    url.searchParams.set(k, v)
  }
  return new Request(url.toString())
}

describe('GET /auth/cli-callback', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockFirst.mockResolvedValue(null)
  })

  it('returns error redirect for expired state and cleans up state', async () => {
    // State was created 11 minutes ago (expired)
    const expiredCreatedAt = Date.now() - 11 * 60 * 1000
    let firstCallDone = false
    mockFirst.mockImplementation(() => {
      if (!firstCallDone) {
        firstCallDone = true
        return Promise.resolve({
          code_challenge: 'test-challenge',
          redirect_uri: 'http://localhost:9999/callback',
          created_at: expiredCreatedAt,
        })
      }
      return Promise.resolve(null)
    })

    const req = makeRequest({ code: 'gh-code', state: 'test-state' })
    const res = await GET({ request: req } as Parameters<typeof GET>[0])

    // Should redirect with state_expired error
    expect(res.status).toBe(302)
    const location = res.headers.get('Location')
    expect(location).toContain('state_expired')

    // Verify DELETE was called (state cleanup)
    const deleteCalls = mockPrepare.mock.calls.filter(
      (call: string[]) => typeof call[0] === 'string' && call[0].includes('DELETE'),
    )
    expect(deleteCalls.length).toBe(1)
  })

  it('returns error redirect for missing params', async () => {
    const req = makeRequest({})
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    expect(res.status).toBe(302)
    const location = res.headers.get('Location')
    expect(location).toContain('missing_params')
  })

  it('returns error redirect for invalid state', async () => {
    mockFirst.mockResolvedValue(null)
    const req = makeRequest({ code: 'gh-code', state: 'bad-state' })
    const res = await GET({ request: req } as Parameters<typeof GET>[0])
    expect(res.status).toBe(302)
    const location = res.headers.get('Location')
    expect(location).toContain('invalid_state')
  })
})
