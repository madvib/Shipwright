import { Puzzle } from 'lucide-react'
import type { PackageSkill } from './types'

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
}

interface SkillsListProps {
  skills: PackageSkill[]
}

export function SkillsList({ skills }: SkillsListProps) {
  if (skills.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-10 text-center">
        <Puzzle className="size-5 text-muted-foreground/30 mb-2" />
        <p className="text-xs text-muted-foreground">No exported skills in this package.</p>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      {skills.map((skill) => (
        <div
          key={skill.id}
          className="rounded-lg border border-border/40 bg-card/50 p-3 flex items-start gap-3"
        >
          <div className="shrink-0 flex size-7 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-400">
            <Puzzle className="size-3.5" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2 mb-0.5">
              <p className="text-xs font-semibold text-foreground">{skill.name}</p>
              <span className="text-[10px] font-mono text-muted-foreground/40">{skill.skill_id}</span>
            </div>
            <p className="text-[11px] text-muted-foreground leading-relaxed">
              {skill.description}
            </p>
            <div className="flex items-center gap-3 mt-1.5">
              <span className="text-[10px] text-muted-foreground/40">{formatBytes(skill.content_length)}</span>
              <span className="text-[10px] font-mono text-muted-foreground/30 truncate max-w-[120px]" title={skill.content_hash}>
                {skill.content_hash.slice(0, 12)}
              </span>
            </div>
          </div>
        </div>
      ))}
    </div>
  )
}
