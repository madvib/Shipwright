import { describe, it, expect, vi, beforeEach } from 'vitest'
import { McpClient, McpClientError } from '../mcp-client'

function makeReadableBody(text: string) {
  const encoder = new TextEncoder()
  return new ReadableStream({
    start(controller) {
      controller.enqueue(encoder.encode(text))
      controller.close()
    },
  })
}

/** Stream that delivers text in multiple chunks of a given size. */
function makeChunkedBody(text: string, chunkSize: number) {
  const encoder = new TextEncoder()
  const bytes = encoder.encode(text)
  let offset = 0
  return new ReadableStream({
    pull(controller) {
      if (offset >= bytes.length) {
        controller.close()
        return
      }
      const end = Math.min(offset + chunkSize, bytes.length)
      controller.enqueue(bytes.slice(offset, end))
      offset = end
    },
  })
}

function initResponse(headers?: Record<string, string>) {
  return {
    ok: true, status: 200, statusText: 'OK',
    headers: new Headers({ 'Mcp-Session-Id': 'sess-1', ...headers }),
    json: async () => ({ jsonrpc: '2.0', result: { serverInfo: { name: 'ship', version: '0.1.0' } }, id: 1 }),
    text: async () => '',
    body: makeReadableBody(JSON.stringify({
      jsonrpc: '2.0', result: { serverInfo: { name: 'ship', version: '0.1.0' } }, id: 1,
    })),
  } as unknown as Response
}

function notifAck() {
  return {
    ok: true, status: 200, statusText: 'OK',
    headers: new Headers(),
    json: async () => ({}),
    text: async () => '',
  } as unknown as Response
}

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('McpClient SSE large payload', () => {
  it('reassembles a 200KB+ JSON-RPC response split across SSE chunks', async () => {
    // Build a payload larger than 200KB
    const largeField = 'x'.repeat(220_000)
    const rpcResult = { serverInfo: { name: 'ship', version: '0.1.0', extra: largeField } }
    const jsonPayload = JSON.stringify({ jsonrpc: '2.0', result: rpcResult, id: 1 })
    const sseText = `data: \n\ndata: ${jsonPayload}\n\n`

    // Deliver in 1KB chunks to simulate network fragmentation
    const fetchMock = vi.fn()
      .mockResolvedValueOnce({
        ok: true, status: 200, statusText: 'OK',
        headers: new Headers({ 'Content-Type': 'text/event-stream', 'Mcp-Session-Id': 'large-sess' }),
        body: makeChunkedBody(sseText, 1024),
      })
      .mockResolvedValueOnce(notifAck())
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    const result = await client.initialize()
    expect(result.serverInfo.name).toBe('ship')
    expect(client.getSessionId()).toBe('large-sess')
  })
})

describe('McpClient session recovery', () => {
  it('detects stale session when server returns 404 and throws McpClientError', async () => {
    // First: successful init
    const staleResponse = {
      ok: false, status: 404, statusText: 'Not Found',
      headers: new Headers(),
      json: async () => ({ error: 'session not found' }),
      text: async () => '{"error":"session not found"}',
      body: makeReadableBody(''),
    }
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(initResponse())
      .mockResolvedValueOnce(notifAck())
      .mockResolvedValueOnce(staleResponse)
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await client.initialize()
    expect(client.getSessionId()).toBe('sess-1')

    // The call should throw McpClientError with the 404 status code
    try {
      await client.listTools()
      expect.unreachable('should have thrown')
    } catch (err) {
      expect(err).toBeInstanceOf(McpClientError)
      expect((err as McpClientError).code).toBe(404)
      expect((err as McpClientError).message).toContain('404')
    }
  })

  it('replaces session ID when server issues a new one mid-session', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(initResponse())
      .mockResolvedValueOnce(notifAck())
      // tools/list returns new session ID
      .mockResolvedValueOnce({
        ok: true, status: 200, statusText: 'OK',
        headers: new Headers({ 'Mcp-Session-Id': 'sess-2' }),
        json: async () => ({ jsonrpc: '2.0', result: { tools: [] }, id: 2 }),
        text: async () => '',
        body: makeReadableBody(JSON.stringify({ jsonrpc: '2.0', result: { tools: [] }, id: 2 })),
      })
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await client.initialize()
    expect(client.getSessionId()).toBe('sess-1')

    await client.listTools()
    expect(client.getSessionId()).toBe('sess-2')
  })
})

describe('McpClient notification listener lifecycle', () => {
  it('dispatches notifications and stops cleanly', async () => {
    const notifications: Array<{ method: string; params?: unknown }> = []

    const sseStream = [
      'data: {"jsonrpc":"2.0","method":"skills/changed","params":{"id":"tdd"}}\n\n',
      'data: {"jsonrpc":"2.0","method":"agents/updated"}\n\n',
    ].join('')

    const fetchMock = vi.fn()
      .mockResolvedValueOnce(initResponse())
      .mockResolvedValueOnce(notifAck())
      // GET for notification listener
      .mockResolvedValueOnce({
        ok: true, status: 200, statusText: 'OK',
        headers: new Headers({ 'Content-Type': 'text/event-stream' }),
        body: makeReadableBody(sseStream),
      })
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await client.initialize()

    await client.startNotificationListener((method, params) => {
      notifications.push({ method, params })
    })

    expect(notifications).toHaveLength(2)
    expect(notifications[0].method).toBe('skills/changed')
    expect(notifications[0].params).toEqual({ id: 'tdd' })
    expect(notifications[1].method).toBe('agents/updated')
  })

  it('stopNotificationListener aborts an existing listener', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(initResponse())
      .mockResolvedValueOnce(notifAck())
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await client.initialize()

    // Stop should not throw even if no listener is active
    client.stopNotificationListener()
    expect(client.getSessionId()).toBe('sess-1')
  })
})

describe('McpClient initialized notification ordering', () => {
  it('sends initialized notification before tools/list', async () => {
    const callBodies: string[] = []
    const fetchMock = vi.fn(async (_url: string, opts?: RequestInit) => {
      if (opts?.body) callBodies.push(opts.body as string)
      const idx = callBodies.length
      if (idx === 1) return initResponse()
      if (idx === 2) return notifAck()
      return {
        ok: true, status: 200, statusText: 'OK',
        headers: new Headers(),
        json: async () => ({ jsonrpc: '2.0', result: { tools: [{ name: 't1' }] }, id: 3 }),
        text: async () => '',
        body: makeReadableBody(JSON.stringify({ jsonrpc: '2.0', result: { tools: [{ name: 't1' }] }, id: 3 })),
      } as unknown as Response
    })
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await client.initialize()
    await client.listTools()

    // Verify ordering: initialize -> notifications/initialized -> tools/list
    expect(callBodies).toHaveLength(3)
    expect(JSON.parse(callBodies[0]).method).toBe('initialize')
    expect(JSON.parse(callBodies[1]).method).toBe('notifications/initialized')
    expect(JSON.parse(callBodies[2]).method).toBe('tools/list')
  })
})
