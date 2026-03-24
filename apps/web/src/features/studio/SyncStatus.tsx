import { useEffect, useState } from 'react'
import { Check, Loader2, AlertCircle } from 'lucide-react'

export type SyncStatusValue = 'idle' | 'saving' | 'saved' | 'error'

interface SyncStatusProps {
  status: SyncStatusValue
}

/**
 * Combine multiple sync status values into one, using worst-wins priority:
 * syncing > error > saved > idle
 */
export function combineSyncStatuses(...statuses: SyncStatusValue[]): SyncStatusValue {
  if (statuses.some((s) => s === 'saving')) return 'saving'
  if (statuses.some((s) => s === 'error')) return 'error'
  if (statuses.some((s) => s === 'saved')) return 'saved'
  return 'idle'
}

/**
 * Minimal non-blocking indicator for local save state.
 * - idle: "Local draft" (always visible)
 * - saving: spinner + "Saving..."
 * - saved: checkmark + "Synced to CLI", fades to idle after 2 s
 * - error: warning + "Save failed", fades to idle after 3 s
 */
export function SyncStatus({ status }: SyncStatusProps) {
  const [visible, setVisible] = useState(false)
  const [fading, setFading] = useState(false)

  useEffect(() => {
    if (status === 'saving') {
      setVisible(true)
      setFading(false)
    } else if (status === 'saved') {
      setVisible(true)
      setFading(false)
      const fade = setTimeout(() => setFading(true), 2000)
      const hide = setTimeout(() => setVisible(false), 2600)
      return () => {
        clearTimeout(fade)
        clearTimeout(hide)
      }
    } else if (status === 'error') {
      setVisible(true)
      setFading(false)
      const fade = setTimeout(() => setFading(true), 3000)
      const hide = setTimeout(() => setVisible(false), 3600)
      return () => {
        clearTimeout(fade)
        clearTimeout(hide)
      }
    } else {
      setVisible(false)
      setFading(false)
    }
  }, [status])

  if (!visible && status === 'idle') {
    return (
      <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground" aria-live="polite" aria-label="Local draft">
        <span className="size-1.5 rounded-full bg-muted-foreground/40" />
        <span>Local draft</span>
      </span>
    )
  }

  if (!visible) return null

  return (
    <span
      className={`flex items-center gap-1.5 text-[11px] text-muted-foreground transition-opacity duration-500 ${
        fading ? 'opacity-0' : 'opacity-100'
      }`}
      aria-live="polite"
      aria-label={status === 'saving' ? 'Saving' : status === 'error' ? 'Save failed' : 'Synced to CLI'}
    >
      {status === 'saving' && (
        <>
          <Loader2 className="size-3 animate-spin" />
          <span>Saving...</span>
        </>
      )}
      {status === 'saved' && (
        <>
          <Check className="size-3 text-emerald-500" />
          <span>Synced to CLI</span>
        </>
      )}
      {status === 'error' && (
        <>
          <AlertCircle className="size-3 text-amber-500" />
          <span className="text-amber-500">Save failed</span>
        </>
      )}
    </span>
  )
}
