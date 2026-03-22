import { Users, Eye, Palette, Server, Search } from 'lucide-react'
import { AGENT_TEMPLATES } from '#/features/agents/agent-templates'
import type { AgentTemplate } from '#/features/agents/agent-templates'

const ICON_MAP: Record<string, typeof Users> = {
  Users, Eye, Palette, Server, Search,
}

export function TemplateGrid({ onSelect, compact }: {
  onSelect: (t: AgentTemplate) => void
  compact?: boolean
}) {
  return (
    <div className={compact
      ? 'grid grid-cols-2 lg:grid-cols-5 gap-2'
      : 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3'
    }>
      {AGENT_TEMPLATES.map((t, i) => (
        <TemplateCard
          key={t.id}
          template={t}
          onSelect={onSelect}
          index={i}
          compact={compact}
        />
      ))}
    </div>
  )
}

function TemplateCard({ template, onSelect, index, compact }: {
  template: AgentTemplate
  onSelect: (t: AgentTemplate) => void
  index: number
  compact?: boolean
}) {
  const Icon = ICON_MAP[template.icon] ?? Users
  const [colorText, colorBg] = template.color.split(' ')

  if (compact) {
    return (
      <button
        onClick={() => onSelect(template)}
        className="group flex items-center gap-2.5 rounded-xl border border-border/60 bg-card px-3 py-2.5 text-left transition hover:border-primary/30 hover:shadow-sm"
        style={{ animationDelay: `${index * 40}ms` }}
      >
        <div className={`size-7 shrink-0 rounded-lg ${colorBg} flex items-center justify-center`}>
          <Icon className={`size-3.5 ${colorText}`} />
        </div>
        <span className="text-xs font-semibold text-foreground truncate">{template.name}</span>
      </button>
    )
  }

  return (
    <button
      onClick={() => onSelect(template)}
      className="group relative flex flex-col rounded-xl border border-border/60 bg-card p-5 text-left transition-all hover:border-primary/30 hover:shadow-md hover:-translate-y-0.5"
      style={{ animationDelay: `${index * 60}ms` }}
    >
      <div className={`size-10 rounded-xl ${colorBg} flex items-center justify-center mb-3 transition-transform group-hover:scale-110`}>
        <Icon className={`size-5 ${colorText}`} />
      </div>
      <h3 className="font-display text-sm font-semibold text-foreground mb-1">{template.name}</h3>
      <p className="text-xs text-muted-foreground leading-relaxed flex-1 mb-3">{template.description}</p>
      <div className="flex items-center gap-1.5 flex-wrap">
        {template.providers.map((p) => (
          <span key={p} className="inline-block rounded-md border border-border/40 bg-muted/50 px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground capitalize">
            {p}
          </span>
        ))}
        <span className="inline-block rounded-md border border-border/40 bg-muted/50 px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground">
          {template.permissionPreset.replace('ship-', '')}
        </span>
      </div>
      <div className="absolute inset-x-0 bottom-0 h-px bg-gradient-to-r from-transparent via-primary/0 to-transparent group-hover:via-primary/20 transition-all" />
    </button>
  )
}
