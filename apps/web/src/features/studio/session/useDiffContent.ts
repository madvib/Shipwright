// Hook that reads diff content from two sources:
// 1. Daemon git diff endpoint (preferred, live working tree diff)
// 2. .ship-session/diff.txt fallback (manual or agent-generated)

import { useQuery } from '@tanstack/react-query'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import { sessionKeys } from './query-keys'

const DIFF_PATH = 'diff.txt'

export function useDiffContent() {
  const { workspaces } = useDaemon()
  const wsId = workspaces.find((w) => w.status === 'active')?.branch ?? 'v0.2.0'

  const query = useQuery({
    queryKey: sessionKeys.diff(),
    queryFn: async (): Promise<string | null> => {
      // Try git diff first
      try {
        const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/git/diff`)
        if (res.ok) {
          const body = (await res.json()) as { ok: boolean; data: { output: string } }
          if (body.data.output.trim().length > 0) return body.data.output
        }
      } catch {
        // endpoint may not exist yet — fall through
      }

      // Fall back to diff.txt session file
      try {
        const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(DIFF_PATH)}`)
        if (!res.ok) return null
        const body = (await res.json()) as { ok: boolean; data: { content: string } }
        return body.data.content
      } catch {
        return null
      }
    },
    staleTime: 5_000,
    refetchInterval: 10_000,
  })

  return {
    diffText: query.data ?? null,
    isLoading: query.isLoading,
  }
}
