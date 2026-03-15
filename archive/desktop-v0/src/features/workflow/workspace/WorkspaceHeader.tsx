import { ChevronDown, Home } from 'lucide-react';
import { Button, Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';

interface WorkspaceHeaderProps {
    branch: string;
    sidebarCollapsed: boolean;
    onHome: () => void;
    onExpandSidebar: () => void;
    actions?: React.ReactNode;
}

export function WorkspaceHeader({
    branch,
    sidebarCollapsed,
    onHome,
    onExpandSidebar,
    actions,
}: WorkspaceHeaderProps) {
    return (
        <header className="flex h-14 shrink-0 items-center justify-between border-b border-sidebar-border bg-sidebar/95 backdrop-blur-md px-6 z-10 shadow-sm">
            <div className="flex items-center gap-3 min-w-0">
                {sidebarCollapsed && (
                    <div className="flex items-center gap-1 mr-1">
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="icon-xs"
                                    className="size-8 text-muted-foreground hover:text-foreground"
                                    onClick={onHome}
                                >
                                    <Home className="size-4" />
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent side="right">Back to Project Overview</TooltipContent>
                        </Tooltip>

                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    size="icon-xs"
                                    variant="ghost"
                                    className="size-8 text-muted-foreground hover:text-foreground transition-all"
                                    onClick={onExpandSidebar}
                                >
                                    <ChevronDown className="size-4 -rotate-90" />
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent side="bottom">Expand Sidebar (⌘B)</TooltipContent>
                        </Tooltip>
                        <div className="h-4 w-px bg-border/40 mx-1" />
                    </div>
                )}
                <div className="min-w-0">
                    <div className="flex items-center gap-3">
                        <h1 className="truncate text-lg font-bold tracking-tight text-foreground">{branch}</h1>
                    </div>
                </div>
            </div>
            <div className="flex items-center gap-2">
                {actions}
            </div>
        </header>
    );
}
