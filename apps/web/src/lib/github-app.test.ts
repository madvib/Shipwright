import { describe, it, expect, vi, beforeEach } from 'vitest'
import { getUser, listRepos } from '#/lib/github-app'

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('getUser', () => {
  it('returns user data on success', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ login: 'testuser', avatar_url: 'https://example.com/avatar.png' }),
    }))

    const user = await getUser('test-token')
    expect(user.login).toBe('testuser')
    expect(user.avatar_url).toBe('https://example.com/avatar.png')
  })

  it('throws on non-ok response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false, status: 401 }))
    await expect(getUser('bad-token')).rejects.toThrow('GitHub API /user failed: 401')
  })
})

describe('listRepos', () => {
  it('returns repos on success', async () => {
    const mockRepos = [
      { full_name: 'user/repo1', name: 'repo1', owner: { login: 'user' }, private: false, default_branch: 'main', description: null },
    ]
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockRepos),
    }))

    const repos = await listRepos('test-token')
    expect(repos).toHaveLength(1)
    expect(repos[0].full_name).toBe('user/repo1')
  })

  it('passes page parameter', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve([]),
    })
    vi.stubGlobal('fetch', mockFetch)

    await listRepos('test-token', 3)
    const calledUrl = mockFetch.mock.calls[0][0] as string
    expect(calledUrl).toContain('page=3')
  })

  it('throws on non-ok response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false, status: 403 }))
    await expect(listRepos('bad-token')).rejects.toThrow('GitHub API /user/repos failed: 403')
  })
})
