import { useState, useEffect, useCallback, useRef } from 'react'
import { Check, ShieldAlert, AlertTriangle, Loader2, Github, X } from 'lucide-react'
import { toast } from 'sonner'
import { authClient } from '#/lib/auth-client'

export interface ClaimDialogProps {
  open: boolean
  packagePath: string
  repoUrl: string
  onClose: () => void
  onClaimed: () => void
}

export function ClaimDialog({ open, packagePath, repoUrl, onClose, onClaimed }: ClaimDialogProps) {
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle')
  const [errorMsg, setErrorMsg] = useState('')
  const { data: session } = authClient.useSession()

  // Reset on open
  useEffect(() => {
    if (open) { setStatus('idle'); setErrorMsg('') }
  }, [open])

  const handleEscape = useCallback(
    (e: KeyboardEvent) => { if (e.key === 'Escape') onClose() },
    [onClose],
  )
  useEffect(() => {
    if (!open) return
    document.addEventListener('keydown', handleEscape)
    return () => document.removeEventListener('keydown', handleEscape)
  }, [open, handleEscape])

  const cancelRef = useRef<HTMLButtonElement>(null)
  useEffect(() => {
    if (open) cancelRef.current?.focus()
  }, [open])

  async function handleVerify() {
    setStatus('loading')
    setErrorMsg('')
    try {
      const res = await fetch('/api/registry/claim', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ package_path: packagePath }),
      })
      const data = (await res.json()) as { claimed?: boolean; error?: string }
      if (!res.ok) {
        setStatus('error')
        setErrorMsg(data.error ?? `Request failed (${res.status})`)
        return
      }
      setStatus('success')
      toast.success('Package claimed successfully.')
      onClaimed()
    } catch {
      setStatus('error')
      setErrorMsg('Network error — please try again.')
    }
  }

  if (!open) return null

  const isSignedIn = !!session?.user

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={onClose} />
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          role="dialog"
          aria-modal="true"
          className="w-full max-w-sm rounded-xl border border-border/60 bg-card shadow-2xl"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-3.5">
            <div className="flex items-center gap-2">
              <ShieldAlert className="size-4 text-amber-500" />
              <h2 className="font-display text-sm font-semibold text-foreground">Claim this package</h2>
            </div>
            <button
              ref={cancelRef}
              onClick={onClose}
              aria-label="Close"
              className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition"
            >
              <X className="size-4" />
            </button>
          </div>

          {/* Body */}
          <div className="px-5 py-4 space-y-3">
            {status === 'success' ? (
              <div className="flex items-start gap-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20 px-3 py-2.5">
                <Check className="size-4 text-emerald-500 mt-0.5 shrink-0" />
                <p className="text-xs text-emerald-400">You now own this package. The claim has been recorded.</p>
              </div>
            ) : (
              <>
                <p className="text-xs text-muted-foreground">
                  You become the maintainer of this package. You can publish updates and manage versions.
                  The package scope will transition from <span className="font-medium text-amber-400">unofficial</span> to{' '}
                  <span className="font-medium text-emerald-400">community</span>.
                </p>
                <p className="text-[11px] text-muted-foreground/60">
                  We verify your GitHub account has admin or write access to{' '}
                  <a
                    href={repoUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-foreground underline underline-offset-2"
                  >
                    {repoUrl.replace('https://', '')}
                  </a>
                  {' '}before granting ownership.
                </p>
                {!isSignedIn && (
                  <div className="rounded-lg border border-amber-500/20 bg-amber-500/5 px-3 py-2">
                    <p className="text-[11px] text-amber-400">Sign in with GitHub first to claim packages.</p>
                  </div>
                )}
                {status === 'error' && (
                  <div className="flex items-start gap-2 rounded-lg border border-destructive/20 bg-destructive/5 px-3 py-2">
                    <AlertTriangle className="size-3.5 text-destructive mt-0.5 shrink-0" />
                    <p className="text-[11px] text-destructive">{errorMsg}</p>
                  </div>
                )}
              </>
            )}
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-2 border-t border-border/60 px-5 py-3.5">
            <button
              onClick={onClose}
              className="rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              {status === 'success' ? 'Close' : 'Cancel'}
            </button>
            {status !== 'success' && (
              isSignedIn ? (
                <button
                  onClick={() => void handleVerify()}
                  disabled={status === 'loading'}
                  className="inline-flex items-center gap-1.5 rounded-lg bg-violet-600 dark:bg-violet-500 px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-violet-500 dark:hover:bg-violet-400 disabled:opacity-60 disabled:cursor-not-allowed"
                >
                  {status === 'loading' && <Loader2 className="size-3 animate-spin" />}
                  <Github className="size-3" />
                  Verify with GitHub
                </button>
              ) : (
                <button
                  onClick={() =>
                    void authClient.signIn.social({
                      provider: 'github',
                      callbackURL: window.location.pathname,
                    })
                  }
                  className="inline-flex items-center gap-1.5 rounded-lg bg-violet-600 dark:bg-violet-500 px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-violet-500 dark:hover:bg-violet-400"
                >
                  <Github className="size-3" />
                  Sign in with GitHub
                </button>
              )
            )}
          </div>
        </div>
      </div>
    </>
  )
}
