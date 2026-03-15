import { ReactNode } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@ship/primitives';
import { cn } from '@/lib/utils';

type ExplorerDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
};

export function ExplorerDialog({
  open,
  onOpenChange,
  title,
  icon,
  children,
  className,
}: ExplorerDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className={cn(
          'w-[min(1480px,calc(100vw-1rem))] h-[min(90vh,960px)] max-w-none overflow-hidden p-0 gap-0 flex flex-col',
          className
        )}
      >
        <DialogHeader className="shrink-0 border-b px-4 py-2.5">
          <DialogTitle className="flex items-center gap-2 text-sm font-semibold">
            {icon}
            {title}
          </DialogTitle>
        </DialogHeader>
        <div className="min-h-0 flex-1 overflow-y-auto px-4 py-3">
          {children}
        </div>
      </DialogContent>
    </Dialog>
  );
}
