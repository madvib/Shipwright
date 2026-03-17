import { describe, it, expect } from 'vitest'
import { signJwt, verifyJwt } from './cloud-auth'

const SECRET = 'test-secret-value-at-least-32-chars-long'

describe('signJwt / verifyJwt', () => {
  it('round-trips a payload', async () => {
    const token = await signJwt({ sub: 'user-1', org: 'org-1' }, SECRET)
    expect(typeof token).toBe('string')
    expect(token.split('.').length).toBe(3)

    const payload = await verifyJwt(token, SECRET)
    expect(payload).not.toBeNull()
    expect(payload?.sub).toBe('user-1')
    expect(payload?.org).toBe('org-1')
  })

  it('rejects a token signed with a different secret', async () => {
    const token = await signJwt({ sub: 'user-1', org: 'org-1' }, SECRET)
    const payload = await verifyJwt(token, 'wrong-secret-value-at-least-32-chars')
    expect(payload).toBeNull()
  })

  it('rejects a tampered payload', async () => {
    const token = await signJwt({ sub: 'user-1', org: 'org-1' }, SECRET)
    const [header, _body, sig] = token.split('.')
    const fakeBody = btoa(JSON.stringify({ sub: 'attacker', org: 'org-1', iat: 0, exp: 9999999999 }))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '')
    const tampered = `${header}.${fakeBody}.${sig}`
    const payload = await verifyJwt(tampered, SECRET)
    expect(payload).toBeNull()
  })

  it('rejects a malformed token', async () => {
    expect(await verifyJwt('not.a.token.with.five.parts', SECRET)).toBeNull()
    expect(await verifyJwt('', SECRET)).toBeNull()
    expect(await verifyJwt('only-one-part', SECRET)).toBeNull()
  })
})
