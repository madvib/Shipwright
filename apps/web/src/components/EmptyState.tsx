import type { ReactNode } from 'react'
import { Button } from '@ship/primitives'

interface EmptyStateAction {
  label: string
  icon?: ReactNode
  onClick?: () => void
  href?: string
  variant?: 'default' | 'outline'
}

interface EmptyStateProps {
  icon: ReactNode
  title: string
  description: string
  primaryAction?: EmptyStateAction
  secondaryAction?: EmptyStateAction
  children?: ReactNode
  className?: string
}

export function EmptyState({
  icon,
  title,
  description,
  primaryAction,
  secondaryAction,
  children,
  className = '',
}: EmptyStateProps) {
  return (
    <div
      className={`flex flex-col items-center justify-center px-5 py-10 text-center ${className}`}
    >
      <div className="mb-5 flex size-16 items-center justify-center rounded-2xl border border-border/60 bg-muted/40">
        <div className="text-muted-foreground">{icon}</div>
      </div>

      <h2 className="font-display text-xl font-bold text-foreground">
        {title}
      </h2>
      <p className="mt-1.5 max-w-[400px] text-[13px] leading-relaxed text-muted-foreground">
        {description}
      </p>

      {(primaryAction || secondaryAction) && (
        <div className="mt-6 flex items-center gap-2.5">
          {primaryAction && (
            <Button
              size="default"
              variant="default"
              onClick={primaryAction.onClick}
            >
              {primaryAction.icon}
              {primaryAction.label}
            </Button>
          )}
          {secondaryAction && (
            <Button
              size="default"
              variant="outline"
              onClick={secondaryAction.onClick}
            >
              {secondaryAction.icon}
              {secondaryAction.label}
            </Button>
          )}
        </div>
      )}

      {children && <div className="mt-8 w-full max-w-[560px]">{children}</div>}
    </div>
  )
}

interface QuickStartCardProps {
  icon: ReactNode
  iconColorClass: string
  title: string
  description: string
  onClick?: () => void
  orangeDot?: boolean
}

export function QuickStartCard({
  icon,
  iconColorClass,
  title,
  description,
  onClick,
  orangeDot,
}: QuickStartCardProps) {
  return (
    <button
      onClick={onClick}
      className="group flex flex-col items-start rounded-xl border border-border/60 bg-card p-4 text-left transition hover:border-border hover:-translate-y-px hover:shadow-sm"
    >
      <div
        className={`mb-2 flex size-8 items-center justify-center rounded-lg ${iconColorClass}`}
      >
        {icon}
      </div>
      <div className="flex items-center gap-1.5">
        <span className="text-xs font-semibold text-foreground">{title}</span>
        {orangeDot && (
          <span className="size-2 rounded-full bg-primary" />
        )}
      </div>
      <span className="mt-0.5 text-[10px] leading-snug text-muted-foreground">
        {description}
      </span>
    </button>
  )
}

export function QuickStartGrid({
  children,
}: {
  children: ReactNode
}) {
  return (
    <div className="grid grid-cols-3 gap-2.5">{children}</div>
  )
}
