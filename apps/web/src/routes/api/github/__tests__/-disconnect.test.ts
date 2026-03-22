import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({ env: {} }))

vi.mock('#/lib/session-auth', () => ({
  requireSession: vi.fn(),
}))

vi.mock('#/lib/github-app', () => ({
  clearTokenCookie: vi.fn().mockReturnValue('gh_token=; Max-Age=0; Path=/'),
}))

import { Route } from '../disconnect'
import * as sessionAuth from '#/lib/session-auth'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const POST = (Route.options.server!.handlers as any).POST!

function makeRequest(): Request {
  return new Request('http://localhost/api/github/disconnect', {
    method: 'POST',
  })
}

beforeEach(() => {
  vi.restoreAllMocks()
  vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'user-1', org: 'user-1' })
})

describe('POST /api/github/disconnect', () => {
  it('returns 401 when not authenticated', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 }),
    )
    const res = await POST({ request: makeRequest() } as Parameters<typeof POST>[0])
    expect(res.status).toBe(401)
  })

  it('returns ok and clears cookie when authenticated', async () => {
    const res = await POST({ request: makeRequest() } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.ok).toBe(true)
    expect(res.headers.get('Set-Cookie')).toBeTruthy()
  })
})
