// React hook for connecting to a local `ship mcp serve --http` instance.
// Manages connection lifecycle, exposes tool calls, and tracks status.

import { useState, useCallback, useRef, useEffect } from 'react'
import { McpClient, McpClientError } from '#/lib/mcp-client'
import type { McpTool } from '#/lib/mcp-client'

export type McpConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

const DEFAULT_PORT = 51741
const STORAGE_KEY = 'ship-mcp-config'
const EVER_CONNECTED_KEY = 'ship-cli-ever-connected'

interface McpConfig {
  port: number
  token?: string
}

function loadConfig(): McpConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return JSON.parse(raw) as McpConfig
  } catch { /* ignore */ }
  return { port: DEFAULT_PORT }
}

function saveConfig(config: McpConfig): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(config))
}

export interface UseLocalMcpReturn {
  status: McpConnectionStatus
  error: string | null
  serverName: string | null
  tools: McpTool[]
  localAgentIds: Set<string>
  hasEverConnected: boolean
  port: number
  setPort: (port: number) => void
  setToken: (token: string | undefined) => void
  connect: () => Promise<void>
  disconnect: () => void
  callTool: (name: string, args?: Record<string, unknown>) => Promise<string>
  refreshLocalAgents: () => Promise<void>
}

export function useLocalMcp(): UseLocalMcpReturn {
  const [status, setStatus] = useState<McpConnectionStatus>('disconnected')
  const [error, setError] = useState<string | null>(null)
  const [serverName, setServerName] = useState<string | null>(null)
  const [tools, setTools] = useState<McpTool[]>([])
  const [localAgentIds, setLocalAgentIds] = useState<Set<string>>(new Set())
  const [config, setConfig] = useState<McpConfig>(loadConfig)
  const [hasEverConnected, setHasEverConnected] = useState(
    () => localStorage.getItem(EVER_CONNECTED_KEY) === 'true',
  )
  const clientRef = useRef<McpClient | null>(null)

  const setPort = useCallback((port: number) => {
    setConfig((prev) => {
      const next = { ...prev, port }
      saveConfig(next)
      return next
    })
  }, [])

  const setToken = useCallback((token: string | undefined) => {
    setConfig((prev) => {
      const next = { ...prev, token }
      saveConfig(next)
      return next
    })
  }, [])

  const disconnect = useCallback(() => {
    clientRef.current = null
    setStatus('disconnected')
    setError(null)
    setServerName(null)
    setTools([])
    setLocalAgentIds(new Set())
  }, [])

  const connect = useCallback(async () => {
    disconnect()
    setStatus('connecting')
    setError(null)

    const client = new McpClient({
      url: `http://localhost:${config.port}`,
      token: config.token,
    })

    try {
      const { serverInfo } = await client.initialize()
      clientRef.current = client
      setServerName(`${serverInfo.name} ${serverInfo.version}`)

      const toolList = await client.listTools()
      setTools(toolList)
      setStatus('connected')
      setHasEverConnected(true)
      localStorage.setItem(EVER_CONNECTED_KEY, 'true')

      // Fetch which agents exist locally for sync badges
      try {
        const agentResult = await client.callTool('list_local_agents')
        const text = agentResult.content.find((c) => c.type === 'text')?.text
        if (text) {
          const parsed = JSON.parse(text) as { agents: string[] }
          setLocalAgentIds(new Set(parsed.agents))
        }
      } catch { /* non-fatal */ }
    } catch (err) {
      clientRef.current = null
      const msg =
        err instanceof McpClientError
          ? err.message
          : err instanceof TypeError
            ? `Cannot reach localhost:${config.port} — is \`ship mcp serve --http\` running?`
            : 'Connection failed'
      setError(msg)
      setStatus('error')
    }
  }, [config.port, config.token, disconnect])

  const callTool = useCallback(
    async (name: string, args: Record<string, unknown> = {}): Promise<string> => {
      const client = clientRef.current
      if (!client) throw new Error('Not connected to MCP server')

      const result = await client.callTool(name, args)
      const textContent = result.content.find((c) => c.type === 'text')
      return textContent?.text ?? JSON.stringify(result.content)
    },
    [],
  )

  const refreshLocalAgents = useCallback(async () => {
    const client = clientRef.current
    if (!client) return
    try {
      const result = await client.callTool('list_local_agents')
      const text = result.content.find((c) => c.type === 'text')?.text
      if (text) {
        const parsed = JSON.parse(text) as { agents: string[] }
        setLocalAgentIds(new Set(parsed.agents))
      }
    } catch {
      // Non-fatal — just can't show sync badges
    }
  }, [])

  // Clean up on unmount
  useEffect(() => {
    return () => { clientRef.current = null }
  }, [])

  return {
    status,
    error,
    serverName,
    tools,
    localAgentIds,
    hasEverConnected,
    port: config.port,
    setPort,
    setToken,
    connect,
    disconnect,
    callTool,
    refreshLocalAgents,
  }
}
