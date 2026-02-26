import { ReactNode } from 'react';
import { X } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';

interface DetailSheetProps {
  label: string;
  title: ReactNode;
  meta?: ReactNode;
  onClose: () => void;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

export default function DetailSheet({
  label,
  title,
  meta,
  onClose,
  children,
  footer,
  className,
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
        <header className="border-b px-4 py-3 md:px-5 md:py-4">
          <div className="mb-2 flex items-start justify-between gap-3">
            <Badge variant="outline" className="text-[10px] uppercase tracking-wide">
              {label}
            </Badge>
            <Button variant="ghost" size="icon-sm" onClick={onClose} title="Close panel">
              <X className="size-4" />
            </Button>
          </div>
          <div className="space-y-1">
            {title}
            {meta}
          </div>
        </header>

        <div className="flex-1 overflow-auto px-4 py-4 md:px-5 md:py-5">{children}</div>

        {footer && <footer className="border-t px-4 py-3 md:px-5 md:py-4">{footer}</footer>}
      </section>
    </div>
  );
}
