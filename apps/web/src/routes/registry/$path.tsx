import { createFileRoute, Link } from '@tanstack/react-router'
import { useState, type ReactNode } from 'react'
import { ArrowLeft, AlertTriangle } from 'lucide-react'
import { useQueryClient } from '@tanstack/react-query'
import { usePackageDetail, registryKeys } from '#/features/registry/useRegistry'
import { SkillsList } from '#/features/registry/SkillsList'
import { VersionsTable } from '#/features/registry/VersionsTable'
import { ClaimDialog } from '#/features/registry/ClaimDialog'
import { PackageHeader } from '#/features/registry/PackageHeader'

export const Route = createFileRoute('/registry/$path')({ component: PackageDetailPage })

type DetailTab = 'skills' | 'versions'

function PackageDetailPage() {
  const { path } = Route.useParams()
  const decodedPath = decodeURIComponent(path)
  const { data, isLoading, error, refetch } = usePackageDetail(decodedPath)
  const [tab, setTab] = useState<DetailTab>('skills')
  const [claimOpen, setClaimOpen] = useState(false)
  const queryClient = useQueryClient()

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

  if (error) {
    return (
      <div className="h-full flex flex-col items-center justify-center gap-3 p-5">
        <div className="flex size-12 items-center justify-center rounded-2xl border border-destructive/20 bg-destructive/5 text-destructive mb-1">
          <AlertTriangle className="size-5" />
        </div>
        <p className="text-sm font-medium text-foreground">Failed to load package</p>
        <p className="text-[11px] text-muted-foreground max-w-xs text-center">
          {error.message || 'An unexpected error occurred.'}
        </p>
        <div className="flex items-center gap-2">
          <button
            onClick={() => void refetch()}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
          >
            Retry
          </button>
          <Link
            to="/registry"
            className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
          >
            <ArrowLeft className="size-3" />
            Back to registry
          </Link>
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
          to="/registry"
          className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
        >
          <ArrowLeft className="size-3" />
          Back to registry
        </Link>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-auto p-5 pb-20">
        <PackageHeader
          pkg={data.package}
          skills={data.skills}
          decodedPath={decodedPath}
          onClaimClick={() => setClaimOpen(true)}
        />

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
        repoUrl={data.package.repo_url}
        onClose={() => setClaimOpen(false)}
        onClaimed={() => void queryClient.invalidateQueries({ queryKey: registryKeys.detail(decodedPath) })}
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
