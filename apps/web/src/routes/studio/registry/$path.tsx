import { createFileRoute, Link } from '@tanstack/react-router'
import { useState, type ReactNode } from 'react'
import { ArrowLeft, ExternalLink, Download, Copy, Check, AlertTriangle, ShieldAlert } from 'lucide-react'
import { usePackageDetail } from '#/features/registry/useRegistry'
import { SCOPE_COLORS } from '#/features/registry/types'
import { SkillsList } from '#/features/registry/SkillsList'
import { VersionsTable } from '#/features/registry/VersionsTable'

export const Route = createFileRoute('/studio/registry/$path')({ component: PackageDetailPage })

type DetailTab = 'skills' | 'versions'

function PackageDetailPage() {
  const { path } = Route.useParams()
  const decodedPath = decodeURIComponent(path)
  const { data, isLoading } = usePackageDetail(decodedPath)
  const [tab, setTab] = useState<DetailTab>('skills')
  const [copied, setCopied] = useState(false)

  const installCmd = `ship add ${decodedPath}`

  function handleCopy() {
    void navigator.clipboard.writeText(installCmd)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
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

        {/* Add to project button */}
        <div className="mb-6">
          <button className="rounded-lg bg-violet-600 dark:bg-violet-500 hover:bg-violet-500 dark:hover:bg-violet-400 transition-colors px-5 py-2.5 text-xs font-semibold text-primary-foreground">
            Add to project
          </button>
          {pkg.scope === 'unofficial' && !pkg.claimed_by && (
            <button className="ml-3 rounded-lg border border-amber-500/30 bg-amber-500/5 px-4 py-2.5 text-xs font-medium text-amber-400 transition hover:bg-amber-500/10">
              Claim this package
            </button>
          )}
        </div>

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
