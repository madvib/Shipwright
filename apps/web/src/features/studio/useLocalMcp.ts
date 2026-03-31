// React hook for connecting to a local `ship studio` instance.
// Manages connection lifecycle, exposes tool calls, and tracks status.
// Listens for server-pushed notifications via SSE to reactively invalidate
// React Query caches instead of polling.

import { useState, useCallback, useRef, useEffect } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { McpClient, McpClientError } from '#/lib/mcp-client'
import type { McpTool } from '#/lib/mcp-client'
import { mcpKeys } from '#/lib/query-keys'
import { startNotificationListener } from './notification-listener'

export type McpConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

const DEFAULT_PORT = 51741
const STORAGE_KEY = 'ship-mcp-config'
const EVER_CONNECTED_KEY = 'ship-cli-ever-connected'

interface McpConfig {
  port: number
  token?: string
}

function loadConfig(): McpConfig {
  if (typeof window === 'undefined') return { port: DEFAULT_PORT }
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
    () => typeof window !== 'undefined' && localStorage.getItem(EVER_CONNECTED_KEY) === 'true',
  )
  const clientRef = useRef<McpClient | null>(null)
  const queryClient = useQueryClient()
  // Track whether the notification listener is actively running
  const listenerActiveRef = useRef(false)

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
    if (clientRef.current) {
      clientRef.current.stopNotificationListener()
    }
    listenerActiveRef.current = false
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

      // Health check: verify the session actually works with a real tool call.
      // If the server doesn't recognize the session, this will throw and we
      // won't falsely report 'connected'.
      if (toolList.some((t) => t.name === 'get_project_info')) {
        await client.callTool('get_project_info')
      }

      setStatus('connected')
      setHasEverConnected(true)
      localStorage.setItem(EVER_CONNECTED_KEY, 'true')

      // Seed the events cache so it exists before any events arrive
      if (!queryClient.getQueryData(mcpKeys.events())) {
        queryClient.setQueryData(mcpKeys.events(), [])
      }

      // Start the SSE notification listener for reactive cache invalidation.
      // Runs in the background -- stream drop triggers a single refetch as fallback.
      startNotificationListener(client, queryClient, { listenerActiveRef, clientRef })

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
            ? `Cannot reach localhost:${config.port} — is \`ship studio\` running?`
            : 'Connection failed'
      setError(msg)
      setStatus('error')
    }
  }, [config.port, config.token, disconnect]) // eslint-disable-line react-hooks/exhaustive-deps -- startNotificationListener is stable via refs

  // Track whether a reconnect is in-flight to prevent concurrent reconnects
  const reconnectingRef = useRef<Promise<void> | null>(null)

  const callTool = useCallback(
    async (name: string, args: Record<string, unknown> = {}): Promise<string> => {
      const client = clientRef.current
      if (!client) throw new Error('Not connected to MCP server')

      try {
        const result = await client.callTool(name, args)
        const textContent = result.content.find((c) => c.type === 'text')
        const text = textContent?.text ?? JSON.stringify(result.content)

        // Bug 3: Surface project-not-found errors clearly
        if (result.content.some((c) =>
          c.type === 'text' && c.text && /no active project|project.not.(found|set|open)/i.test(c.text),
        )) {
          setError('No active project. Run `ship studio` from your project directory.')
        }

        return text
      } catch (err) {
        // Bug 1: Detect stale session after server restart.
        // Server returns 4xx or errors mentioning "initialized" when session is dead.
        const isStaleSession =
          (err instanceof McpClientError && err.code !== undefined && err.code >= 400 && err.code < 500) ||
          (err instanceof Error && /initializ/i.test(err.message))

        if (isStaleSession) {
          // Reconnect once, then let React Query's refetch retry the call
          if (!reconnectingRef.current) {
            reconnectingRef.current = connect().finally(() => {
              reconnectingRef.current = null
            })
          }
          await reconnectingRef.current
          throw new Error('Session expired — reconnecting. Retry automatically.')
        }

        throw err
      }
    },
    [connect],
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

  // Auto-connect on mount when previously connected
  useEffect(() => {
    if (hasEverConnected && status === 'disconnected') {
      const timer = setTimeout(() => void connect(), 500)
      return () => clearTimeout(timer)
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps -- intentionally run once on mount

  // Clean up on unmount
  useEffect(() => {
    return () => {
      listenerActiveRef.current = false
      if (clientRef.current) {
        clientRef.current.stopNotificationListener()
      }
      clientRef.current = null
    }
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
