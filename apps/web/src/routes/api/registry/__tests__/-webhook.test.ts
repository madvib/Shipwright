import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({
  env: { GITHUB_WEBHOOK_SECRET: 'test-secret' },
}))

vi.mock('#/db/registry-repositories', () => ({
  createRegistryRepositories: vi.fn(),
}))

vi.mock('#/lib/d1', () => ({
  getD1: vi.fn(),
  nanoid: vi.fn(() => 'test-id'),
}))

vi.mock('#/lib/registry-github', () => ({
  fetchFileFromGitHub: vi.fn(),
  parseShipToml: vi.fn(),
}))

import { Route } from '../webhook'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const POST = (Route.options.server!.handlers as any).POST!

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

function makeWebhookRequest(
  event: string,
  payload: Record<string, unknown>,
  options: { validSignature?: boolean } = {},
) {
  const body = JSON.stringify(payload)
  const { validSignature = true } = options

  // We compute the real HMAC in the test; for invalid signature tests we use a dummy
  return {
    body,
    event,
    validSignature,
    async toRequest(): Promise<Request> {
      const signature = validSignature
        ? await computeHmac('test-secret', body)
        : 'sha256=0000000000000000000000000000000000000000000000000000000000000000'
      return new Request('http://localhost/api/registry/webhook', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-hub-signature-256': signature,
          'x-github-event': event,
        },
        body,
      })
    },
  }
}

describe('POST /api/registry/webhook', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('rejects requests with invalid signature', async () => {
    const helper = makeWebhookRequest('ping', {}, { validSignature: false })
    const req = await helper.toRequest()
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(401)
  })

  it('accepts valid signature and ignores unknown events', async () => {
    const helper = makeWebhookRequest('ping', {})
    const req = await helper.toRequest()
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = await res.json() as Record<string, unknown>
    expect(body.status).toBe('ignored')
  })

  it('rejects webhook payloads older than 5 minutes', async () => {
    const tenMinutesAgo = Math.floor(Date.now() / 1000) - 600
    const payload = {
      ref_type: 'tag',
      ref: 'v1.0.0',
      repository: {
        full_name: 'owner/repo',
        default_branch: 'main',
        pushed_at: tenMinutesAgo,
      },
    }
    const helper = makeWebhookRequest('create', payload)
    const req = await helper.toRequest()
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(400)
    const body = await res.json() as Record<string, unknown>
    expect(String(body.error)).toMatch(/replay/i)
  })

  it('accepts webhook payloads within 5-minute window', async () => {
    const oneMinuteAgo = Math.floor(Date.now() / 1000) - 60
    const payload = {
      ref_type: 'tag',
      ref: 'v1.0.0',
      repository: {
        full_name: 'owner/repo',
        default_branch: 'main',
        pushed_at: oneMinuteAgo,
      },
    }

    // Mock the GitHub file fetch to return null (skipped, no toml)
    const registryGithub = await import('#/lib/registry-github')
    vi.mocked(registryGithub.fetchFileFromGitHub).mockResolvedValue(null)

    const helper = makeWebhookRequest('create', payload)
    const req = await helper.toRequest()
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    // Should proceed past timestamp check (200 with skipped status)
    expect(res.status).toBe(200)
    const body = await res.json() as Record<string, unknown>
    expect(body.status).toBe('skipped')
  })

  it('allows payloads without a parseable timestamp (no false rejections)', async () => {
    const payload = {
      action: 'created',
    }
    const helper = makeWebhookRequest('installation', payload)
    const req = await helper.toRequest()
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = await res.json() as Record<string, unknown>
    expect(body.status).toBe('acknowledged')
  })

  it('rejects installation events with stale updated_at', async () => {
    const tenMinutesAgo = new Date(Date.now() - 600_000).toISOString()
    const payload = {
      action: 'created',
      installation: {
        id: 12345,
        updated_at: tenMinutesAgo,
      },
    }
    const helper = makeWebhookRequest('installation', payload)
    const req = await helper.toRequest()
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(400)
    const body = await res.json() as Record<string, unknown>
    expect(String(body.error)).toMatch(/replay/i)
  })
})
