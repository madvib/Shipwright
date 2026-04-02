// Terminal tab — connects to shipd PTY WebSocket endpoint.
// Connects only when the tab is visible. Does not auto-reconnect.

import { useEffect, useRef, useCallback, useState } from 'react'
import type { IDisposable } from '@xterm/xterm'
import { Terminal as XTerm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { RefreshCw, TerminalIcon } from 'lucide-react'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import '@xterm/xterm/css/xterm.css'

interface Props {
  workspaceId: string | null
  visible: boolean
}

type ConnectionState = 'idle' | 'connecting' | 'connected' | 'disconnected' | 'error'

function buildWsUrl(workspaceId: string): string {
  const base = DAEMON_BASE_URL.replace(/^http/, 'ws')
  return `${base}/api/runtime/workspaces/${workspaceId}/pty`
}

export function TerminalTab({ workspaceId, visible }: Props) {
  const containerRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<XTerm | null>(null)
  const fitAddonRef = useRef<FitAddon | null>(null)
  const wsRef = useRef<WebSocket | null>(null)
  const inputDisposableRef = useRef<IDisposable | null>(null)
  const resizeDisposableRef = useRef<IDisposable | null>(null)
  const [connState, setConnState] = useState<ConnectionState>('idle')

  // Init xterm once on mount
  useEffect(() => {
    const term = new XTerm({
      theme: {
        background: '#09090b',
        foreground: '#e4e4e7',
        cursor: '#a1a1aa',
        selectionBackground: '#3f3f46',
      },
      fontSize: 12,
      fontFamily: '"JetBrains Mono", "Fira Code", monospace',
      cursorBlink: true,
      scrollback: 2000,
    })
    const fit = new FitAddon()
    term.loadAddon(fit)
    xtermRef.current = term
    fitAddonRef.current = fit

    if (containerRef.current) {
      term.open(containerRef.current)
      fit.fit()
    }

    return () => {
      term.dispose()
      xtermRef.current = null
      fitAddonRef.current = null
    }
  }, [])

  // ResizeObserver to refit when drawer resizes
  useEffect(() => {
    if (!containerRef.current) return
    const ro = new ResizeObserver(() => {
      fitAddonRef.current?.fit()
    })
    ro.observe(containerRef.current)
    return () => ro.disconnect()
  }, [])

  const disconnect = useCallback(() => {
    inputDisposableRef.current?.dispose()
    inputDisposableRef.current = null
    resizeDisposableRef.current?.dispose()
    resizeDisposableRef.current = null
    const ws = wsRef.current
    if (ws) {
      ws.onopen = null
      ws.onclose = null
      ws.onerror = null
      ws.onmessage = null
      ws.close()
      wsRef.current = null
    }
  }, [])

  const connect = useCallback(() => {
    if (!workspaceId) return
    const term = xtermRef.current
    if (!term) return

    disconnect()
    setConnState('connecting')
    term.reset()

    const url = buildWsUrl(workspaceId)
    let ws: WebSocket
    try {
      ws = new WebSocket(url)
    } catch {
      setConnState('error')
      term.writeln('\r\n\x1b[31mTerminal unavailable — daemon not connected\x1b[0m')
      return
    }
    wsRef.current = ws
    ws.binaryType = 'arraybuffer'

    // Forward input → WS (registered once, lives until disconnect)
    inputDisposableRef.current = term.onData((data) => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: 'input', data }))
      }
    })

    // Forward resize → WS
    resizeDisposableRef.current = term.onResize(({ cols, rows }) => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: 'resize', cols, rows }))
      }
    })

    ws.onopen = () => {
      setConnState('connected')
      fitAddonRef.current?.fit()
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'resize', cols: term.cols, rows: term.rows }))
      }
    }

    ws.onmessage = (evt) => {
      if (typeof evt.data === 'string') {
        term.write(evt.data)
      } else if (evt.data instanceof ArrayBuffer) {
        term.write(new Uint8Array(evt.data))
      }
    }

    ws.onerror = () => {
      setConnState('error')
      term.writeln('\r\n\x1b[31mTerminal unavailable — daemon not connected\x1b[0m')
    }

    ws.onclose = () => {
      setConnState('disconnected')
      term.writeln('\r\n\x1b[33mSession ended\x1b[0m')
      wsRef.current = null
    }
  }, [workspaceId, disconnect])

  // Connect when tab becomes visible, disconnect when hidden
  useEffect(() => {
    if (visible && workspaceId && connState === 'idle') {
      connect()
    }
    if (!visible) {
      disconnect()
      if (connState !== 'idle') setConnState('idle')
    }
  }, [visible]) // eslint-disable-line react-hooks/exhaustive-deps

  // Disconnect on unmount
  useEffect(() => {
    return () => disconnect()
  }, [disconnect])

  if (!workspaceId) {
    return (
      <div className="flex flex-col items-center justify-center flex-1 gap-3 px-4 text-center">
        <TerminalIcon className="size-8 text-muted-foreground/40" />
        <p className="text-xs text-muted-foreground">No active workspace</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col flex-1 min-h-0">
      {/* Status bar */}
      <div className="flex items-center justify-between px-2 py-1 border-b border-border shrink-0">
        <span className="text-[10px] text-muted-foreground">
          {connState === 'connected' && (
            <span className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-green-500 inline-block" />
              Connected
            </span>
          )}
          {connState === 'connecting' && (
            <span className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-yellow-500 inline-block animate-pulse" />
              Connecting…
            </span>
          )}
          {(connState === 'disconnected' || connState === 'error') && (
            <span className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-red-500 inline-block" />
              {connState === 'error' ? 'Unavailable' : 'Disconnected'}
            </span>
          )}
          {connState === 'idle' && (
            <span className="text-muted-foreground/60">Not connected</span>
          )}
        </span>
        {(connState === 'disconnected' || connState === 'error' || connState === 'idle') && (
          <button
            onClick={connect}
            className="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground transition-colors"
            title="Reconnect"
          >
            <RefreshCw className="size-3" />
            Reconnect
          </button>
        )}
      </div>

      {/* xterm container */}
      <div
        ref={containerRef}
        className="flex-1 min-h-0 p-1 bg-[#09090b] overflow-hidden"
        style={{ fontVariantLigatures: 'none' }}
      />
    </div>
  )
}
