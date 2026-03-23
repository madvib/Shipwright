import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('#/lib/auth', () => ({
  getAuth: vi.fn(),
}))

import { getAuth } from '#/lib/auth'
import { getGitHubToken } from '#/lib/github-token'

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('getGitHubToken', () => {
  it('returns accessToken when user has a linked GitHub account', async () => {
    vi.mocked(getAuth).mockResolvedValue({
      api: {
        getAccessToken: vi.fn().mockResolvedValue({ accessToken: 'ghp_abc123' }),
      },
    } as never)

    const request = new Request('http://localhost', {
      headers: { Cookie: 'better_auth_session=xyz' },
    })
    const token = await getGitHubToken(request)
    expect(token).toBe('ghp_abc123')
  })

  it('returns null when getAccessToken returns no accessToken', async () => {
    vi.mocked(getAuth).mockResolvedValue({
      api: {
        getAccessToken: vi.fn().mockResolvedValue({}),
      },
    } as never)

    const request = new Request('http://localhost')
    const token = await getGitHubToken(request)
    expect(token).toBeNull()
  })

  it('returns null when getAccessToken throws', async () => {
    vi.mocked(getAuth).mockResolvedValue({
      api: {
        getAccessToken: vi.fn().mockRejectedValue(new Error('No session')),
      },
    } as never)

    const request = new Request('http://localhost')
    const token = await getGitHubToken(request)
    expect(token).toBeNull()
  })

  it('returns null when getAuth throws', async () => {
    vi.mocked(getAuth).mockRejectedValue(new Error('DB unavailable'))

    const request = new Request('http://localhost')
    const token = await getGitHubToken(request)
    expect(token).toBeNull()
  })

  it('passes request headers to getAccessToken', async () => {
    const mockGetAccessToken = vi.fn().mockResolvedValue({ accessToken: 'ghp_xyz' })
    vi.mocked(getAuth).mockResolvedValue({
      api: { getAccessToken: mockGetAccessToken },
    } as never)

    const request = new Request('http://localhost', {
      headers: { Cookie: 'session=abc123' },
    })
    await getGitHubToken(request)

    expect(mockGetAccessToken).toHaveBeenCalledWith({
      body: { providerId: 'github' },
      headers: request.headers,
    })
  })
})
