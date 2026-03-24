// Minimal MCP client for Streamable HTTP transport.
// Speaks JSON-RPC 2.0 over POST to a local `ship mcp serve --http` endpoint.
// No SDK dependency — just fetch.

export interface McpClientOptions {
  /** Base URL of the MCP server, e.g. "http://localhost:4567" */
  url: string
  /** Bearer token for auth (from ~/.ship/config.toml) */
  token?: string
}

export interface McpTool {
  name: string
  description?: string
  inputSchema?: Record<string, unknown>
}

interface JsonRpcRequest {
  jsonrpc: '2.0'
  method: string
  params?: unknown
  id: number
}

interface JsonRpcResponse {
  jsonrpc: '2.0'
  result?: unknown
  error?: { code: number; message: string; data?: unknown }
  id: number
}

export class McpClientError extends Error {
  constructor(
    message: string,
    public code?: number,
  ) {
    super(message)
    this.name = 'McpClientError'
  }
}

export class McpClient {
  private endpoint: string
  private token?: string
  private nextId = 1
  private sessionId?: string

  constructor(options: McpClientOptions) {
    // Strip trailing slash, append /mcp if not present
    const base = options.url.replace(/\/+$/, '')
    this.endpoint = base.endsWith('/mcp') ? base : `${base}/mcp`
    this.token = options.token
  }

  private async rpc(method: string, params?: unknown): Promise<unknown> {
    const body: JsonRpcRequest = {
      jsonrpc: '2.0',
      method,
      params,
      id: this.nextId++,
    }

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      Accept: 'application/json, text/event-stream',
    }
    if (this.token) headers['Authorization'] = `Bearer ${this.token}`
    if (this.sessionId) headers['Mcp-Session-Id'] = this.sessionId

    const res = await fetch(this.endpoint, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    })

    // Capture session ID from response
    const sid = res.headers.get('Mcp-Session-Id')
    if (sid) this.sessionId = sid

    if (!res.ok) {
      throw new McpClientError(
        `MCP server returned ${res.status}: ${res.statusText}`,
        res.status,
      )
    }

    const contentType = res.headers.get('Content-Type') ?? ''

    if (contentType.includes('text/event-stream')) {
      return this.parseSSE(res)
    }

    const json = (await res.json()) as JsonRpcResponse
    if (json.error) {
      throw new McpClientError(json.error.message, json.error.code)
    }
    return json.result
  }

  private async parseSSE(res: Response): Promise<unknown> {
    const reader = res.body?.getReader()
    if (!reader) throw new McpClientError('No response body')

    const decoder = new TextDecoder()
    let buffer = ''

    // Server sends session-setup event first (empty data), then the JSON-RPC
    // response in a second event. Keep reading chunks until we find it.
    const deadline = Date.now() + 15_000
    try {
      while (Date.now() < deadline) {
        const { done, value } = await reader.read()
        if (value) buffer += decoder.decode(value, { stream: true })

        // Scan buffer for a data line containing JSON
        for (const line of buffer.split('\n')) {
          const trimmed = line.trim()
          if (trimmed.startsWith('data:')) {
            const payload = trimmed.slice(trimmed.startsWith('data: ') ? 6 : 5).trim()
            if (payload.startsWith('{')) {
              const json = JSON.parse(payload) as JsonRpcResponse
              if (json.error) {
                throw new McpClientError(json.error.message, json.error.code)
              }
              return json.result
            }
          }
        }

        if (done) break
      }
    } finally {
      reader.cancel().catch(() => {})
    }

    throw new McpClientError(
      `No JSON-RPC response in SSE stream (${buffer.length} bytes, starts: ${buffer.slice(0, 200).replace(/\n/g, '\\n')})`,
    )
  }

  /** Initialize the MCP session. Must be called before other methods. */
  async initialize(): Promise<{ serverInfo: { name: string; version: string } }> {
    const result = (await this.rpc('initialize', {
      protocolVersion: '2025-03-26',
      capabilities: {},
      clientInfo: { name: 'ship-studio', version: '0.1.0' },
    })) as { serverInfo: { name: string; version: string } }

    // Send initialized notification (no response expected)
    await fetch(this.endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json, text/event-stream',
        ...(this.token ? { Authorization: `Bearer ${this.token}` } : {}),
        ...(this.sessionId ? { 'Mcp-Session-Id': this.sessionId } : {}),
      },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' }),
    })

    return result
  }

  /** List available tools on the MCP server. */
  async listTools(): Promise<McpTool[]> {
    const result = (await this.rpc('tools/list')) as { tools: McpTool[] }
    return result.tools
  }

  /** Call a tool on the MCP server. */
  async callTool(
    name: string,
    args: Record<string, unknown> = {},
  ): Promise<{ content: Array<{ type: string; text?: string }> }> {
    return (await this.rpc('tools/call', { name, arguments: args })) as {
      content: Array<{ type: string; text?: string }>
    }
  }

  /** Get the current session ID (set after initialize). */
  getSessionId(): string | undefined {
    return this.sessionId
  }
}
