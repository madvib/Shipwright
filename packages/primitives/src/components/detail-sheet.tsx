import { ReactNode } from 'react';
import { X } from 'lucide-react';
import { cn } from '../lib/utils';
import { Badge } from './badge';
import { Button } from './button';

interface DetailSheetProps {
  label?: ReactNode;
  title: ReactNode;
  meta?: ReactNode;
  onClose: () => void;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
  headerClassName?: string;
  footerClassName?: string;
  bodyClassName?: string;
  bodyScrollable?: boolean;
  showHeader?: boolean;
  inlineHeader?: boolean;
}

export function DetailSheet({
  label,
  title,
  meta,
  onClose,
  children,
  footer,
  className,
  headerClassName,
  footerClassName,
  bodyClassName,
  bodyScrollable = true,
  showHeader = true,
  inlineHeader = false,
}: DetailSheetProps) {
  return (
    <div
      className="fixed inset-0 z-50 bg-black/45 p-3 supports-backdrop-filter:backdrop-blur-xs md:p-6"
      onClick={onClose}
    >
      <section
        className={cn(
          'bg-background mx-auto flex h-full w-full max-w-[1200px] flex-col overflow-hidden rounded-2xl border shadow-2xl',
          className
        )}
        onClick={(event) => event.stopPropagation()}
      >
        {showHeader && (
          <header className={cn('border-b px-4 py-2 md:px-5 md:py-2.5', headerClassName)}>
            {inlineHeader ? (
              <div className="flex items-center gap-3">
                <div className="min-w-0 flex-1">
                  {title}
                </div>
                <Button variant="ghost" size="icon-sm" onClick={onClose} title="Close panel">
                  <X className="size-4" />
                </Button>
              </div>
            ) : (
              <>
                <div
                  className={cn(
                    'flex items-start gap-3',
                    title || meta ? 'mb-2' : 'mb-0',
                    label ? 'justify-between' : 'justify-end'
                  )}
                >
                  {label && (
                    <Badge variant="outline" className="text-[10px] uppercase tracking-wide">
                      {label}
                    </Badge>
                  )}
                  <Button variant="ghost" size="icon-sm" onClick={onClose} title="Close panel">
                    <X className="size-4" />
                  </Button>
                </div>
                <div className="space-y-1">
                  {title}
                  {meta}
                </div>
              </>
            )}
          </header>
        )}

        <div
          className={cn(
            'flex-1 px-4 py-3 md:px-5 md:py-4',
            bodyScrollable ? 'overflow-auto' : 'overflow-hidden',
            bodyClassName
          )}
        >
          {children}
        </div>

        {footer && <footer className={cn('border-t px-4 py-3 md:px-5 md:py-4', footerClassName)}>{footer}</footer>}
      </section>
    </div>
  );
}
