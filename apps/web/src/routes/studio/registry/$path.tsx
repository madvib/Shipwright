import { createFileRoute, Link } from '@tanstack/react-router'
import { useState, useEffect, useCallback, useRef, type ReactNode } from 'react'
import {
  ArrowLeft, ExternalLink, Download, Copy, Check, AlertTriangle,
  ShieldAlert, Plus, Loader2, Github, X,
} from 'lucide-react'
import { toast } from 'sonner'
import { usePackageDetail } from '#/features/registry/useRegistry'
import { SCOPE_COLORS } from '#/features/registry/types'
import { SkillsList } from '#/features/registry/SkillsList'
import { VersionsTable } from '#/features/registry/VersionsTable'
import { useLibrary } from '#/features/compiler/useLibrary'
import { authClient } from '#/lib/auth-client'
import type { PackageSkill } from '#/features/registry/types'
import type { Skill } from '#/features/compiler/types'

export const Route = createFileRoute('/studio/registry/$path')({ component: PackageDetailPage })

type DetailTab = 'skills' | 'versions'

/** Convert a registry PackageSkill into a Skill for the library. */
function packageSkillToLibrarySkill(ps: PackageSkill): Skill {
  return {
    id: ps.skill_id,
    name: ps.name,
    description: ps.description || null,
    version: null,
    author: null,
    content: `# ${ps.name}\n\n${ps.description ?? ''}\n`,
    source: 'imported',
  }
}

// ── Claim Dialog ──────────────────────────────────────────────────────────────

interface ClaimDialogProps {
  open: boolean
  packagePath: string
  repoUrl: string
  onClose: () => void
}

function ClaimDialog({ open, packagePath, repoUrl, onClose }: ClaimDialogProps) {
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
                  Claim ownership of this unofficial package by verifying you are a maintainer of{' '}
                  <a
                    href={repoUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-foreground underline underline-offset-2"
                  >
                    {repoUrl.replace('https://', '')}
                  </a>
                  .
                </p>
                <p className="text-[11px] text-muted-foreground/60">
                  We verify your GitHub account has admin or write access to the source repository.
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

// ── Main page ─────────────────────────────────────────────────────────────────

function PackageDetailPage() {
  const { path } = Route.useParams()
  const decodedPath = decodeURIComponent(path)
  const { data, isLoading } = usePackageDetail(decodedPath)
  const [tab, setTab] = useState<DetailTab>('skills')
  const [copied, setCopied] = useState(false)
  const [adding, setAdding] = useState(false)
  const [claimOpen, setClaimOpen] = useState(false)
  const { addSkill } = useLibrary()
  const { data: session } = authClient.useSession()

  const installCmd = `ship add ${decodedPath}`

  function handleCopy() {
    void navigator.clipboard.writeText(installCmd)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  async function handleAddToProject() {
    if (!data) return
    const isSignedIn = !!session?.user
    if (!isSignedIn) {
      void authClient.signIn.social({
        provider: 'github',
        callbackURL: window.location.pathname,
      })
      return
    }

    setAdding(true)
    try {
      const skills = data.skills
      if (skills.length === 0) {
        toast.error('No skills found in this package.')
        return
      }
      for (const ps of skills) {
        addSkill(packageSkillToLibrarySkill(ps))
      }
      toast.success(`Added ${skills.length} skill${skills.length !== 1 ? 's' : ''} from ${data.package.name}`)
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to add skills'
      toast.error(msg)
    } finally {
      setAdding(false)
    }
  }

  if (isLoading) {
    return (
      <div className="h-full flex flex-col">
        <div className="flex-1 overflow-auto p-5 pb-20">
          <div className="animate-pulse">
            <div className="h-5 bg-muted/50 rounded w-1/3 mb-4" />
            <div className="h-3 bg-muted/30 rounded w-2/3 mb-2" />
            <div className="h-3 bg-muted/30 rounded w-1/2" />
          </div>
        </div>
      </div>
    )
  }

  if (!data) {
    return (
      <div className="h-full flex flex-col items-center justify-center gap-3 p-5">
        <p className="text-sm font-medium text-foreground">Package not found</p>
        <p className="text-[11px] text-muted-foreground">No package exists at {decodedPath}</p>
        <Link
          to="/studio/registry"
          className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
        >
          <ArrowLeft className="size-3" />
          Back to registry
        </Link>
      </div>
    )
  }

  const pkg = data.package
  const colors = SCOPE_COLORS[pkg.scope]
  const isSignedIn = !!session?.user

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-auto p-5 pb-20">

        {/* Back link */}
        <Link
          to="/studio/registry"
          className="inline-flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors mb-4 no-underline"
        >
          <ArrowLeft className="size-3" />
          Registry
        </Link>

        {/* Deprecated banner */}
        {pkg.deprecated_by && (
          <div className="mb-4 flex items-center gap-2 rounded-xl border border-amber-500/20 bg-amber-500/5 px-4 py-3">
            <AlertTriangle className="size-4 text-amber-500 shrink-0" />
            <div>
              <p className="text-xs font-medium text-amber-400">This package has been deprecated</p>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Replaced by{' '}
                <Link
                  to={`/studio/registry/${encodeURIComponent(pkg.deprecated_by)}` as '/'}
                  className="text-amber-400 underline underline-offset-2"
                >
                  {pkg.deprecated_by}
                </Link>
              </p>
            </div>
          </div>
        )}

        {/* Header */}
        <div className="mb-5">
          <div className="flex items-start gap-3 mb-2">
            <h1 className="text-lg font-semibold text-foreground">{pkg.name}</h1>
            <span className={`shrink-0 mt-1 rounded-md px-1.5 py-0.5 text-[10px] font-medium border ${colors.bg} ${colors.text} ${colors.border}`}>
              {pkg.scope}
            </span>
            {pkg.scope === 'unofficial' && (
              <span className="shrink-0 mt-1 flex items-center gap-1 rounded-md bg-amber-500/10 border border-amber-500/20 px-1.5 py-0.5">
                <ShieldAlert className="size-3 text-amber-500/70" />
                <span className="text-[10px] font-medium text-amber-500/70">Unverified</span>
              </span>
            )}
          </div>
          <p className="text-[11px] text-muted-foreground leading-relaxed mb-3">
            {pkg.description}
          </p>

          {/* Meta row */}
          <div className="flex flex-wrap items-center gap-3">
            <a
              href={pkg.repo_url}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors no-underline"
            >
              <ExternalLink className="size-3" />
              {pkg.repo_url.replace('https://', '')}
            </a>
            <div className="flex items-center gap-1 text-muted-foreground/60">
              <Download className="size-3" />
              <span className="text-[11px]">{pkg.installs.toLocaleString()} installs</span>
            </div>
            {pkg.latest_version && (
              <span className="rounded-md bg-muted/50 px-1.5 py-0.5 text-[10px] font-mono text-muted-foreground">
                v{pkg.latest_version}
              </span>
            )}
          </div>
        </div>

        {/* Install command */}
        <div className="mb-6 rounded-xl border border-border/60 bg-card p-4">
          <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground mb-2">Install</p>
          <div className="flex items-center gap-2">
            <code className="flex-1 rounded-lg bg-muted/40 px-3 py-2 text-xs font-mono text-foreground">
              {installCmd}
            </code>
            <button
              onClick={handleCopy}
              className="shrink-0 flex items-center gap-1 rounded-lg border border-border/60 bg-card px-3 py-2 text-[11px] font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              {copied ? <><Check className="size-3 text-emerald-500" /> Copied</> : <><Copy className="size-3" /> Copy</>}
            </button>
          </div>
        </div>

        {/* Action buttons */}
        <div className="mb-6 flex flex-wrap items-center gap-3">
          <button
            onClick={() => void handleAddToProject()}
            disabled={adding}
            title={isSignedIn ? undefined : 'Sign in to add packages'}
            className="inline-flex items-center gap-1.5 rounded-lg bg-violet-600 dark:bg-violet-500 hover:bg-violet-500 dark:hover:bg-violet-400 transition-colors px-5 py-2.5 text-xs font-semibold text-primary-foreground disabled:opacity-60 disabled:cursor-not-allowed"
          >
            {adding ? (
              <Loader2 className="size-3.5 animate-spin" />
            ) : (
              <Plus className="size-3.5" />
            )}
            {adding ? 'Adding...' : 'Add to project'}
          </button>

          {pkg.scope === 'unofficial' && !pkg.claimed_by && (
            <button
              onClick={() => setClaimOpen(true)}
              className="rounded-lg border border-amber-500/30 bg-amber-500/5 px-4 py-2.5 text-xs font-medium text-amber-400 transition hover:bg-amber-500/10"
            >
              Claim this package
            </button>
          )}
        </div>

        {/* Not signed in hint */}
        {!isSignedIn && (
          <p className="mb-4 text-[11px] text-muted-foreground/60">
            <button
              onClick={() => void authClient.signIn.social({ provider: 'github', callbackURL: window.location.pathname })}
              className="text-primary underline underline-offset-2 hover:text-primary/80 transition-colors"
            >
              Sign in
            </button>
            {' '}to add packages to your project.
          </p>
        )}

        {/* Tabs */}
        <div className="flex items-center gap-1 border-b border-border/40 mb-4">
          <TabButton active={tab === 'skills'} onClick={() => setTab('skills')} count={data.skills.length}>
            Skills
          </TabButton>
          <TabButton active={tab === 'versions'} onClick={() => setTab('versions')} count={data.versions.length}>
            Versions
          </TabButton>
        </div>

        {/* Tab content */}
        {tab === 'skills' && <SkillsList skills={data.skills} />}
        {tab === 'versions' && <VersionsTable versions={data.versions} />}
      </div>

      {/* Claim dialog */}
      <ClaimDialog
        open={claimOpen}
        packagePath={decodedPath}
        repoUrl={pkg.repo_url}
        onClose={() => setClaimOpen(false)}
      />
    </div>
  )
}

function TabButton({ active, onClick, count, children }: { active: boolean; onClick: () => void; count: number; children: ReactNode }) {
  return (
    <button
      onClick={onClick}
      className={`relative px-3 py-2 text-xs font-medium transition-colors ${
        active ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'
      }`}
    >
      {children}
      <span className="ml-1 text-[10px] text-muted-foreground/50">{count}</span>
      {active && (
        <span className="absolute bottom-0 left-1/2 -translate-x-1/2 w-6 h-[2px] rounded-full bg-primary" />
      )}
    </button>
  )
}
