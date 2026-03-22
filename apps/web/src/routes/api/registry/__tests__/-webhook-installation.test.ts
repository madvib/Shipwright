import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('cloudflare:workers', () => ({
  env: { GITHUB_WEBHOOK_SECRET: 'test-secret' },
}))

vi.mock('#/db/registry-repositories', () => ({
  createRegistryRepositories: vi.fn(),
}))

const mockRun = vi.fn().mockResolvedValue({})
const mockFirst = vi.fn()
const mockBind = vi.fn(() => ({ run: mockRun, first: mockFirst }))
const mockPrepare = vi.fn(() => ({ bind: mockBind }))
const mockD1 = { prepare: mockPrepare }

vi.mock('#/lib/d1', () => ({
  getRegistryDb: vi.fn(() => mockD1),
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

async function makeRequest(event: string, payload: Record<string, unknown>): Promise<Request> {
  const body = JSON.stringify(payload)
  const signature = await computeHmac('test-secret', body)
  return new Request('http://localhost/api/registry/webhook', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-hub-signature-256': signature,
      'x-github-event': event,
    },
    body,
  })
}

describe('webhook installation handling', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockFirst.mockResolvedValue(null)
  })

  it('stores installation on created event', async () => {
    const payload = {
      action: 'created',
      installation: {
        id: 12345,
        account: { login: 'test-org', type: 'Organization' },
      },
      repositories: [
        { id: 1, full_name: 'test-org/repo-a', private: false },
        { id: 2, full_name: 'test-org/repo-b', private: true },
      ],
    }

    const req = await makeRequest('installation', payload)
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.status).toBe('stored')
    expect(body.installation_id).toBe(12345)
    expect(mockPrepare).toHaveBeenCalledWith(
      expect.stringContaining('INSERT INTO github_installations'),
    )
  })

  it('removes installation on deleted event', async () => {
    const payload = {
      action: 'deleted',
      installation: {
        id: 12345,
        account: { login: 'test-org', type: 'Organization' },
      },
    }

    const req = await makeRequest('installation', payload)
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.status).toBe('removed')
    expect(mockPrepare).toHaveBeenCalledWith(
      expect.stringContaining('DELETE FROM github_installations'),
    )
  })

  it('updates repos on added event', async () => {
    mockFirst.mockResolvedValue({
      repos_json: JSON.stringify([{ id: 1, full_name: 'org/existing', private: false }]),
    })

    const payload = {
      action: 'added',
      installation: {
        id: 12345,
        account: { login: 'test-org', type: 'Organization' },
      },
      repositories_added: [
        { id: 3, full_name: 'test-org/new-repo', private: false },
      ],
    }

    const req = await makeRequest('installation_repositories', payload)
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.status).toBe('updated')
  })

  it('acknowledges unknown installation actions', async () => {
    const payload = {
      action: 'suspend',
      installation: {
        id: 12345,
        account: { login: 'test-org', type: 'User' },
      },
    }

    const req = await makeRequest('installation', payload)
    const res = await POST({ request: req } as Parameters<typeof POST>[0])
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.status).toBe('acknowledged')
  })
})
