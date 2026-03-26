// Debounced registry search hooks for skill and MCP server autocomplete.
// Both use 300ms debounce and return loading state.

import { useState, useEffect, useRef } from 'react'

// ── Skill search ─────────────────────────────────────────────────────────────

export interface RegistrySkillResult {
  id: string
  path: string
  name: string
  description: string
  scope: string
}

export function useRegistrySkillSearch(query: string, enabled: boolean) {
  const [results, setResults] = useState<RegistrySkillResult[]>([])
  const [loading, setLoading] = useState(false)
  const abortRef = useRef<AbortController | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current)
    if (abortRef.current) abortRef.current.abort()

    if (!enabled || query.length < 2) {
      setResults([])
      setLoading(false)
      return
    }

    setLoading(true)
    timerRef.current = setTimeout(async () => {
      const controller = new AbortController()
      abortRef.current = controller
      try {
        const params = new URLSearchParams({ q: query, limit: '8' })
        const res = await fetch(`/api/registry/search?${params}`, {
          signal: controller.signal,
        })
        if (!res.ok) { setResults([]); return }
        const data = await res.json()
        if (!controller.signal.aborted) {
          setResults(
            (data.packages ?? []).map((p: RegistrySkillResult) => ({
              id: p.id,
              path: p.path,
              name: p.name,
              description: p.description,
              scope: p.scope,
            })),
          )
        }
      } catch {
        if (!controller.signal.aborted) setResults([])
      } finally {
        if (!controller.signal.aborted) setLoading(false)
      }
    }, 300)

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current)
      if (abortRef.current) abortRef.current.abort()
    }
  }, [query, enabled])

  return { results, loading }
}

// ── MCP server search ────────────────────────────────────────────────────────

export interface RegistryMcpResult {
  id: string
  name: string
  description: string | null
  command: string | null
  args: string[]
}

export function useRegistryMcpSearch(query: string, enabled: boolean) {
  const [results, setResults] = useState<RegistryMcpResult[]>([])
  const [loading, setLoading] = useState(false)
  const abortRef = useRef<AbortController | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current)
    if (abortRef.current) abortRef.current.abort()

    if (!enabled || query.length < 2) {
      setResults([])
      setLoading(false)
      return
    }

    setLoading(true)
    timerRef.current = setTimeout(async () => {
      const controller = new AbortController()
      abortRef.current = controller
      try {
        const params = new URLSearchParams({ q: query, limit: '8', vetted: 'false' })
        const res = await fetch(`/api/mcp/servers?${params}`, {
          signal: controller.signal,
        })
        if (!res.ok) { setResults([]); return }
        const data = await res.json()
        if (!controller.signal.aborted) {
          type McpServerJson = { id: string; name: string; description: string | null; command: string | null; args: string[] }
          setResults(
            (data.servers ?? []).map((s: McpServerJson) => ({
              id: s.id,
              name: s.name,
              description: s.description,
              command: s.command,
              args: s.args ?? [],
            })),
          )
        }
      } catch {
        if (!controller.signal.aborted) setResults([])
      } finally {
        if (!controller.signal.aborted) setLoading(false)
      }
    }, 300)

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current)
      if (abortRef.current) abortRef.current.abort()
    }
  }, [query, enabled])

  return { results, loading }
}
