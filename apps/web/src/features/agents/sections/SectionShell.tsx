import { Plus } from 'lucide-react'

// ── Orange Dot ──────────────────────────────────────────────────────────────
// Small indicator for buttons/actions without a working backend yet.
export function OrangeDot() {
  return <span className="size-2 shrink-0 rounded-full bg-primary" />
}

// ── Section wrapper ─────────────────────────────────────────────────────────

interface SectionShellProps {
  icon: React.ReactNode
  title: string
  count?: string
  actionLabel?: string
  onAction?: () => void
  showOrangeDot?: boolean
  children: React.ReactNode
}

export function SectionShell({
  icon,
  title,
  count,
  actionLabel,
  onAction,
  showOrangeDot,
  children,
}: SectionShellProps) {
  return (
    <div className="border-b border-border/40 py-5 px-5">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2 text-[13px] font-semibold text-muted-foreground">
          <span className="text-muted-foreground/60">{icon}</span>
          {title}
          {count && (
            <span className="text-[11px] font-normal text-muted-foreground/50">
              {count}
            </span>
          )}
        </div>
        {actionLabel && (
          <button
            onClick={onAction}
            className="flex items-center gap-1 rounded-md border border-border/60 px-2.5 py-1 text-[11px] text-muted-foreground hover:border-primary hover:text-primary transition-colors"
          >
            <Plus className="size-3" />
            {actionLabel}
            {showOrangeDot && <OrangeDot />}
          </button>
        )}
      </div>
      {children}
    </div>
  )
}

// ── Chip grid ───────────────────────────────────────────────────────────────

interface ChipProps {
  icon: React.ReactNode
  name: string
  meta: string
  badge?: React.ReactNode
  onRemove?: () => void
  onClick?: () => void
  active?: boolean
  ariaExpanded?: boolean
}

export function Chip({
  icon,
  name,
  meta,
  badge,
  onRemove,
  onClick,
  active,
  ariaExpanded,
}: ChipProps) {
  return (
    <div
      className={`flex min-w-[180px] items-center gap-2 rounded-lg border px-3 py-2 transition-colors ${
        active
          ? 'border-primary bg-primary/5'
          : 'border-border/60 bg-card/50 hover:border-border'
      } ${onClick ? 'cursor-pointer' : ''}`}
      onClick={onClick}
      {...(onClick ? { role: 'button', 'aria-expanded': ariaExpanded } : {})}
    >
      <div className="shrink-0">{icon}</div>
      <div className="flex-1 min-w-0">
        <div className="text-xs font-medium text-foreground truncate">{name}</div>
        <div className="text-[10px] text-muted-foreground/60 mt-0.5 truncate">{meta}</div>
      </div>
      {badge}
      {onRemove && (
        <button
          onClick={(e) => { e.stopPropagation(); onRemove() }}
          aria-label="Remove"
          className="shrink-0 text-muted-foreground/30 hover:text-destructive transition-colors text-sm"
        >
          x
        </button>
      )}
    </div>
  )
}

export function ChipIcon({
  letters,
  variant,
}: {
  letters: string
  variant: 'skill' | 'mcp' | 'agent'
}) {
  const colors = {
    skill: 'bg-primary/10 text-primary',
    mcp: 'bg-blue-500/10 text-blue-500 dark:text-blue-400',
    agent: 'bg-violet-500/10 text-violet-500 dark:text-violet-400',
  }

  return (
    <div
      className={`flex size-7 items-center justify-center rounded-lg text-[11px] font-semibold ${colors[variant]}`}
    >
      {letters}
    </div>
  )
}

export function AddChip({
  label,
  onClick,
  showOrangeDot,
}: {
  label: string
  onClick?: () => void
  showOrangeDot?: boolean
}) {
  return (
    <button
      onClick={onClick}
      className="flex min-w-[120px] items-center justify-center gap-1.5 rounded-lg border border-dashed border-border/60 px-3 py-2 text-[11px] text-muted-foreground/50 hover:border-primary hover:text-primary transition-colors"
    >
      <Plus className="size-3" />
      {label}
      {showOrangeDot && <OrangeDot />}
    </button>
  )
}
