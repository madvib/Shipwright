import { useState } from 'react'
import { Link } from '@tanstack/react-router'
import {
  ArrowLeft, ExternalLink, Download, Copy, Check, CheckCircle2,
  AlertTriangle, ShieldAlert, Plus, Loader2,
} from 'lucide-react'
import { toast } from 'sonner'
import { SCOPE_COLORS, extractGitHubOwner } from '#/features/registry/types'
import { useLibrary } from '#/features/compiler/useLibrary'
import { authClient } from '#/lib/auth-client'
import type { RegistryPackage, PackageSkill } from '#/features/registry/types'
import type { Skill } from '@ship/ui'

/** Convert a registry PackageSkill into a Skill for the library. */
function packageSkillToLibrarySkill(ps: PackageSkill): Skill {
  return {
    id: ps.skillId,
    name: ps.name,
    description: ps.description || null,
    content: `# ${ps.name}\n\n${ps.description ?? ''}\n`,
    source: 'imported',
  }
}

/** Owner avatar with error fallback. */
function OwnerAvatar({ owner }: { owner: string }) {
  const [failed, setFailed] = useState(false)

  if (failed) {
    return (
      <div
        className="size-7 shrink-0 rounded-full bg-muted flex items-center justify-center text-[10px] font-semibold text-muted-foreground uppercase"
        aria-hidden="true"
      >
        {owner[0]}
      </div>
    )
  }

  return (
    <img
      src={`https://github.com/${owner}.png?size=56`}
      alt={`${owner} avatar`}
      width={28}
      height={28}
      loading="lazy"
      className="size-7 shrink-0 rounded-full"
      onError={() => setFailed(true)}
    />
  )
}

export interface PackageHeaderProps {
  pkg: RegistryPackage
  skills: PackageSkill[]
  decodedPath: string
  onClaimClick: () => void
}

export function PackageHeader({ pkg, skills, decodedPath, onClaimClick }: PackageHeaderProps) {
  const [copied, setCopied] = useState(false)
  const [adding, setAdding] = useState(false)
  const { addSkill } = useLibrary()
  const { data: session } = authClient.useSession()

  const installCmd = `ship add ${decodedPath}`
  const colors = SCOPE_COLORS[pkg.scope]
  const isSignedIn = !!session?.user
  const owner = extractGitHubOwner(pkg.repoUrl)

  function handleCopy() {
    void navigator.clipboard.writeText(installCmd)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  async function handleAddToProject() {
    if (!isSignedIn) {
      void authClient.signIn.social({
        provider: 'github',
        callbackURL: window.location.pathname,
      })
      return
    }

    setAdding(true)
    try {
      if (skills.length === 0) {
        toast.error('No skills found in this package.')
        return
      }
      for (const ps of skills) {
        addSkill(packageSkillToLibrarySkill(ps))
      }
      toast.success(`Added ${skills.length} skill${skills.length !== 1 ? 's' : ''} from ${pkg.name}`)
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to add skills'
      toast.error(msg)
    } finally {
      setAdding(false)
    }
  }

  return (
    <>
      {/* Back link */}
      <Link
        to="/registry"
        className="inline-flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors mb-4 no-underline"
      >
        <ArrowLeft className="size-3" />
        Registry
      </Link>

      {/* Deprecated banner */}
      {pkg.deprecatedBy && (
        <div className="mb-4 flex items-center gap-2 rounded-xl border border-amber-500/20 bg-amber-500/5 px-4 py-3">
          <AlertTriangle className="size-4 text-amber-500 shrink-0" />
          <div>
            <p className="text-xs font-medium text-amber-400">This package has been deprecated</p>
            <p className="text-[11px] text-muted-foreground mt-0.5">
              Replaced by{' '}
              <Link
                to={`/registry/${encodeURIComponent(pkg.deprecatedBy)}` as '/'}
                className="text-amber-400 underline underline-offset-2"
              >
                {pkg.deprecatedBy}
              </Link>
            </p>
          </div>
        </div>
      )}

      {/* Owner info */}
      {owner && (
        <div className="flex items-center gap-2 mb-3">
          <OwnerAvatar owner={owner} />
          <a
            href={`https://github.com/${owner}`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs text-muted-foreground hover:text-foreground transition-colors no-underline"
          >
            {owner}
          </a>
          {pkg.claimedBy && (
            <CheckCircle2 className="size-3.5 text-blue-500" aria-label="Verified owner" />
          )}
        </div>
      )}

      {/* Name + badges */}
      <div className="mb-5">
        <div className="flex items-start gap-3 mb-2">
          <h1 className="text-lg font-semibold text-foreground">{pkg.name}</h1>
          <span className={`shrink-0 mt-1 rounded-md px-1.5 py-0.5 text-[10px] font-medium border ${colors.bg} ${colors.text} ${colors.border}`}>
            {pkg.scope}
          </span>
          {pkg.scope === 'unofficial' && !pkg.claimedBy && (
            <span className="shrink-0 mt-1 flex items-center gap-1 rounded-md bg-amber-500/10 border border-amber-500/20 px-1.5 py-0.5">
              <ShieldAlert className="size-3 text-amber-500/70" />
              <span className="text-[10px] font-medium text-amber-500/70">Unverified</span>
            </span>
          )}
          {pkg.claimedBy && pkg.claimedBy === session?.user?.id && (
            <span className="shrink-0 mt-1 flex items-center gap-1 rounded-md bg-emerald-500/10 border border-emerald-500/20 px-1.5 py-0.5">
              <Check className="size-3 text-emerald-500" />
              <span className="text-[10px] font-medium text-emerald-400">Claimed by you</span>
            </span>
          )}
        </div>
        <p className="text-[11px] text-muted-foreground leading-relaxed mb-3">
          {pkg.description}
        </p>

        {/* Meta row */}
        <div className="flex flex-wrap items-center gap-3">
          <a
            href={pkg.repoUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors no-underline"
          >
            <ExternalLink className="size-3" />
            {pkg.repoUrl.replace('https://', '')}
          </a>
          <div className="flex items-center gap-1 text-muted-foreground/60">
            <Download className="size-3" />
            <span className="text-[11px]">{pkg.installs.toLocaleString()} installs</span>
          </div>
          {pkg.latestVersion && (
            <span className="rounded-md bg-muted/50 px-1.5 py-0.5 text-[10px] font-mono text-muted-foreground">
              v{pkg.latestVersion}
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

        {pkg.scope === 'unofficial' && !pkg.claimedBy && (
          <button
            onClick={onClaimClick}
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
    </>
  )
}
