import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';

type PageWidth = 'narrow' | 'default' | 'wide';

interface PageFrameProps {
  children: ReactNode;
  className?: string;
  width?: PageWidth;
}

interface PageHeaderProps {
  title: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  eyebrow?: ReactNode;
  badge?: ReactNode;
  footer?: ReactNode;
  className?: string;
}

export function PageFrame({ children, className, width = 'default' }: PageFrameProps) {
  return (
    <div
      className={cn(
        'mx-auto flex w-full flex-col gap-5 p-5 md:p-6',
        width === 'wide' ? 'max-w-[1550px]' : width === 'narrow' ? 'max-w-5xl' : 'max-w-6xl',
        className
      )}
    >
      {children}
    </div>
  );
}

export function PageHeader({
  title,
  description,
  actions,
  eyebrow,
  badge,
  footer,
  className,
}: PageHeaderProps) {
  return (
    <header
      className={cn(
        'relative overflow-hidden rounded-xl border border-primary/20 bg-gradient-to-br from-primary/12 via-card to-card p-4 md:p-5',
        className
      )}
    >
      <div className="pointer-events-none absolute -right-20 -top-20 size-48 rounded-full bg-accent/12 blur-3xl" />
      <div className="relative space-y-3">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="space-y-1">
            {eyebrow && (
              <p className="text-muted-foreground text-[11px] font-medium uppercase tracking-widest">{eyebrow}</p>
            )}
            <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
            {description && <p className="text-muted-foreground text-sm">{description}</p>}
          </div>
          {(badge || actions) && (
            <div className="flex shrink-0 flex-wrap items-center gap-2">
              {badge}
              {actions}
            </div>
          )}
        </div>
        {footer && <div className="relative">{footer}</div>}
      </div>
    </header>
  );
}
