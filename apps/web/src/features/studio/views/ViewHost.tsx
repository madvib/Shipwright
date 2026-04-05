// ViewHost — iframe container that bridges ship-sdk postMessage to daemon APIs.
// Handles request/response proxying, SSE event forwarding, and theme sync.

import { useRef, useEffect, useCallback } from 'react'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { getResolvedTheme } from '#/features/studio/session/canvas-helpers'

interface ViewHostProps {
  /** Raw HTML content of the view (loaded by the parent). */
  html: string
  /** Active workspace branch ID for file operations. */
  workspaceId?: string
}

export function ViewHost({ html, workspaceId }: ViewHostProps) {
  const iframeRef = useRef<HTMLIFrameElement>(null)

  // Bridge postMessage requests from the view to daemon HTTP APIs.
  const handleMessage = useCallback(async (e: MessageEvent) => {
    const msg = e.data
    if (!msg || !msg.__ship || msg.type !== 'request') return

    const iframe = iframeRef.current
    if (!iframe?.contentWindow) return
    // Only handle messages from our iframe
    if (e.source !== iframe.contentWindow) return

    const { method, params, seq } = msg
    try {
      const data = await routeRequest(method, params, workspaceId)
      iframe.contentWindow.postMessage({ __ship: true, type: 'response', seq, data }, '*')
    } catch (err) {
      const error = err instanceof Error ? err.message : String(err)
      iframe.contentWindow.postMessage({ __ship: true, type: 'response', seq, error }, '*')
    }
  }, [workspaceId])

  useEffect(() => {
    window.addEventListener('message', handleMessage)
    return () => window.removeEventListener('message', handleMessage)
  }, [handleMessage])

  // Forward SSE events from daemon to the view iframe.
  useEffect(() => {
    const iframe = iframeRef.current
    if (!iframe) return

    const es = new EventSource(`${DAEMON_BASE_URL}/api/runtime/events`)
    es.addEventListener('ship.event', (e: MessageEvent) => {
      try {
        const envelope = JSON.parse(e.data as string)
        iframe.contentWindow?.postMessage({ __ship: true, type: 'event', envelope }, '*')
      } catch { /* ignore malformed */ }
    })

    return () => es.close()
  }, [])

  // Sync theme to iframe on load and on host theme changes.
  const postTheme = useCallback(() => {
    const iframe = iframeRef.current
    if (!iframe?.contentWindow) return
    const theme = getResolvedTheme()
    iframe.contentWindow.postMessage({ type: 'theme', theme }, '*')
    iframe.contentWindow.postMessage({ __ship: true, type: 'theme', theme }, '*')
  }, [])

  useEffect(() => {
    const observer = new MutationObserver(() => postTheme())
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'data-theme'] })
    return () => observer.disconnect()
  }, [postTheme])

  return (
    <iframe
      ref={iframeRef}
      srcDoc={html}
      title="Ship View"
      sandbox="allow-same-origin allow-scripts"
      className="w-full h-full border-0"
      onLoad={postTheme}
    />
  )
}

// Route SDK method calls to daemon HTTP endpoints.
async function routeRequest(method: string, params: Record<string, unknown> | undefined, activeWsId?: string): Promise<unknown> {
  switch (method) {
    case 'workspace.active':
      return activeWsId ?? null

    case 'jobs.list': {
      const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/jobs`)
      if (!res.ok) throw new Error(`jobs.list: ${res.status}`)
      const body = await res.json() as { ok: boolean; data: { jobs: unknown[] } }
      return body.data.jobs
    }

    case 'jobs.create': {
      const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/jobs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(params),
      })
      if (!res.ok) throw new Error(`jobs.create: ${res.status}`)
      return await res.json()
    }

    case 'events.emit': {
      const p = params as { event_type: string; entity_id: string; payload: unknown }
      const res = await fetch(`${DAEMON_BASE_URL}/api/events/emit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(p),
      })
      if (!res.ok) throw new Error(`events.emit: ${res.status}`)
      return await res.json()
    }

    case 'files.list': {
      const p = params as { workspace_id: string }
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(p.workspace_id)}/session-files`)
      if (!res.ok) throw new Error(`files.list: ${res.status}`)
      return await res.json()
    }

    case 'files.read': {
      const p = params as { workspace_id: string; path: string }
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(p.workspace_id)}/session-files/${p.path}`)
      if (!res.ok) throw new Error(`files.read: ${res.status}`)
      return await res.json()
    }

    default:
      throw new Error(`unknown method: ${method}`)
  }
}
