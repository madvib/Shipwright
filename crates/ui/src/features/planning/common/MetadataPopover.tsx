import { ReactNode } from 'react';
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
    Button,
} from '@ship/ui';
import { LucideIcon } from 'lucide-react';

interface MetadataPopoverProps {
    icon: LucideIcon;
    label: string;
    title: string;
    children: ReactNode;
    action?: ReactNode;
    align?: 'start' | 'center' | 'end';
    triggerClassName?: string;
    contentClassName?: string;
}

/**
 * A reusable pattern for "Icon + Label -> Popover with Header and Content".
 * Common in ADR and Feature metadata headers.
 */
export function MetadataPopover({
    icon: Icon,
    label,
    title,
    children,
    action,
    align = 'start',
    triggerClassName = 'max-w-[200px]',
    contentClassName = 'w-80 p-2',
}: MetadataPopoverProps) {
    return (
        <Popover>
            <PopoverTrigger
                render={
                    <Button
                        variant="ghost"
                        size="xs"
                        className={`h-7 gap-1.5 px-2 text-xs font-normal text-muted-foreground hover:bg-accent/50 ${triggerClassName}`}
                    >
                        <Icon className="size-3.5 shrink-0" />
                        <span className="truncate">{label}</span>
                    </Button>
                }
            />
            <PopoverContent align={align} className={contentClassName} sideOffset={8}>
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground">{title}</p>
                        {action}
                    </div>
                    {children}
                </div>
            </PopoverContent>
        </Popover>
    );
}
