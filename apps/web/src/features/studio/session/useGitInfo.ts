// Hooks for fetching git status, diff, and log via the shipd daemon.

import { useQuery } from '@tanstack/react-query'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
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
  message?: string
  subject?: string
  author: string
  date: string
}

function useActiveWorkspaceId(): string {
  const { workspaces } = useDaemon()
  return workspaces.find((w) => w.status === 'active')?.branch ?? ''
}

export function useGitStatus() {
  const wsId = useActiveWorkspaceId()

  return useQuery({
    queryKey: sessionKeys.gitStatus(),
    queryFn: async (): Promise<GitStatusResult | null> => {
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/git/status`)
      if (!res.ok) return null
      const body = (await res.json()) as { ok: boolean; data: { output: string } }
      return JSON.parse(body.data.output) as GitStatusResult
    },
    staleTime: 10_000,
    refetchInterval: 15_000,
  })
}

export function useGitDiff(range?: string) {
  const wsId = useActiveWorkspaceId()

  return useQuery({
    queryKey: sessionKeys.gitDiff(range),
    queryFn: async (): Promise<string | null> => {
      const params = range ? `?range=${encodeURIComponent(range)}` : ''
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/git/diff${params}`)
      if (!res.ok) return null
      const body = (await res.json()) as { ok: boolean; data: { output: string } }
      if (!body.data.output.trim()) return null
      return body.data.output
    },
    enabled: range !== undefined,
    staleTime: 10_000,
    refetchInterval: 15_000,
  })
}

export function useGitLog(limit = 10) {
  const wsId = useActiveWorkspaceId()

  return useQuery({
    queryKey: sessionKeys.gitLog(limit),
    queryFn: async (): Promise<GitLogEntry[]> => {
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/git/log?limit=${limit}`)
      if (!res.ok) return []
      const body = (await res.json()) as { ok: boolean; data: { output: string; commits: GitLogEntry[] } }
      return body.data.commits
    },
    staleTime: 15_000,
    refetchInterval: 30_000,
  })
}
