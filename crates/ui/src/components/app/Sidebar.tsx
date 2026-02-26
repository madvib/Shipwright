import type { ComponentType } from 'react';
import {
  Bot,
  ChevronRight,
  FileCode2,
  FileCog,
  FileStack,
  Flag,
  FolderSearch,
  FolderGit2,
  FolderOpen,
  FolderPlus,
  LayoutDashboard,
  Package,
  PanelLeftClose,
  PanelLeftOpen,
  ScrollText,
} from 'lucide-react';
import { ProjectDiscovery as Project } from '@/bindings';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { cn } from '@/lib/utils';
import {
  AppRoutePath,
  ACTIVITY_ROUTE as ACTIVITY_PATH,
  ADRS_ROUTE as ADRS_PATH,
  AGENTS_ROUTE as AGENTS_PATH,
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

const NAV_ITEMS: {
  path: AppRoutePath;
  label: string;
  icon: ComponentType<{ className?: string }>;
}[] = [
  { path: OVERVIEW_PATH, label: 'Overview', icon: LayoutDashboard },
  { path: AGENTS_PATH, label: 'Agents', icon: Bot },
  { path: ISSUES_PATH, label: 'Issues', icon: FolderGit2 },
  { path: RELEASES_PATH, label: 'Releases', icon: Package },
  { path: FEATURES_PATH, label: 'Features', icon: Flag },
  { path: SPECS_PATH, label: 'Specs', icon: FileCode2 },
  { path: ADRS_PATH, label: 'Decisions', icon: FileStack },
  { path: ACTIVITY_PATH, label: 'Activity', icon: ScrollText },
];

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
  const otherProjects = recentProjects
    .filter((project) => project.path !== activeProject?.path)
    .slice(0, 6);

  return (
    <aside className={cn('sidebar flex h-full min-h-0 flex-col gap-4 p-3', collapsed && 'items-center px-2')}>
      <header
        className={cn(
          'flex w-full items-center gap-2 rounded-lg border bg-card/60 px-2 py-2',
          collapsed && 'w-auto flex-col gap-1 px-1.5 py-1.5'
        )}
      >
        <div className="bg-primary/10 border-primary/30 flex size-10 items-center justify-center rounded-md border">
          <img src="/logo.svg" alt="Shipwright" className="size-8 object-contain" />
        </div>
        {!collapsed && (
          <div className="min-w-0">
            <p className="truncate text-sm font-semibold tracking-tight">Shipwright</p>
          </div>
        )}
        <Button
          variant="ghost"
          size="icon-sm"
          className={cn('ml-auto', collapsed && 'ml-0')}
          onClick={onToggleCollapse}
          title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {collapsed ? <PanelLeftOpen className="size-4" /> : <PanelLeftClose className="size-4" />}
        </Button>
      </header>

      <section className={cn('w-full space-y-3 rounded-lg border bg-card/40 p-2.5', collapsed && 'p-2')}>
        <div className="flex items-center justify-between">
          {!collapsed && (
            <p className="text-muted-foreground text-xs font-medium uppercase tracking-wide">Project</p>
          )}
          {activeProject && !collapsed && (
            <Badge variant="outline" className="text-[10px]">
              Active
            </Badge>
          )}
        </div>

        {!collapsed && (
          <>
            {activeProject ? (
              <div className="space-y-1 rounded-md border bg-background/60 px-3 py-2">
                <p className="truncate text-sm font-medium">{activeProject.name}</p>
                <p className="text-muted-foreground text-xs">
                  {typeof activeProject.issue_count === 'number'
                    ? `${activeProject.issue_count} issues`
                    : 'Issue count unavailable'}
                </p>
              </div>
            ) : (
              <div className="text-muted-foreground rounded-md border border-dashed px-3 py-2 text-xs">
                No project selected
              </div>
            )}

            {otherProjects.length > 0 && (
              <div className="space-y-1">
                <p className="text-muted-foreground px-1 text-[11px] font-medium uppercase tracking-wide">
                  Recent
                </p>
                {otherProjects.map((project) => (
                  <Button
                    key={project.path}
                    variant="ghost"
                    className="h-auto w-full justify-start px-2 py-1.5 text-left"
                    title={project.path}
                    onClick={() => onSelectProject(project)}
                  >
                    <span className="truncate text-sm">{project.name}</span>
                  </Button>
                ))}
              </div>
            )}
          </>
        )}

        <div className={cn('grid gap-2', collapsed ? 'grid-cols-1' : 'grid-cols-2')}>
          <Button
            variant="outline"
            size={collapsed ? 'icon-sm' : 'sm'}
            className={cn('w-full', !collapsed && 'justify-start')}
            onClick={onOpenProject}
            title="Open project"
            aria-label="Open project"
          >
            <FolderOpen className="size-4" />
            {!collapsed && 'Open'}
          </Button>
          <Button
            variant="secondary"
            size={collapsed ? 'icon-sm' : 'sm'}
            className={cn('w-full', !collapsed && 'justify-start')}
            onClick={onNewProject}
            title="Create project"
            aria-label="Create project"
          >
            <FolderPlus className="size-4" />
            {!collapsed && 'New'}
          </Button>
        </div>

        <Button
          variant={activePath === PROJECTS_PATH ? 'secondary' : 'ghost'}
          size={collapsed ? 'icon-sm' : 'sm'}
          className={cn('w-full', !collapsed && 'justify-start')}
          onClick={() => onNavigate(PROJECTS_PATH)}
          title="Projects"
          aria-label="Projects"
        >
          <FolderSearch className="size-4" />
          {!collapsed && 'Projects'}
        </Button>
      </section>

      <Separator className="w-full" />

      <nav className={cn('flex w-full flex-1 flex-col gap-1.5', collapsed && 'items-center')}>
        {NAV_ITEMS.map((item) => {
          const Icon = item.icon;
          const active = activePath === item.path;
          return (
            <Button
              key={item.path}
              variant={active ? 'secondary' : 'ghost'}
              size={collapsed ? 'icon-sm' : 'default'}
              className={cn('w-full', !collapsed && 'justify-start', active && 'font-medium')}
              onClick={() => onNavigate(item.path)}
              title={item.label}
              aria-label={item.label}
            >
              <Icon className="size-4" />
              {!collapsed && item.label}
              {!collapsed && active && <ChevronRight className="ml-auto size-3.5" />}
            </Button>
          );
        })}
      </nav>

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
