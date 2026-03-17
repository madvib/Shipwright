import { describe, it, expect } from 'vitest'
import {
  buildAuthorizeUrl,
  getTokenFromCookie,
  getStateFromCookie,
  setTokenCookie,
  setStateCookie,
  clearTokenCookie,
} from '#/lib/github-app'

describe('buildAuthorizeUrl', () => {
  it('builds correct GitHub OAuth URL', () => {
    const url = buildAuthorizeUrl('my-client-id', 'https://example.com/callback', 'state123')
    expect(url).toContain('https://github.com/login/oauth/authorize')
    expect(url).toContain('client_id=my-client-id')
    expect(url).toContain('redirect_uri=https%3A%2F%2Fexample.com%2Fcallback')
    expect(url).toContain('state=state123')
    expect(url).toContain('scope=repo')
  })
})

describe('cookie helpers', () => {
  it('getTokenFromCookie extracts token', () => {
    const req = new Request('http://localhost', {
      headers: { Cookie: 'gh_token=abc123; other=val' },
    })
    expect(getTokenFromCookie(req)).toBe('abc123')
  })

  it('getTokenFromCookie returns null when missing', () => {
    const req = new Request('http://localhost', {
      headers: { Cookie: 'other=val' },
    })
    expect(getTokenFromCookie(req)).toBeNull()
  })

  it('getStateFromCookie extracts state', () => {
    const req = new Request('http://localhost', {
      headers: { Cookie: 'gh_oauth_state=xyz789' },
    })
    expect(getStateFromCookie(req)).toBe('xyz789')
  })

  it('setTokenCookie returns HttpOnly Secure cookie string', () => {
    const cookie = setTokenCookie('mytoken')
    expect(cookie).toContain('gh_token=mytoken')
    expect(cookie).toContain('HttpOnly')
    expect(cookie).toContain('Secure')
    expect(cookie).toContain('SameSite=Lax')
  })

  it('setStateCookie returns short-lived cookie', () => {
    const cookie = setStateCookie('state123')
    expect(cookie).toContain('gh_oauth_state=state123')
    expect(cookie).toContain('Max-Age=600')
  })

  it('clearTokenCookie sets Max-Age=0', () => {
    const cookie = clearTokenCookie()
    expect(cookie).toContain('Max-Age=0')
  })
})
