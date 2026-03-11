import { memo, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  Activity,
  ChevronDown,
  Filter,
  GitBranch,
  GitBranchPlus,
  Home,
  LayoutPanelTop,
  RefreshCw,
  Search,
} from 'lucide-react';
import {
  Badge,
  Button,
  Checkbox,
  Input,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Switch,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@ship/ui';
import { EditorQuickOpenMenu } from './IDEComponents';
import { type WorkspaceEditorInfo, type GitBranchInfo } from '@/lib/platform/tauri/commands';
import { WorkspaceRow } from './types';
import { cn } from '@/lib/utils';

type SidebarView = 'workspaces' | 'branches';
type WorkspaceTypeFilter = WorkspaceRow['workspaceType'];
type BranchManagedFilter = 'managed' | 'unmanaged';

interface WorkspaceRowItemProps {
  row: WorkspaceRow;
  title: string;
  subtitle: string;
  selected: boolean;
  hasActiveSession: boolean;
  availableEditors: WorkspaceEditorInfo[];
  isDarkTheme: boolean;
  onSelectBranch: (branch: string) => void;
  onOpenEditor: (branch: string, editorId: string) => void;
}

const WorkspaceRowItem = memo(function WorkspaceRowItem({
  row,
  title,
  subtitle,
  selected,
  hasActiveSession,
  availableEditors,
  isDarkTheme,
  onSelectBranch,
  onOpenEditor,
}: WorkspaceRowItemProps) {
  return (
    <div
      role="button"
      tabIndex={0}
      className={cn(
        'group/ws-item w-full rounded-xl border px-3 py-2.5 text-left transition-all',
        selected
          ? 'border-primary/50 bg-primary/10 shadow-inner'
          : 'border-border/40 bg-card/40 hover:border-border/60 hover:bg-muted/40',
      )}
      onClick={() => onSelectBranch(row.branch)}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          onSelectBranch(row.branch);
        }
      }}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <p className="text-sm font-semibold leading-tight text-foreground break-words">{title}</p>
          <p className="mt-0.5 truncate font-mono text-[10px] text-muted-foreground">{subtitle}</p>
        </div>

        <div className="flex items-center gap-1.5">
          <EditorQuickOpenMenu
            branch={row.branch}
            editors={availableEditors}
            isDarkTheme={isDarkTheme}
            onOpenEditor={onOpenEditor}
          />
          {row.status === 'archived' ? (
            <Badge variant="secondary" className="h-4.5 px-1.5 text-[9px] uppercase">
              archived
            </Badge>
          ) : null}
        </div>
      </div>

      <div className="mt-2 flex flex-wrap items-center gap-1.5">
        <Badge
          variant="outline"
          className="h-4.5 border-border/40 bg-background/50 px-1.5 text-[9px] text-muted-foreground"
        >
          {row.workspaceType}
        </Badge>
        {hasActiveSession && (
          <Badge className="h-4.5 bg-emerald-500/15 px-1.5 text-[9px] uppercase text-emerald-600 dark:text-emerald-300">
            session live
          </Badge>
        )}
      </div>
    </div>
  );
});

interface WorkspaceSidebarProps {
  rows: WorkspaceRow[];
  gitBranches: GitBranchInfo[];
  activeSessionBranches: string[];
  selectedBranch: string | null;
  onSelectBranch: (branch: string) => void;
  onConfigureBranch: (branch: string) => void;
  availableEditors: WorkspaceEditorInfo[];
  isDarkTheme: boolean;
  onOpenEditor: (branch: string, editorId: string) => void;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  loading: boolean;
  onRefresh: () => void;
  onHome: () => void;
  onCollapse: () => void;
  featureLabels: Record<string, string>;
}

function humanizeBranchToken(value: string): string {
  const normalized = value
    .replace(/^\w+\//, '')
    .replace(/[-_]+/g, ' ')
    .trim();
  if (!normalized) return value;
  return normalized.replace(/\b\w/g, (match) => match.toUpperCase());
}

export function WorkspaceSidebar({
  rows,
  gitBranches,
  activeSessionBranches,
  selectedBranch,
  onSelectBranch,
  onConfigureBranch,
  availableEditors,
  isDarkTheme,
  onOpenEditor,
  searchQuery,
  onSearchChange,
  loading,
  onRefresh,
  onHome,
  onCollapse,
  featureLabels,
}: WorkspaceSidebarProps) {
  const [localSearch, setLocalSearch] = useState(searchQuery);
  const [viewMode, setViewMode] = useState<SidebarView>('workspaces');
  const [showActiveSessionsOnly, setShowActiveSessionsOnly] = useState(false);
  const [workspaceTypeFilters, setWorkspaceTypeFilters] = useState<Set<WorkspaceTypeFilter>>(new Set());
  const [branchManagedFilters, setBranchManagedFilters] = useState<Set<BranchManagedFilter>>(new Set());
  const [includeArchived, setIncludeArchived] = useState(true);

  useEffect(() => {
    setLocalSearch(searchQuery);
  }, [searchQuery]);

  const searchTimer = useRef<number | undefined>(undefined);

  const handleSearchChange = useCallback(
    (value: string) => {
      setLocalSearch(value);
      window.clearTimeout(searchTimer.current);
      searchTimer.current = window.setTimeout(() => onSearchChange(value), 200);
    },
    [onSearchChange],
  );

  const toggleWorkspaceTypeFilter = useCallback((type: WorkspaceTypeFilter) => {
    setWorkspaceTypeFilters((current) => {
      const next = new Set(current);
      if (next.has(type)) {
        next.delete(type);
      } else {
        next.add(type);
      }
      return next;
    });
  }, []);

  const toggleBranchManagedFilter = useCallback((type: BranchManagedFilter) => {
    setBranchManagedFilters((current) => {
      const next = new Set(current);
      if (next.has(type)) {
        next.delete(type);
      } else {
        next.add(type);
      }
      return next;
    });
  }, []);

  const clearAllFilters = useCallback(() => {
    setWorkspaceTypeFilters(new Set());
    setBranchManagedFilters(new Set());
    setIncludeArchived(true);
    setShowActiveSessionsOnly(false);
  }, []);

  const activeSessionSet = useMemo(() => new Set(activeSessionBranches), [activeSessionBranches]);

  const workspaceByBranch = useMemo(
    () => new Map(rows.map((row) => [row.branch, row] as const)),
    [rows],
  );

  const workspaceTitleForRow = useCallback(
    (row: WorkspaceRow) => {
      const featureTitle = row.featureId ? featureLabels[row.featureId] : null;

      if (row.workspaceType === 'patch') {
        const title = featureTitle || 'Patch Workspace';
        return {
          title,
          subtitle: row.branch,
        };
      }

      const title = featureTitle || humanizeBranchToken(row.branch);
      return {
        title,
        subtitle: row.branch,
      };
    },
    [featureLabels],
  );

  const query = localSearch.trim().toLowerCase();

  const filteredWorkspaceRows = useMemo(() => {
    return rows.filter((row) => {
      if (!includeArchived && row.status === 'archived') {
        return false;
      }

      if (showActiveSessionsOnly && !activeSessionSet.has(row.branch)) {
        return false;
      }

      if (workspaceTypeFilters.size > 0 && !workspaceTypeFilters.has(row.workspaceType)) {
        return false;
      }

      if (!query) return true;

      const title = workspaceTitleForRow(row).title.toLowerCase();
      if (title.includes(query)) return true;
      if (row.branch.toLowerCase().includes(query)) return true;
      if (row.featureId?.toLowerCase().includes(query)) return true;
      return false;
    });
  }, [
    rows,
    includeArchived,
    showActiveSessionsOnly,
    activeSessionSet,
    workspaceTypeFilters,
    query,
    workspaceTitleForRow,
  ]);

  const filteredBranches = useMemo(() => {
    return gitBranches.filter((branch) => {
      const hasWorkspace = workspaceByBranch.has(branch.name);
      const managedType: BranchManagedFilter = hasWorkspace ? 'managed' : 'unmanaged';

      if (showActiveSessionsOnly && !activeSessionSet.has(branch.name)) {
        return false;
      }

      if (branchManagedFilters.size > 0 && !branchManagedFilters.has(managedType)) {
        return false;
      }

      if (!query) return true;

      if (branch.name.toLowerCase().includes(query)) return true;
      const managedTitle = hasWorkspace
        ? workspaceTitleForRow(workspaceByBranch.get(branch.name) as WorkspaceRow).title.toLowerCase()
        : '';
      return managedTitle.includes(query);
    });
  }, [
    gitBranches,
    workspaceByBranch,
    showActiveSessionsOnly,
    activeSessionSet,
    branchManagedFilters,
    query,
    workspaceTitleForRow,
  ]);

  const stableOnSelectBranch = useCallback(onSelectBranch, [onSelectBranch]);
  const stableOnOpenEditor = useCallback(onOpenEditor, [onOpenEditor]);

  return (
    <div className="flex h-full flex-col overflow-hidden border-r border-sidebar-border bg-sidebar">
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

        <div className="flex min-w-0 flex-1 items-center gap-2 px-2">
          <div className="size-2 rounded-full bg-primary" />
          <h2 className="truncate text-[11px] font-bold uppercase tracking-widest text-foreground">
            Workspaces
          </h2>
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
            <TooltipContent side="bottom">Collapse Sidebar (Ctrl/Cmd+B)</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon-xs"
                className="size-8 text-muted-foreground hover:text-foreground"
                onClick={onRefresh}
              >
                <RefreshCw className={loading ? 'size-3 animate-spin' : 'size-3'} />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">Refresh Workspaces</TooltipContent>
          </Tooltip>
        </div>
      </div>

      <div className="mt-2 flex h-12 shrink-0 items-center border-y border-border px-4 transition-all focus-within:bg-muted/10">
        <div className="relative w-full overflow-hidden">
          <Search className="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground/40" />
          <Input
            value={localSearch}
            onChange={(event) => handleSearchChange(event.target.value)}
            placeholder={viewMode === 'workspaces' ? 'Filter workspaces...' : 'Filter branches...'}
            className="h-8 border-none bg-transparent pl-8 text-[11px] font-bold text-foreground placeholder:font-medium placeholder:text-muted-foreground/40 focus-visible:ring-0"
          />
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-2 border-b border-border/60 px-3 py-2">
        <Popover>
          <Tooltip>
            <TooltipTrigger asChild>
              <PopoverTrigger>
                <Button size="icon-sm" variant="outline" className="size-8">
                  <LayoutPanelTop className="size-3.5" />
                </Button>
              </PopoverTrigger>
            </TooltipTrigger>
            <TooltipContent side="bottom">Switch View</TooltipContent>
          </Tooltip>
          <PopoverContent className="w-44 p-2" align="start" sideOffset={8}>
            <div className="space-y-1">
              <Button
                size="xs"
                variant={viewMode === 'workspaces' ? 'secondary' : 'ghost'}
                className="h-8 w-full justify-start"
                onClick={() => setViewMode('workspaces')}
              >
                Workspace View
              </Button>
              <Button
                size="xs"
                variant={viewMode === 'branches' ? 'secondary' : 'ghost'}
                className="h-8 w-full justify-start"
                onClick={() => setViewMode('branches')}
              >
                Branch View
              </Button>
            </div>
          </PopoverContent>
        </Popover>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              size="icon-sm"
              variant={showActiveSessionsOnly ? 'secondary' : 'outline'}
              className="size-8"
              onClick={() => setShowActiveSessionsOnly((current) => !current)}
            >
              <Activity className="size-3.5" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="bottom">Toggle active sessions only</TooltipContent>
        </Tooltip>

        <Popover>
          <Tooltip>
            <TooltipTrigger asChild>
              <PopoverTrigger>
                <Button size="icon-sm" variant="outline" className="size-8">
                  <Filter className="size-3.5" />
                </Button>
              </PopoverTrigger>
            </TooltipTrigger>
            <TooltipContent side="bottom">More Filters</TooltipContent>
          </Tooltip>
          <PopoverContent className="w-[280px] p-3" align="start" sideOffset={8}>
            <div className="space-y-3">
              <div className="space-y-2">
                <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
                  Workspace Type
                </p>
                {(['feature', 'patch', 'service'] as WorkspaceTypeFilter[]).map((type) => (
                  <label key={type} className="flex cursor-pointer items-center gap-2 text-xs">
                    <Checkbox
                      checked={workspaceTypeFilters.has(type)}
                      onCheckedChange={() => toggleWorkspaceTypeFilter(type)}
                    />
                    {type}
                  </label>
                ))}
                <label className="flex cursor-pointer items-center justify-between text-xs">
                  <span>Include archived</span>
                  <Switch checked={includeArchived} onCheckedChange={setIncludeArchived} />
                </label>
              </div>

              <div className="space-y-2 border-t pt-3">
                <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
                  Branch Scope
                </p>
                {(['managed', 'unmanaged'] as BranchManagedFilter[]).map((type) => (
                  <label key={type} className="flex cursor-pointer items-center gap-2 text-xs">
                    <Checkbox
                      checked={branchManagedFilters.has(type)}
                      onCheckedChange={() => toggleBranchManagedFilter(type)}
                    />
                    {type}
                  </label>
                ))}
              </div>

              <Button size="xs" variant="ghost" className="h-7 px-2" onClick={clearAllFilters}>
                Clear all
              </Button>
            </div>
          </PopoverContent>
        </Popover>
      </div>

      <div className="m-2 flex min-h-0 flex-1 flex-col rounded-lg border border-border/70 bg-card/60 shadow-sm">
        <div className="custom-scrollbar min-h-0 flex-1 overflow-y-auto p-2.5">
          {viewMode === 'workspaces' ? (
            filteredWorkspaceRows.length === 0 ? (
              <div className="rounded-lg border border-dashed border-border/60 px-3 py-6 text-center text-[11px] text-muted-foreground">
                No workspaces match this filter.
              </div>
            ) : (
              <div className="space-y-2">
                {filteredWorkspaceRows.map((row) => {
                  const labels = workspaceTitleForRow(row);
                  return (
                    <WorkspaceRowItem
                      key={row.id}
                      row={row}
                      title={labels.title}
                      subtitle={labels.subtitle}
                      selected={row.branch === selectedBranch}
                      hasActiveSession={activeSessionSet.has(row.branch)}
                      availableEditors={availableEditors}
                      isDarkTheme={isDarkTheme}
                      onSelectBranch={stableOnSelectBranch}
                      onOpenEditor={stableOnOpenEditor}
                    />
                  );
                })}
              </div>
            )
          ) : filteredBranches.length === 0 ? (
            <div className="rounded-lg border border-dashed border-border/60 px-3 py-6 text-center text-[11px] text-muted-foreground">
              No branches match this filter.
            </div>
          ) : (
            <div className="space-y-2">
              {filteredBranches.map((branchInfo) => {
                const managedRow = workspaceByBranch.get(branchInfo.name);
                const hasWorkspace = Boolean(managedRow);
                const hasActiveSession = activeSessionSet.has(branchInfo.name);
                const managedTitle = managedRow
                  ? workspaceTitleForRow(managedRow).title
                  : humanizeBranchToken(branchInfo.name);

                return (
                  <div
                    key={branchInfo.name}
                    role="button"
                    tabIndex={0}
                    className={cn(
                      'rounded-xl border px-3 py-2.5 transition-all',
                      branchInfo.name === selectedBranch
                        ? 'border-primary/50 bg-primary/10'
                        : 'border-border/40 bg-card/40 hover:border-border/60 hover:bg-muted/40',
                    )}
                    onClick={() => onSelectBranch(branchInfo.name)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter' || event.key === ' ') {
                        event.preventDefault();
                        onSelectBranch(branchInfo.name);
                      }
                    }}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="min-w-0">
                        <p className="text-sm font-semibold leading-tight text-foreground break-words">{managedTitle}</p>
                        {hasWorkspace ? (
                          <p className="mt-0.5 truncate font-mono text-[10px] text-muted-foreground">
                            {branchInfo.name}
                          </p>
                        ) : null}
                      </div>

                      <div className="flex shrink-0 items-center gap-1">
                        {branchInfo.current && (
                          <Badge variant="default" className="h-4.5 px-1.5 text-[9px] uppercase">
                            current
                          </Badge>
                        )}
                      </div>
                    </div>

                    <div className="mt-2 flex items-center justify-between gap-2">
                      <div className="flex items-center gap-2 text-[10px] font-semibold">
                        <span className="text-blue-700 dark:text-blue-300">files {branchInfo.touched_files}</span>
                        <span className="text-emerald-700 dark:text-emerald-300">+{branchInfo.insertions}</span>
                        <span className="text-red-700 dark:text-red-300">-{branchInfo.deletions}</span>
                        <span className="text-muted-foreground">↑{branchInfo.ahead} ↓{branchInfo.behind}</span>
                      </div>

                      <div className="flex items-center gap-1">
                        <Badge variant={hasWorkspace ? 'secondary' : 'outline'} className="h-4.5 px-1.5 text-[9px] uppercase">
                          {hasWorkspace ? 'managed' : 'unmanaged'}
                        </Badge>
                        {hasActiveSession && (
                          <Badge className="h-4.5 bg-emerald-500/15 px-1.5 text-[9px] uppercase text-emerald-600 dark:text-emerald-300">
                            session
                          </Badge>
                        )}
                      </div>
                    </div>

                    <div className="mt-2 flex justify-end">
                      <Button
                        size="xs"
                        variant={hasWorkspace ? 'outline' : 'default'}
                        className="h-7 gap-1.5"
                        onClick={(event) => {
                          event.stopPropagation();
                          if (hasWorkspace) {
                            onSelectBranch(branchInfo.name);
                          } else {
                            onConfigureBranch(branchInfo.name);
                          }
                        }}
                      >
                        {hasWorkspace ? <GitBranch className="size-3" /> : <GitBranchPlus className="size-3" />}
                        {hasWorkspace ? 'Open Detail' : 'Configure Workspace'}
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
