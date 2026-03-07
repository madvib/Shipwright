import { Search, RefreshCw, ChevronDown, Home } from 'lucide-react';
import { Button, Input, Badge, Tooltip, TooltipTrigger, TooltipContent } from '@ship/ui';
import { EditorQuickOpenMenu } from './IDEComponents';
import { type WorkspaceEditorInfo } from '@/lib/platform/tauri/commands';
import { WorkspaceRow } from './types';
import { WorkspaceGraphStatus } from '../components/WorkspaceLifecycleGraph';

interface WorkspaceSidebarProps {
    filteredRows: WorkspaceRow[];
    selectedBranch: string | null;
    onSelectBranch: (branch: string) => void;
    availableEditors: WorkspaceEditorInfo[];
    isDarkTheme: boolean;
    onOpenEditor: (branch: string, editorId: string) => void;
    searchQuery: string;
    onSearchChange: (query: string) => void;
    loading: boolean;
    onRefresh: () => void;
    onHome: () => void;
    onCollapse: () => void;
    statusVariant: (status: WorkspaceGraphStatus) => 'default' | 'secondary' | 'outline';
}

export function WorkspaceSidebar({
    filteredRows,
    selectedBranch,
    onSelectBranch,
    availableEditors,
    isDarkTheme,
    onOpenEditor,
    searchQuery,
    onSearchChange,
    loading,
    onRefresh,
    onHome,
    onCollapse,
    statusVariant,
}: WorkspaceSidebarProps) {
    return (
        <div className="flex flex-col h-full bg-sidebar border-r border-sidebar-border overflow-hidden">
            {/* Header */}
            <div className="flex h-14 shrink-0 items-center justify-between gap-3 border-b border-border/50 px-4">
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

                <div className="flex flex-1 min-w-0 items-center gap-2 px-2">
                    <div className="size-2 rounded-full bg-primary" />
                    <h2 className="text-[11px] font-bold uppercase tracking-widest text-foreground truncate">Workspaces</h2>
                </div>

                <div className="flex items-center gap-1">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button
                                size="icon-xs"
                                variant="ghost"
                                className="size-8 text-muted-foreground hover:text-foreground"
                                onClick={onCollapse}
                            >
                                <ChevronDown className="size-4 rotate-90" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="bottom">Collapse Sidebar (⌘B)</TooltipContent>
                    </Tooltip>

                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button variant="ghost" size="icon-xs" className="size-8 text-muted-foreground hover:text-foreground" onClick={onRefresh}>
                                <RefreshCw className={loading ? "size-3 animate-spin" : "size-3"} />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="bottom">Refresh Workspaces</TooltipContent>
                    </Tooltip>
                </div>
            </div>

            {/* Search */}
            <div className="flex h-12 shrink-0 items-center border-b border-border px-4 transition-all focus-within:bg-muted/10">
                <div className="relative w-full overflow-hidden">
                    <Search className="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground/40 transition-colors group-focus-within:text-primary" />
                    <Input
                        value={searchQuery}
                        onChange={(e) => onSearchChange(e.target.value)}
                        placeholder="Filter list..."
                        className="h-8 border-none bg-transparent pl-8 text-[11px] font-bold text-foreground placeholder:font-medium placeholder:text-muted-foreground/40 focus-visible:ring-0"
                    />
                </div>
            </div>

            <div className="flex h-full min-h-0 flex-col rounded-lg border border-border/70 bg-card/60 shadow-sm m-2">
                <div className="flex items-center justify-between border-b border-border/50 px-4 py-3 bg-muted/20">
                    <h3 className="text-[11px] font-black uppercase tracking-widest text-muted-foreground">Workspace Roster</h3>
                    <span className="text-[10px] font-black tabular-nums text-muted-foreground/50">{filteredRows.length}</span>
                </div>
                <div className="min-h-0 flex-1 overflow-y-auto p-2.5 custom-scrollbar">
                    <div className="space-y-2">
                        {filteredRows.map((row) => {
                            const selected = row.branch === selectedBranch;
                            return (
                                <div
                                    key={row.id}
                                    role="button"
                                    tabIndex={0}
                                    className={`group/ws-item w-full rounded-xl border px-3 py-2.5 text-left transition-all ${selected
                                        ? 'border-primary/50 bg-primary/10 shadow-inner'
                                        : 'border-border/40 bg-card/40 hover:bg-muted/40 hover:border-border/60'
                                        }`}
                                    onClick={() => onSelectBranch(row.branch)}
                                    onKeyDown={(event) => {
                                        if (event.key === 'Enter' || event.key === ' ') {
                                            event.preventDefault();
                                            onSelectBranch(row.branch);
                                        }
                                    }}
                                >
                                    <div className="flex items-start justify-between gap-2">
                                        <span className={`truncate text-sm font-bold tracking-tight ${selected ? 'text-foreground' : 'text-muted-foreground group-hover/ws-item:text-foreground'}`}>{row.branch}</span>
                                        <div className="flex items-center gap-1.5">
                                            <EditorQuickOpenMenu
                                                branch={row.branch}
                                                editors={availableEditors}
                                                isDarkTheme={isDarkTheme}
                                                onOpenEditor={onOpenEditor}
                                            />
                                            <Badge variant={statusVariant(row.status)} className="h-5 px-1.5 text-[9px] font-black uppercase tracking-tighter shadow-none">
                                                {row.status}
                                            </Badge>
                                        </div>
                                    </div>
                                    <div className="mt-2 flex flex-wrap gap-1">
                                        <Badge variant="outline" className="h-4.5 px-1.5 text-[9px] border-border/40 bg-background/50 text-muted-foreground">
                                            {row.workspaceType}
                                        </Badge>
                                        <Badge variant="secondary" className="h-4.5 px-1.5 text-[9px]">
                                            {row.activeMode ?? 'default'}
                                        </Badge>
                                        <Badge variant="outline" className="h-4.5 px-1.5 text-[9px] border-border/40 bg-background/50 text-muted-foreground">
                                            {row.providers.length}
                                        </Badge>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                </div>
            </div>
        </div>
    );
}
