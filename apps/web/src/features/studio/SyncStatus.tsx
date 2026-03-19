import { useEffect, useState } from 'react'
import { Check, Loader2 } from 'lucide-react'

export type SyncStatusValue = 'idle' | 'saving' | 'saved' | 'error'

interface SyncStatusProps {
  status: SyncStatusValue
}

/**
 * Minimal non-blocking indicator shown while a library is being saved.
 * - idle: renders nothing
 * - saving: cloud icon + "Saving..."
 * - saved: checkmark + "Saved", fades out after 2 s
 * - error: not shown (offline / unauthenticated — do not nag)
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
    } else {
      setVisible(false)
      setFading(false)
    }
  }, [status])

  if (!visible || status === 'idle' || status === 'error') return null

  return (
    <span
      className={`flex items-center gap-1.5 text-[11px] text-muted-foreground transition-opacity duration-500 ${
        fading ? 'opacity-0' : 'opacity-100'
      }`}
      aria-live="polite"
      aria-label={status === 'saving' ? 'Saving' : 'Saved'}
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
          <span>Saved</span>
        </>
      )}
    </span>
  )
}
