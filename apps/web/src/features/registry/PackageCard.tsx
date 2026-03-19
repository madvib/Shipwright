import { Link } from '@tanstack/react-router'
import { Download, ShieldAlert } from 'lucide-react'
import type { RegistryPackage } from './types'
import { SCOPE_COLORS } from './types'

function formatInstalls(n: number): string {
  if (n >= 10_000) return `${(n / 1000).toFixed(1)}k`
  if (n >= 1_000) return `${(n / 1000).toFixed(1)}k`
  return String(n)
}

interface PackageCardProps {
  pkg: RegistryPackage
}

export function PackageCard({ pkg }: PackageCardProps) {
  const colors = SCOPE_COLORS[pkg.scope]
  const linkPath = `/studio/registry/${encodeURIComponent(pkg.path)}`

  return (
    <Link
      to={linkPath as '/'}
      className="group relative flex flex-col rounded-xl border border-border/60 bg-card p-4 transition-all duration-200 hover:border-border hover:shadow-md hover:shadow-foreground/[0.03] no-underline"
    >
      {/* Top row: name + scope badge */}
      <div className="flex items-start justify-between gap-2 mb-2">
        <h3 className="text-sm font-semibold text-foreground leading-tight line-clamp-1 group-hover:text-primary transition-colors">
          {pkg.name}
        </h3>
        <span className={`shrink-0 rounded-md px-1.5 py-0.5 text-[10px] font-medium ${colors.bg} ${colors.text} ${colors.border} border`}>
          {pkg.scope}
        </span>
      </div>

      {/* Description */}
      <p className="text-[11px] text-muted-foreground leading-relaxed line-clamp-2 mb-3 flex-1">
        {pkg.description}
      </p>

      {/* Unverified badge for unofficial */}
      {pkg.scope === 'unofficial' && (
        <div className="flex items-center gap-1 mb-2">
          <ShieldAlert className="size-3 text-amber-500/70" />
          <span className="text-[10px] text-amber-500/70 font-medium">Unverified</span>
        </div>
      )}

      {/* Bottom row: version + installs */}
      <div className="flex items-center justify-between mt-auto pt-2 border-t border-border/30">
        {pkg.latest_version ? (
          <span className="rounded-md bg-muted/50 px-1.5 py-0.5 text-[10px] font-mono text-muted-foreground">
            v{pkg.latest_version}
          </span>
        ) : (
          <span className="text-[10px] text-muted-foreground/40">no release</span>
        )}
        <div className="flex items-center gap-1 text-muted-foreground/60">
          <Download className="size-3" />
          <span className="text-[10px]">{formatInstalls(pkg.installs)}</span>
        </div>
      </div>

      {/* Path — subtle, below the fold */}
      <p className="mt-2 text-[10px] font-mono text-muted-foreground/30 truncate">
        {pkg.path}
      </p>
    </Link>
  )
}
