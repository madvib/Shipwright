import { useState, type ComponentType } from 'react';
import {
  Bot,
  ChevronDown,
  ChevronRight,
  ChevronsUpDown,
  FileCode2,
  FileCog,
  FileStack,
  Flag,
  FolderSearch,
  FolderOpen,
  FolderPlus,
  LayoutDashboard,
  Menu,
  Package,
  PanelLeftClose,
  PanelLeftOpen,
  ScrollText,
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
  AGENTS_PROMPTS_ROUTE as AGENTS_PROMPTS_PATH,
  AGENTS_PROVIDERS_ROUTE as AGENTS_PROVIDERS_PATH,
  AGENTS_SKILLS_ROUTE as AGENTS_SKILLS_PATH,
  FEATURES_ROUTE as FEATURES_PATH,
  ISSUES_ROUTE as ISSUES_PATH,
  OVERVIEW_ROUTE as OVERVIEW_PATH,
  PROJECTS_ROUTE as PROJECTS_PATH,
  RELEASES_ROUTE as RELEASES_PATH,
  SETTINGS_ROUTE as SETTINGS_PATH,
  SPECS_ROUTE as SPECS_PATH,
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
  { path: ADRS_PATH, label: 'Decisions', icon: FileStack },
  { path: RELEASES_PATH, label: 'Releases', icon: Package },
  { path: FEATURES_PATH, label: 'Features', icon: Flag },
];

const WORKFLOW_ITEMS: NavItem[] = [
  { path: ISSUES_PATH, label: 'Issues', icon: FolderSearch },
  { path: SPECS_PATH, label: 'Specs', icon: FileCode2 },
];

const AGENT_ITEMS: NavItem[] = [
  { path: AGENTS_PROVIDERS_PATH, label: 'Providers', icon: Bot },
  { path: AGENTS_MCP_PATH, label: 'MCP', icon: Package },
  { path: AGENTS_SKILLS_PATH, label: 'Skills', icon: FileStack },
  { path: AGENTS_PROMPTS_PATH, label: 'Prompts', icon: FileCode2 },
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

  const renderNavButton = (item: NavItem) => {
    const Icon = item.icon;
    const active = activePath === item.path;
    return (
      <Button
        key={item.path}
        variant={active ? 'secondary' : 'ghost'}
        size={collapsed ? 'icon-sm' : 'default'}
        className={cn('w-full rounded-md', !collapsed && 'justify-start', active && 'font-medium')}
        onClick={() => onNavigate(item.path)}
        title={item.label}
        aria-label={item.label}
      >
        <Icon className="size-4" />
        {!collapsed && item.label}
        {!collapsed && active && <ChevronRight className="ml-auto size-3.5" />}
      </Button>
    );
  };

  return (
    <aside className={cn('sidebar flex h-full min-h-0 flex-col gap-4 p-3', collapsed && 'items-center px-2')}>
      <header
        className={cn(
          'flex w-full items-center gap-2 rounded-lg border bg-card/60 px-2 py-2',
          collapsed && 'w-auto flex-col gap-1 px-1.5 py-1.5'
        )}
      >
        <div className="border-amber-400/45 bg-amber-500/12 text-amber-800 dark:text-amber-200 flex size-10 items-center justify-center rounded-md border text-xs font-semibold">
          {avatarLabel}
        </div>
        {!collapsed && (
          <div className="min-w-0 flex-1">
            <p
              className="truncate text-sm font-semibold tracking-tight"
              title={activeProject ? activeProject.path : 'No active project path'}
            >
              {activeProject?.name?.trim() || 'No Project Selected'}
            </p>
          </div>
        )}
        <div className={cn('ml-auto flex items-center gap-1', collapsed && 'ml-0 flex-row')}>
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <Button variant="ghost" size="icon-sm" title="Project switcher" aria-label="Project switcher" />
              }
            >
              {collapsed ? <Menu className="size-4" /> : <ChevronsUpDown className="size-4" />}
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align={collapsed ? 'end' : 'start'}
              side="bottom"
              sideOffset={6}
              className="!w-72 p-2"
            >
              <DropdownMenuGroup>
                <DropdownMenuLabel className="px-1 pb-1">Project Switcher</DropdownMenuLabel>
                {activeProject ? (
                  <div className="border-amber-400/40 bg-amber-500/[0.06] mb-1 rounded-md border px-2.5 py-2">
                    <p className="truncate text-sm font-medium">{activeProject.name}</p>
                    <p className="text-muted-foreground truncate text-xs">{activeProject.path}</p>
                  </div>
                ) : (
                  <div className="text-muted-foreground mb-1 rounded-md border border-dashed px-2.5 py-2 text-xs">
                    No active project selected.
                  </div>
                )}
              </DropdownMenuGroup>
              <DropdownMenuSeparator />
              <DropdownMenuGroup>
                <DropdownMenuLabel className="px-1 pb-1">Recent Projects</DropdownMenuLabel>
                {otherProjects.length === 0 ? (
                  <div className="text-muted-foreground rounded-md px-2.5 py-1.5 text-xs">No recent projects yet.</div>
                ) : (
                  otherProjects.map((project) => (
                    <DropdownMenuItem
                      key={project.path}
                      className="border-amber-400/35 bg-amber-500/[0.05] mb-1 h-auto rounded-md border px-2.5 py-2"
                      title={project.path}
                      onClick={() => onSelectProject(project)}
                    >
                      <div className="min-w-0">
                        <p className="truncate text-sm font-medium">{project.name}</p>
                        <p className="text-muted-foreground truncate text-xs">{project.path}</p>
                      </div>
                    </DropdownMenuItem>
                  ))
                )}
              </DropdownMenuGroup>
              <DropdownMenuSeparator />
              <DropdownMenuGroup>
                <DropdownMenuItem onClick={onOpenProject}>
                  <FolderOpen className="size-4" />
                  Open
                </DropdownMenuItem>
                <DropdownMenuItem onClick={onNewProject}>
                  <FolderPlus className="size-4" />
                  New
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => onNavigate(PROJECTS_PATH)}>
                  <FolderSearch className="size-4" />
                  Projects
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>

          <Button
            variant="ghost"
            size="icon-sm"
            onClick={onToggleCollapse}
            title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          >
            {collapsed ? <PanelLeftOpen className="size-4" /> : <PanelLeftClose className="size-4" />}
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
              {projectOpen && <div className="space-y-1">{PROJECT_ITEMS.map(renderNavButton)}</div>}
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
              {workflowOpen && <div className="space-y-1">{WORKFLOW_ITEMS.map(renderNavButton)}</div>}
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
              {agentsOpen && <div className="space-y-1">{AGENT_ITEMS.map(renderNavButton)}</div>}
            </div>
          </>
        ) : (
          <div className="space-y-1">
            {[...PROJECT_ITEMS, ...WORKFLOW_ITEMS, ...AGENT_ITEMS].map(renderNavButton)}
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
