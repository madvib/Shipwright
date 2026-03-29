// Hooks for fetching git status, diff, and log via MCP tools.
// Gracefully degrades when tools are unavailable (returns null).

import { useQuery } from '@tanstack/react-query'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { sessionKeys } from './query-keys'

export interface GitStatusFile {
  path: string
  status: string
}

export interface GitStatusResult {
  branch: string
  clean: boolean
  staged: GitStatusFile[]
  modified: GitStatusFile[]
  untracked: GitStatusFile[]
  workingDirectory?: string
}

export interface GitLogEntry {
  hash: string
  short_hash?: string
  /** MCP returns `message`, legacy code expected `subject` */
  message?: string
  subject?: string
  author: string
  date: string
}

export function useGitStatus() {
  const mcp = useLocalMcpContext()

  return useQuery({
    queryKey: sessionKeys.gitStatus(),
    queryFn: async (): Promise<GitStatusResult | null> => {
      if (!mcp) return null
      try {
        const raw = await mcp.callTool('get_git_status')
        return JSON.parse(raw) as GitStatusResult
      } catch {
        return null
      }
    },
    enabled: mcp?.status === 'connected',
    staleTime: 10_000,
    refetchInterval: 15_000,
  })
}

export function useGitDiff(base?: string) {
  const mcp = useLocalMcpContext()

  return useQuery({
    queryKey: sessionKeys.gitDiff(base),
    queryFn: async (): Promise<string | null> => {
      if (!mcp) return null
      try {
        const args = base ? { base } : {}
        const raw = await mcp.callTool('get_git_diff', args)
        if (!raw || raw.startsWith('Error:')) return null
        return raw
      } catch {
        return null
      }
    },
    enabled: mcp?.status === 'connected',
    staleTime: 10_000,
    refetchInterval: 15_000,
  })
}

export function useGitLog(limit = 10) {
  const mcp = useLocalMcpContext()

  return useQuery({
    queryKey: sessionKeys.gitLog(limit),
    queryFn: async (): Promise<GitLogEntry[]> => {
      if (!mcp) return []
      try {
        const raw = await mcp.callTool('get_git_log', { limit })
        return JSON.parse(raw) as GitLogEntry[]
      } catch {
        return []
      }
    },
    enabled: mcp?.status === 'connected',
    staleTime: 15_000,
    refetchInterval: 30_000,
  })
}
