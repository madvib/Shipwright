import { createContext, useCallback, useContext, useEffect, useMemo, type ReactNode } from 'react';
import { cn } from '../lib/utils';

type PageWidth = 'narrow' | 'default' | 'wide' | 'full';

export interface PageChromeContextValue {
    breadcrumb?: ReactNode;
    sidebar?: ReactNode;
    onBack?: () => void;
    setChrome: (value: Partial<PageChromeContextValue>) => void;
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

export const PageChromeContext = createContext<PageChromeContextValue>({
    setChrome: () => { },
});

export function usePageChrome(value: Partial<PageChromeContextValue>) {
    const { setChrome } = useContext(PageChromeContext);
    useEffect(() => {
        setChrome(value);
        return () => setChrome({});
    }, [value, setChrome]);
}

export function PageChromeProvider({
    value,
    onUpdate,
    children,
}: {
    value: Partial<PageChromeContextValue>;
    onUpdate?: (value: Partial<PageChromeContextValue>) => void;
    children: ReactNode;
}) {
    const setChrome = useCallback((updates: Partial<PageChromeContextValue>) => {
        if (onUpdate) onUpdate(updates);
    }, [onUpdate]);

    const contextValue = useMemo(() => ({
        ...value,
        setChrome
    }), [value, setChrome]);

    return <PageChromeContext.Provider value={contextValue}>{children}</PageChromeContext.Provider>;
}

export function PageFrame({ children, className, width = 'default' }: PageFrameProps) {
    return (
        <div
            className={cn(
                'mx-auto flex w-full flex-col gap-5 p-5 md:p-6',
                width === 'full'
                    ? 'max-w-none'
                    : width === 'wide'
                        ? 'max-w-[min(94vw,1920px)]'
                        : width === 'narrow'
                            ? 'max-w-[min(78vw,1280px)]'
                            : 'max-w-[min(88vw,1640px)]',
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
    showGlobalChrome = false,
    className,
}: PageHeaderProps) {
    const chrome = useContext(PageChromeContext);
    const renderGlobalChrome =
        showGlobalChrome && !!chrome && !!chrome.breadcrumb;

    return (
        <header
            className={cn(
                'relative overflow-hidden rounded-lg border border-border',
                'bg-gradient-to-r from-primary/5 via-card to-card',
                'px-4 py-3 md:px-5 md:py-3.5',
                className
            )}
        >
            {renderGlobalChrome ? (
                <div className="space-y-2">
                    <h1 className="sr-only">{title}</h1>
                    <div className="flex items-center justify-between gap-3">
                        <div className="min-w-0 flex-1 overflow-hidden">{chrome?.breadcrumb}</div>
                        {(badge || actions) && (
                            <div className="flex shrink-0 flex-wrap items-center justify-end gap-2">
                                {badge}
                                {actions}
                            </div>
                        )}
                    </div>
                    {(eyebrow || description) && (
                        <div className="space-y-1">
                            {eyebrow && (
                                <p className="font-mono text-[10px] font-medium uppercase tracking-[0.15em] text-primary/70">{eyebrow}</p>
                            )}
                            {description && <p className="text-muted-foreground text-sm leading-snug">{description}</p>}
                        </div>
                    )}
                    {footer && <div className="mt-1">{footer}</div>}
                </div>
            ) : (
                <div className="space-y-2">
                    <div className="flex flex-wrap items-start justify-between gap-3">
                        <div className="space-y-1">
                            {eyebrow && (
                                <p className="font-mono text-[10px] font-medium uppercase tracking-[0.15em] text-primary/70">{eyebrow}</p>
                            )}
                            <h1 className="text-xl font-bold tracking-tight md:text-2xl">{title}</h1>
                            {description && <p className="text-muted-foreground text-sm leading-snug">{description}</p>}
                        </div>
                        {(badge || actions) && (
                            <div className="flex shrink-0 flex-wrap items-center gap-2">
                                {badge}
                                {actions}
                            </div>
                        )}
                    </div>
                    {footer && <div className="mt-2">{footer}</div>}
                </div>
            )}
        </header>
    );
}
