import { useState, type ComponentType } from 'react';
import {
  Bot,
  ChevronDown,
  ChevronRight,
  FileCode2,
  FileCog,
  FileStack,
  Flag,
  FolderSearch,
  FolderOpen,
  FolderPlus,
  LayoutDashboard,
  NotebookPen,
  Package,
  PanelLeftClose,
  PanelLeftOpen,
  ScrollText,
  History,
  Target,
  Workflow,
} from 'lucide-react';
import { ProjectDiscovery as Project } from '@/bindings';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Separator } from '@/components/ui/separator';
import { cn } from '@/lib/utils';
import {
  AppRoutePath,
  ACTIVITY_ROUTE as ACTIVITY_PATH,
  ADRS_ROUTE as ADRS_PATH,
  AGENTS_MCP_ROUTE as AGENTS_MCP_PATH,
  AGENTS_PERMISSIONS_ROUTE as AGENTS_PERMISSIONS_PATH,
  AGENTS_PROVIDERS_ROUTE as AGENTS_PROVIDERS_PATH,
  AGENTS_RULES_ROUTE as AGENTS_RULES_PATH,
  AGENTS_SKILLS_ROUTE as AGENTS_SKILLS_PATH,
  FEATURES_ROUTE as FEATURES_PATH,
  ISSUES_ROUTE as ISSUES_PATH,
  NOTES_ROUTE as NOTES_PATH,
  OVERVIEW_ROUTE as OVERVIEW_PATH,
  RELEASES_ROUTE as RELEASES_PATH,
  SETTINGS_ROUTE as SETTINGS_PATH,
  SPECS_ROUTE as SPECS_PATH,
  WORKFLOW_WORKSPACE_ROUTE as WORKFLOW_WORKSPACE_PATH,
} from '@/lib/constants/routes';

interface SidebarProps {
  collapsed: boolean;
  onToggleCollapse: () => void;
  activePath: AppRoutePath;
  onNavigate: (path: AppRoutePath) => void;
  activeProject: Project | null;
  recentProjects: Project[];
  onOpenProject: () => void;
  onNewProject: () => void;
  onSelectProject: (project: Project) => void;
}

type NavItem = {
  path: AppRoutePath;
  label: string;
  icon: ComponentType<{ className?: string }>;
};

const PROJECT_ITEMS: NavItem[] = [
  { path: OVERVIEW_PATH, label: 'Overview', icon: LayoutDashboard },
  { path: NOTES_PATH, label: 'Notes', icon: NotebookPen },
  { path: ADRS_PATH, label: 'Decisions', icon: FileStack },
  { path: RELEASES_PATH, label: 'Releases', icon: Package },
  { path: FEATURES_PATH, label: 'Features', icon: Flag },
];

const WORKFLOW_ITEMS: NavItem[] = [
  { path: WORKFLOW_WORKSPACE_PATH, label: 'Workspace', icon: Workflow },
  { path: ISSUES_PATH, label: 'Issues', icon: FolderSearch },
  { path: SPECS_PATH, label: 'Specs', icon: FileCode2 },
];

const AGENT_ITEMS: NavItem[] = [
  { path: AGENTS_PROVIDERS_PATH, label: 'Providers', icon: Bot },
  { path: AGENTS_MCP_PATH, label: 'MCP Servers', icon: Package },
  { path: AGENTS_SKILLS_PATH, label: 'Skills', icon: FileStack },
  { path: AGENTS_RULES_PATH, label: 'Rules', icon: FileCode2 },
  { path: AGENTS_PERMISSIONS_PATH, label: 'Permissions', icon: FileCog },
];

function initialsFromProjectName(projectName: string | null | undefined): string {
  const cleaned = (projectName ?? '').trim();
  if (!cleaned) return 'SW';
  const parts = cleaned.split(/\s+/).filter(Boolean);
  if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase();
  }
  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? '')
    .join('');
}

export default function Sidebar({
  collapsed,
  onToggleCollapse,
  activePath,
  onNavigate,
  activeProject,
  recentProjects,
  onOpenProject,
  onNewProject,
  onSelectProject,
}: SidebarProps) {
  const [projectOpen, setProjectOpen] = useState(true);
  const [workflowOpen, setWorkflowOpen] = useState(true);
  const [agentsOpen, setAgentsOpen] = useState(true);
  const otherProjects = recentProjects
    .filter((project) => project.path !== activeProject?.path)
    .slice(0, 3);
  const avatarLabel = initialsFromProjectName(activeProject?.name ?? 'Shipwright');

  const renderNavButton = (item: NavItem, isCompact = false) => {
    const Icon = item.icon;
    const active = activePath === item.path;
    return (
      <Button
        key={item.path}
        variant={active ? 'secondary' : 'ghost'}
        size={isCompact ? 'icon-sm' : 'default'}
        className={cn(
          'relative w-full rounded-md transition-all duration-200',
          !isCompact && 'justify-start hover:pl-3',
          active ? 'font-medium shadow-sm ring-1 ring-primary/20 bg-primary/10' : 'hover:bg-muted/50'
        )}
        onClick={() => onNavigate(item.path)}
        title={item.label}
        aria-label={item.label}
      >
        <Icon className={cn("size-4", active ? "text-primary" : "text-muted-foreground")} />
        {!isCompact && <span className="ml-2 text-sm">{item.label}</span>}
        {!isCompact && active && (
          <div className="ml-auto flex items-center gap-1.5">
            <div className="size-1 rounded-full bg-primary" />
            <ChevronRight className="size-3 opacity-40" />
          </div>
        )}
        {isCompact && active && (
          <div className="absolute right-1 top-1/2 -translate-y-1/2 size-1 rounded-full bg-primary shadow-[0_0_8px_rgba(var(--primary),0.6)]" />
        )}
      </Button>
    );
  };

  return (
    <aside className={cn('sidebar flex h-full min-h-0 flex-col gap-4 p-3', collapsed && 'items-center px-2')}>
      <header
        className={cn(
          'flex w-full items-center gap-2 rounded-xl border border-primary/10 bg-gradient-to-br from-card to-card/50 p-2 shadow-sm',
          collapsed && 'flex-col gap-3 pb-3'
        )}
      >
        <DropdownMenu>
          <DropdownMenuTrigger>
            <button
              className={cn(
                "group relative overflow-hidden flex size-10 items-center justify-center rounded-xl border transition-all duration-300",
                "bg-gradient-to-br from-amber-400 via-amber-500 to-amber-600 shadow-[0_2px_10px_rgba(245,158,11,0.3)]",
                "hover:shadow-[0_4px_20px_rgba(245,158,11,0.5)] hover:scale-105 active:scale-95",
                "border-amber-400/50 dark:border-amber-400/20",
                collapsed && "size-9 rounded-lg"
              )}
              title="Project Switcher"
            >
              <div className="absolute inset-0 bg-white/10 opacity-0 group-hover:opacity-100 transition-opacity" />
              <span className="relative z-10 text-xs font-black tracking-tighter text-white drop-shadow-sm font-mono">
                {avatarLabel}
              </span>
              <div className="absolute -bottom-1 -right-1 size-3.5 rounded-full border-2 border-background bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            align={collapsed ? 'start' : 'start'}
            side={collapsed ? 'right' : 'bottom'}
            sideOffset={12}
            className="w-72 p-1.5 shadow-2xl border-border/50 bg-popover/95 backdrop-blur-md animate-in fade-in zoom-in-95 duration-200"
          >
            <DropdownMenuGroup className="p-1">
              <DropdownMenuLabel className="flex items-center gap-2 px-2 pb-2 opacity-50 uppercase text-[9px] tracking-[0.2em] font-black">
                <Target className="size-3" />
                Current Project
              </DropdownMenuLabel>
              {activeProject ? (
                <div className="bg-gradient-to-br from-amber-500/15 to-amber-600/5 mb-1.5 rounded-lg border border-amber-500/30 px-3.5 py-3 shadow-inner">
                  <p className="truncate text-sm font-bold text-foreground leading-tight">{activeProject.name}</p>
                  <p className="text-muted-foreground truncate text-[10px] opacity-60 font-mono mt-1 flex items-center gap-1">
                    <span className="opacity-40">path:</span> {activeProject.path}
                  </p>
                </div>
              ) : (
                <div className="text-muted-foreground mb-1.5 rounded-lg border border-dashed border-border/60 bg-muted/20 px-3.5 py-3 text-xs italic">
                  No active project selected.
                </div>
              )}
            </DropdownMenuGroup>
            <DropdownMenuSeparator className="mx-1 my-1 opacity-50" />
            <DropdownMenuGroup className="p-1">
              <DropdownMenuLabel className="flex items-center gap-2 px-2 pb-2 opacity-50 uppercase text-[9px] tracking-[0.2em] font-black">
                <History className="size-3" />
                Recent Projects
              </DropdownMenuLabel>
              {otherProjects.length === 0 ? (
                <div className="text-muted-foreground rounded-lg px-2.5 py-3 text-xs italic opacity-60">No recent projects.</div>
              ) : (
                <div className="space-y-1">
                  {otherProjects.map((project) => (
                    <DropdownMenuItem
                      key={project.path}
                      className="cursor-pointer rounded-md px-3 py-2.5 transition-all active:scale-[0.98]"
                      onClick={() => onSelectProject(project)}
                    >
                      <div className="min-w-0">
                        <p className="truncate text-sm font-semibold leading-tight">{project.name}</p>
                        <p className="text-muted-foreground truncate text-[9px] opacity-50 font-mono mt-0.5">{project.path}</p>
                      </div>
                    </DropdownMenuItem>
                  ))}
                </div>
              )}
            </DropdownMenuGroup>
            <DropdownMenuSeparator className="mx-1 my-1 opacity-50" />
            <DropdownMenuGroup className="p-1 space-y-0.5">
              <DropdownMenuItem onClick={onOpenProject} className="cursor-pointer gap-2 py-2 rounded-md">
                <FolderOpen className="size-4 opacity-60" />
                <span className="text-sm font-medium">Open Folder...</span>
              </DropdownMenuItem>
              <DropdownMenuItem onClick={onNewProject} className="cursor-pointer gap-2 py-2 rounded-md">
                <FolderPlus className="size-4 opacity-60" />
                <span className="text-sm font-medium">New Project...</span>
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>

        {!collapsed && (
          <div className="min-w-0 flex-1 ml-1">
            <p className="truncate text-[13px] font-bold tracking-tight text-foreground/90">
              {activeProject?.name?.trim() || 'Shipwright'}
            </p>
          </div>
        )}

        <div className={cn('ml-auto flex items-center', collapsed && 'ml-0')}>
          <Button
            variant="ghost"
            size="icon-xs"
            className="size-7 hover:bg-muted/80"
            onClick={onToggleCollapse}
            title={collapsed ? 'Expand bar' : 'Collapse bar'}
          >
            {collapsed ? <PanelLeftOpen className="size-4 opacity-60" /> : <PanelLeftClose className="size-4 opacity-60" />}
          </Button>
        </div>
      </header>

      <Separator className="w-full" />

      <nav
        className={cn(
          'flex w-full flex-1 flex-col gap-2 rounded-lg border bg-card/30 p-2',
          collapsed && 'items-center p-1.5'
        )}
      >
        {!collapsed ? (
          <>
            <div className="w-full space-y-1">
              <Button
                variant="ghost"
                size="sm"
                className="w-full justify-start px-2"
                onClick={() => setProjectOpen((current) => !current)}
              >
                <span className="text-muted-foreground text-[10px] font-medium uppercase tracking-wider">
                  Project
                </span>
                <ChevronDown className={cn('ml-auto size-3.5 transition-transform', projectOpen && 'rotate-180')} />
              </Button>
              {projectOpen && <div className="space-y-1">{PROJECT_ITEMS.map((item) => renderNavButton(item))}</div>}
            </div>

            <Separator className="my-1" />

            <div className="w-full space-y-1">
              <Button
                variant="ghost"
                size="sm"
                className="w-full justify-start px-2"
                onClick={() => setWorkflowOpen((current) => !current)}
              >
                <span className="text-muted-foreground text-[10px] font-medium uppercase tracking-wider">
                  Workflow
                </span>
                <ChevronDown className={cn('ml-auto size-3.5 transition-transform', workflowOpen && 'rotate-180')} />
              </Button>
              {workflowOpen && <div className="space-y-1">{WORKFLOW_ITEMS.map((item) => renderNavButton(item))}</div>}
            </div>

            <Separator className="my-1" />

            <div className="w-full space-y-1">
              <Button
                variant="ghost"
                size="sm"
                className="w-full justify-start px-2"
                onClick={() => setAgentsOpen((current) => !current)}
              >
                <span className="text-muted-foreground text-[10px] font-medium uppercase tracking-wider">
                  Agents
                </span>
                <ChevronDown className={cn('ml-auto size-3.5 transition-transform', agentsOpen && 'rotate-180')} />
              </Button>
              {agentsOpen && <div className="space-y-1">{AGENT_ITEMS.map((item) => renderNavButton(item))}</div>}
            </div>
          </>
        ) : (
          <div className="flex flex-col items-center gap-6 py-4">
            <div className="group flex flex-col items-center gap-1.5">
              <span className="text-[7px] font-black text-muted-foreground/40 uppercase tracking-[0.2em] transition-colors group-hover:text-primary/70">PRJ</span>
              <div className="flex flex-col gap-1 w-full">
                {PROJECT_ITEMS.map(item => renderNavButton(item, true))}
              </div>
            </div>

            <Separator className="w-8 opacity-20" />

            <div className="group flex flex-col items-center gap-1.5">
              <span className="text-[7px] font-black text-muted-foreground/40 uppercase tracking-[0.2em] transition-colors group-hover:text-primary/70">WKF</span>
              <div className="flex flex-col gap-1 w-full">
                {WORKFLOW_ITEMS.map(item => renderNavButton(item, true))}
              </div>
            </div>

            <Separator className="w-8 opacity-20" />

            <div className="group flex flex-col items-center gap-1.5">
              <span className="text-[7px] font-black text-muted-foreground/40 uppercase tracking-[0.2em] transition-colors group-hover:text-primary/70">AGT</span>
              <div className="flex flex-col gap-1 w-full">
                {AGENT_ITEMS.map(item => renderNavButton(item, true))}
              </div>
            </div>
          </div>
        )}
      </nav>

      <Button
        variant={activePath === ACTIVITY_PATH ? 'secondary' : 'outline'}
        size={collapsed ? 'icon-sm' : 'xs'}
        className={cn('w-full border-dashed', !collapsed && 'justify-start')}
        onClick={() => onNavigate(ACTIVITY_PATH)}
        title="Activity"
        aria-label="Activity"
      >
        <ScrollText className="size-4" />
        {!collapsed && 'Activity'}
      </Button>

      {!collapsed && (
        <p className="text-muted-foreground w-full px-1 text-[10px] font-medium uppercase tracking-wider">
          System
        </p>
      )}
      <Button
        variant={activePath === SETTINGS_PATH ? 'secondary' : 'ghost'}
        size={collapsed ? 'icon-sm' : 'default'}
        className={cn('w-full', !collapsed && 'justify-start', activePath === SETTINGS_PATH && 'font-medium')}
        onClick={() => onNavigate(SETTINGS_PATH)}
        title="Settings"
        aria-label="Settings"
      >
        <FileCog className="size-4" />
        {!collapsed && 'Settings'}
      </Button>
    </aside>
  );
}
