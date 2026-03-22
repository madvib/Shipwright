import { describe, it, expect } from 'vitest'
import { verifySignature, getPayloadAgeMs } from '../webhook-verify'

async function computeHmac(secret: string, body: string): Promise<string> {
  const encoder = new TextEncoder()
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign'],
  )
  const sig = await crypto.subtle.sign('HMAC', key, encoder.encode(body))
  return (
    'sha256=' +
    Array.from(new Uint8Array(sig), (b) =>
      b.toString(16).padStart(2, '0'),
    ).join('')
  )
}

describe('verifySignature', () => {
  it('returns true for a valid signature', async () => {
    const secret = 'my-secret'
    const body = '{"hello":"world"}'
    const signature = await computeHmac(secret, body)
    expect(await verifySignature(secret, body, signature)).toBe(true)
  })

  it('returns false for an invalid signature', async () => {
    const secret = 'my-secret'
    const body = '{"hello":"world"}'
    const badSig = 'sha256=0000000000000000000000000000000000000000000000000000000000000000'
    expect(await verifySignature(secret, body, badSig)).toBe(false)
  })

  it('returns false for wrong-length signature', async () => {
    expect(await verifySignature('s', 'b', 'sha256=short')).toBe(false)
  })
})

describe('getPayloadAgeMs', () => {
  it('returns age from repository.pushed_at', () => {
    const fiveMinutesAgo = Math.floor(Date.now() / 1000) - 300
    const age = getPayloadAgeMs({ repository: { pushed_at: fiveMinutesAgo } })
    expect(age).toBeGreaterThanOrEqual(299_000)
    expect(age).toBeLessThan(310_000)
  })

  it('returns age from installation.updated_at', () => {
    const twoMinutesAgo = new Date(Date.now() - 120_000).toISOString()
    const age = getPayloadAgeMs({ installation: { updated_at: twoMinutesAgo } })
    expect(age).toBeGreaterThanOrEqual(119_000)
    expect(age).toBeLessThan(130_000)
  })

  it('returns null when no timestamp is found', () => {
    expect(getPayloadAgeMs({ action: 'created' })).toBeNull()
  })

  it('returns age from top-level created_at', () => {
    const oneMinuteAgo = new Date(Date.now() - 60_000).toISOString()
    const age = getPayloadAgeMs({ created_at: oneMinuteAgo })
    expect(age).toBeGreaterThanOrEqual(59_000)
    expect(age).toBeLessThan(70_000)
  })
})
