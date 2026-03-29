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

function mockFetch(responses: Array<{ status: number; body: unknown; headers?: Record<string, string> }>) {
  let callIndex = 0
  return vi.fn(async () => {
    const r = responses[callIndex++] ?? responses[responses.length - 1]
    const jsonStr = JSON.stringify(r.body)
    return {
      ok: r.status >= 200 && r.status < 300,
      status: r.status,
      statusText: r.status === 200 ? 'OK' : 'Error',
      headers: new Headers(r.headers ?? {}),
      json: async () => r.body,
      text: async () => jsonStr,
      body: makeReadableBody(jsonStr),
    } as unknown as Response
  })
}

beforeEach(() => {
  vi.restoreAllMocks()
})

describe('McpClient', () => {
  it('initializes and captures session ID', async () => {
    const fetchMock = mockFetch([
      {
        status: 200,
        body: {
          jsonrpc: '2.0',
          result: { serverInfo: { name: 'ship', version: '0.1.0' } },
          id: 1,
        },
        headers: { 'Mcp-Session-Id': 'sess-123' },
      },
      { status: 200, body: {} }, // initialized notification
    ])
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    const result = await client.initialize()

    expect(result.serverInfo.name).toBe('ship')
    expect(client.getSessionId()).toBe('sess-123')
    expect(fetchMock).toHaveBeenCalledTimes(2)

    // First call is initialize
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const calls = fetchMock.mock.calls as any[][]
    const initBody = JSON.parse(calls[0][1].body as string)
    expect(initBody.method).toBe('initialize')
    expect(initBody.params.clientInfo.name).toBe('ship-studio')
  })

  it('appends /mcp to base URL if missing', async () => {
    const fetchMock = mockFetch([
      { status: 200, body: { jsonrpc: '2.0', result: { serverInfo: { name: 'ship', version: '0.1.0' } }, id: 1 } },
      { status: 200, body: {} },
    ])
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await client.initialize()

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    expect((fetchMock.mock.calls as any[][])[0][0]).toBe('http://localhost:4567/mcp')
  })

  it('does not double-append /mcp', async () => {
    const fetchMock = mockFetch([
      { status: 200, body: { jsonrpc: '2.0', result: { serverInfo: { name: 'ship', version: '0.1.0' } }, id: 1 } },
      { status: 200, body: {} },
    ])
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567/mcp' })
    await client.initialize()

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    expect((fetchMock.mock.calls as any[][])[0][0]).toBe('http://localhost:4567/mcp')
  })

  it('sends bearer token when provided', async () => {
    const fetchMock = mockFetch([
      { status: 200, body: { jsonrpc: '2.0', result: { serverInfo: { name: 'ship', version: '0.1.0' } }, id: 1 } },
      { status: 200, body: {} },
    ])
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567', token: 'secret' })
    await client.initialize()

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const headers = (fetchMock.mock.calls as any[][])[0][1].headers as Record<string, string>
    expect(headers['Authorization']).toBe('Bearer secret')
  })

  it('lists tools', async () => {
    const fetchMock = mockFetch([
      { status: 200, body: { jsonrpc: '2.0', result: { serverInfo: { name: 'ship', version: '0.1.0' } }, id: 1 } },
      { status: 200, body: {} },
      { status: 200, body: { jsonrpc: '2.0', result: { tools: [{ name: 'open_project', description: 'Open a project' }] }, id: 2 } },
    ])
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await client.initialize()
    const tools = await client.listTools()

    expect(tools).toHaveLength(1)
    expect(tools[0].name).toBe('open_project')
  })

  it('calls a tool and returns text content', async () => {
    const fetchMock = mockFetch([
      { status: 200, body: { jsonrpc: '2.0', result: { serverInfo: { name: 'ship', version: '0.1.0' } }, id: 1 } },
      { status: 200, body: {} },
      { status: 200, body: { jsonrpc: '2.0', result: { content: [{ type: 'text', text: '{"ok":true}' }] }, id: 2 } },
    ])
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await client.initialize()
    const result = await client.callTool('open_project', { path: '/tmp' })

    expect(result.content[0].text).toBe('{"ok":true}')
  })

  it('throws McpClientError on HTTP error', async () => {
    const fetchMock = mockFetch([
      { status: 401, body: {} },
    ])
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await expect(client.initialize()).rejects.toThrow(McpClientError)
  })

  it('throws McpClientError on JSON-RPC error', async () => {
    const fetchMock = mockFetch([
      { status: 200, body: { jsonrpc: '2.0', error: { code: -32600, message: 'Invalid request' }, id: 1 } },
    ])
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    await expect(client.initialize()).rejects.toThrow('Invalid request')
  })

  it('handles SSE response format', async () => {
    const ssePayload = '{"jsonrpc":"2.0","result":{"serverInfo":{"name":"ship","version":"0.1.0"}},"id":1}'
    const sseText = `data: \n\ndata: ${ssePayload}\n\n`

    const fetchMock = vi.fn()
      .mockResolvedValueOnce({
        ok: true, status: 200, statusText: 'OK',
        headers: new Headers({ 'Content-Type': 'text/event-stream', 'Mcp-Session-Id': 'sse-sess' }),
        body: makeReadableBody(sseText),
      })
      .mockResolvedValueOnce({
        ok: true, status: 200, statusText: 'OK',
        headers: new Headers(),
        json: async () => ({}),
        text: async () => '',
      })
    vi.stubGlobal('fetch', fetchMock)

    const client = new McpClient({ url: 'http://localhost:4567' })
    const result = await client.initialize()
    expect(result.serverInfo.name).toBe('ship')
    expect(client.getSessionId()).toBe('sse-sess')
  })
})
