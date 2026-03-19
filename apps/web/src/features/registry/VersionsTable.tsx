import { Tag } from 'lucide-react'
import type { PackageVersion } from './types'

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    })
  } catch {
    return iso
  }
}

interface VersionsTableProps {
  versions: PackageVersion[]
}

export function VersionsTable({ versions }: VersionsTableProps) {
  if (versions.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-10 text-center">
        <Tag className="size-5 text-muted-foreground/30 mb-2" />
        <p className="text-xs text-muted-foreground">No versions indexed yet.</p>
      </div>
    )
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left">
        <thead>
          <tr className="border-b border-border/30">
            <th className="py-2 pr-4 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Version</th>
            <th className="py-2 pr-4 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Tag</th>
            <th className="py-2 pr-4 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Commit</th>
            <th className="py-2 pr-4 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Skills</th>
            <th className="py-2 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">Date</th>
          </tr>
        </thead>
        <tbody>
          {versions.map((v, i) => (
            <tr key={v.id} className={`border-b border-border/20 ${i === 0 ? 'bg-primary/[0.03]' : ''}`}>
              <td className="py-2.5 pr-4">
                <span className="rounded-md bg-muted/50 px-1.5 py-0.5 text-[11px] font-mono text-foreground">
                  {v.version}
                </span>
                {i === 0 && (
                  <span className="ml-1.5 rounded bg-emerald-500/10 px-1 py-0.5 text-[9px] font-medium text-emerald-400">
                    latest
                  </span>
                )}
              </td>
              <td className="py-2.5 pr-4 text-[11px] font-mono text-muted-foreground">{v.git_tag}</td>
              <td className="py-2.5 pr-4">
                <span className="text-[11px] font-mono text-muted-foreground/60">{v.commit_sha.slice(0, 7)}</span>
              </td>
              <td className="py-2.5 pr-4 text-[11px] text-muted-foreground">{v.skills.length}</td>
              <td className="py-2.5 text-[11px] text-muted-foreground/60">{formatDate(v.indexed_at)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
