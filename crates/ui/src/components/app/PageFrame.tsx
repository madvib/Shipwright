import { createContext, useContext, type ReactNode } from 'react';
import { cn } from '@/lib/utils';

type PageWidth = 'narrow' | 'default' | 'wide';

interface PageChromeContextValue {
  breadcrumb?: ReactNode;
}

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
  showGlobalChrome?: boolean;
  className?: string;
}

const PageChromeContext = createContext<PageChromeContextValue | null>(null);

export function PageChromeProvider({
  value,
  children,
}: {
  value: PageChromeContextValue | null;
  children: ReactNode;
}) {
  return <PageChromeContext.Provider value={value}>{children}</PageChromeContext.Provider>;
}

export function PageFrame({ children, className, width = 'default' }: PageFrameProps) {
  return (
    <div
      className={cn(
        'mx-auto flex w-full flex-col gap-3 p-4 md:p-4',
        width === 'wide'
          ? 'max-w-[min(92vw,1840px)]'
          : width === 'narrow'
            ? 'max-w-[min(78vw,1280px)]'
            : 'max-w-[min(86vw,1560px)]',
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
  showGlobalChrome = true,
  className,
}: PageHeaderProps) {
  const chrome = useContext(PageChromeContext);
  const renderGlobalChrome =
    showGlobalChrome && !!chrome && !!chrome.breadcrumb;

  return (
    <header
      className={cn(
        'rounded-lg border border-primary/15 bg-gradient-to-r from-primary/8 via-card/80 to-card/70 px-3 py-2.5 md:px-3.5 md:py-3',
        className
      )}
    >
      {renderGlobalChrome ? (
        <div className="space-y-2">
          <h1 className="sr-only">{title}</h1>
          <div className="flex items-center justify-between gap-2">
            <div className="min-w-0 flex-1 overflow-hidden">{chrome?.breadcrumb}</div>
            {(badge || actions) && (
              <div className="flex shrink-0 flex-wrap items-center justify-end gap-1.5">
                {badge}
                {actions}
              </div>
            )}
          </div>
          {(eyebrow || description) && (
            <div className="space-y-1">
              {eyebrow && (
                <p className="text-muted-foreground text-[10px] font-medium uppercase tracking-widest">{eyebrow}</p>
              )}
              {description && <p className="text-muted-foreground text-xs leading-snug md:text-sm">{description}</p>}
            </div>
          )}
          {footer && <div>{footer}</div>}
        </div>
      ) : (
        <div className="space-y-2">
          <div className="flex flex-wrap items-start justify-between gap-2">
            <div className="space-y-1">
              {eyebrow && (
                <p className="text-muted-foreground text-[10px] font-medium uppercase tracking-widest">{eyebrow}</p>
              )}
              <h1 className="text-lg font-semibold tracking-tight md:text-xl">{title}</h1>
              {description && <p className="text-muted-foreground text-xs leading-snug md:text-sm">{description}</p>}
            </div>
            {(badge || actions) && (
              <div className="flex shrink-0 flex-wrap items-center gap-1.5">
                {badge}
                {actions}
              </div>
            )}
          </div>
          {footer && <div>{footer}</div>}
        </div>
      )}
    </header>
  );
}
