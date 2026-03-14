import { type ReactNode } from 'react';
import {
  ArrowLeft,
  FileCog,
  PanelLeftClose,
  PanelLeftOpen,
  ScrollText,
  Sun,
  Moon,
} from 'lucide-react';
import { Button } from '@ship/primitives';
import { Separator } from '@ship/primitives';
import { cn } from '@/lib/utils';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@ship/primitives';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';
import {
  AppRoutePath,
  ACTIVITY_ROUTE as ACTIVITY_PATH,
  SETTINGS_ROUTE as SETTINGS_PATH,
} from '@/lib/constants/routes';
import { NavItem, NavSection } from '@/lib/types/navigation';
import ProjectSwitcherMenuContent from '@/components/app/ProjectSwitcherMenuContent';

interface SidebarProps {
  collapsed: boolean;
  onToggleCollapse: () => void;
  activePath: AppRoutePath;
  onNavigate: (path: AppRoutePath) => void;
  activeProject?: { name: string; path: string } | null;
  recentProjects?: { name: string; path: string }[];
  onOpenProject: () => void;
  onNewProject: () => void;
  onSelectProject: (project: { name: string; path: string }) => void;
  onOpenGlobalNotes: () => void;
  sections: NavSection[];
  agentControl?: ReactNode;
  theme?: 'light' | 'dark';
  onThemeChange?: (theme: 'light' | 'dark') => void;
  contextualContent?: ReactNode;
  onBackToGlobal?: () => void;
}

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
  onOpenGlobalNotes,
  agentControl,
  sections,
  theme,
  onThemeChange,
  contextualContent,
  onBackToGlobal,
}: SidebarProps) {
  const hasSingleSection = sections.length <= 1;

  const avatarLabel = initialsFromProjectName(activeProject?.name ?? 'Ship');

  const renderNavButton = (item: NavItem, isCompact = false) => {
    const Icon = item.icon;
    const active = activePath === item.path;
    const secondary = item.priority === 'secondary';
    return (
      <Tooltip key={item.id} delayDuration={300}>
        <TooltipTrigger asChild>
          <Button
            variant='ghost'
            size={isCompact ? 'icon-sm' : 'sm'}
            className={cn(
              'relative w-full rounded-lg transition-all duration-200 group',
              !isCompact && 'justify-start gap-2.5 px-3',
              active
                ? isCompact
                  ? 'text-sidebar-primary hover:bg-sidebar-accent/50'
                  : 'bg-sidebar-primary/10 text-sidebar-primary font-bold border border-sidebar-primary/25 shadow-sm hover:bg-sidebar-primary/20'
                : secondary
                  ? 'text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent/50'
                  : 'text-sidebar-foreground/80 hover:text-sidebar-foreground hover:bg-sidebar-accent/50 hover:scale-[1.02] active:scale-[0.98]'
            )}
            onClick={() => onNavigate(item.path as AppRoutePath)}
            aria-label={item.label}
          >
            <Icon
              className={cn(
                'size-4 shrink-0 transition-all duration-200',
                active
                  ? 'text-sidebar-primary scale-110'
                  : secondary
                    ? 'text-sidebar-foreground/30'
                    : 'text-sidebar-foreground/50 group-hover:text-sidebar-primary/70 group-hover:scale-110'
              )}
            />
            {!isCompact && (
              <div className="flex flex-1 items-center justify-between min-w-0">
                <span className="text-[13px] font-medium tracking-tight truncate">{item.label}</span>
                {/*
                  SELECTION INDICATOR:
                  The primary color dot indicates the currently active route.
                */}
                {active && (
                  <div className="size-1.5 rounded-full bg-sidebar-primary shadow-[0_0_8px_currentColor]" />
                )}
              </div>
            )}

          </Button>
        </TooltipTrigger>
        <TooltipContent side="right" className="font-bold text-[10px] uppercase tracking-widest">
          {item.label}
        </TooltipContent>
      </Tooltip>
    );
  };

  return (
    <aside className={cn('sidebar flex h-full min-h-0 flex-col gap-3 p-3 bg-sidebar transition-colors duration-300', collapsed && 'items-center px-2')}>
      <header
        className={cn(
          'flex w-full items-center gap-2 px-2 py-1',
          collapsed && 'flex-col gap-3 pb-3'
        )}
      >
        <DropdownMenu>
          <DropdownMenuTrigger
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
            {/*
              NOTIFICATION DOT PATTERN:
              The orange dot (bg-emerald-500 here, though feedback mentioned orange in Overview context)
              is used to indicate active status or notifications.
            */}
            <div className="absolute -bottom-1 -right-1 size-3.5 rounded-full border-2 border-sidebar bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />

          </DropdownMenuTrigger>
          <DropdownMenuContent
            align={collapsed ? 'start' : 'start'}
            side={collapsed ? 'right' : 'bottom'}
            sideOffset={12}
            className="w-72 p-1.5 shadow-2xl border-sidebar-border bg-popover/95 backdrop-blur-xl animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10"
          >
            <ProjectSwitcherMenuContent
              activeProject={activeProject ?? null}
              projects={recentProjects ?? []}
              onSelectProject={onSelectProject}
              onOpenProject={onOpenProject}
              onNewProject={onNewProject}
              onOpenGlobalNotes={onOpenGlobalNotes}
            />
            {onThemeChange && (
              <>
                <DropdownMenuSeparator className="mx-1 my-1 opacity-50" />
                <DropdownMenuGroup className="p-1">
                  <DropdownMenuLabel className="flex items-center gap-2 px-2 pb-1.5 opacity-50 uppercase text-[9px] tracking-[0.2em] font-black">
                    Appearance
                  </DropdownMenuLabel>
                  <div className="grid grid-cols-2 gap-1 p-0.5">
                    <Button
                      variant={theme === 'light' ? 'secondary' : 'ghost'}
                      size="xs"
                      className={cn(
                        "h-8 gap-2 px-2 justify-start font-medium",
                        theme === 'light' && "bg-sidebar-primary/10 text-sidebar-primary border-sidebar-primary/20"
                      )}
                      onClick={() => onThemeChange('light')}
                    >
                      <Sun className="size-3.5" />
                      <span className="text-[11px]">Light</span>
                    </Button>
                    <Button
                      variant={theme === 'dark' ? 'secondary' : 'ghost'}
                      size="xs"
                      className={cn(
                        "h-8 gap-2 px-2 justify-start font-medium",
                        theme === 'dark' && "bg-sidebar-primary/10 text-sidebar-primary border-sidebar-primary/20"
                      )}
                      onClick={() => onThemeChange('dark')}
                    >
                      <Moon className="size-3.5" />
                      <span className="text-[11px]">Dark</span>
                    </Button>
                  </div>
                </DropdownMenuGroup>
              </>
            )}
          </DropdownMenuContent>
        </DropdownMenu>

        {!collapsed && (
          <div className="min-w-0 flex-1">
            <p className="truncate text-[13px] font-bold tracking-tight text-sidebar-foreground">
              {activeProject?.name?.trim() || 'Ship'}
            </p>
          </div>
        )}

        <div className={cn('ml-auto flex items-center', collapsed && 'ml-0')}>
          <Button
            variant="ghost"
            size="icon-xs"
            className="size-7 hover:bg-sidebar-accent/80 text-sidebar-foreground/60 hover:text-sidebar-foreground"
            onClick={onToggleCollapse}
            title={collapsed ? 'Expand bar' : 'Collapse bar'}
          >
            {collapsed ? <PanelLeftOpen className="size-4" /> : <PanelLeftClose className="size-4" />}
          </Button>
        </div>
      </header>

      {
        !collapsed && agentControl && (
          <div className="w-full">
            {agentControl}
          </div>
        )
      }

      <Separator className="w-full opacity-20" />

      <nav
        className={cn(
          'flex w-full flex-1 flex-col gap-1 overflow-y-auto no-scrollbar',
          collapsed && 'items-center p-1.5'
        )}
      >
        {contextualContent ? (
          <div className="flex h-full flex-col gap-3">
            {!collapsed && onBackToGlobal && (
              <Button
                variant="ghost"
                size="sm"
                className="justify-start gap-2 h-8 text-xs font-semibold hover:bg-sidebar-accent"
                onClick={onBackToGlobal}
              >
                <ArrowLeft className="size-4" />
                Back to Navigation
              </Button>
            )}
            <div className="flex-1 min-h-0">
              {contextualContent}
            </div>
          </div>
        ) : !collapsed ? (
          sections.map((section, idx) => (
            <div key={section.id} className="w-full">
              <div className="w-full space-y-0.5">
                {!hasSingleSection && section.label.trim().length > 0 && (
                  <p className="px-2 pt-0.5 text-[9px] font-black uppercase tracking-[0.2em] text-sidebar-foreground/50">
                    {section.label}
                  </p>
                )}
                <div className={cn("space-y-0.5", !hasSingleSection && "pb-1")}>
                  {section.items.map((item) => renderNavButton(item))}
                </div>
              </div>
              {!hasSingleSection && idx < sections.length - 1 && <Separator className="my-2 opacity-10 bg-sidebar-border" />}
            </div>
          ))
        ) : (
          <div className="flex flex-col items-center gap-6 py-4">
            {sections.map((section, idx) => (
              <div key={section.id} className="group flex flex-col items-center gap-1.5">
                {section.label.trim().length > 0 && (
                  <span className="text-[7px] font-black text-sidebar-foreground/50 uppercase tracking-[0.2em] transition-colors group-hover:text-sidebar-primary/70">
                    {section.label.slice(0, 3).toUpperCase()}
                  </span>
                )}
                <div className="flex flex-col gap-1 w-full">
                  {section.items.map(item => renderNavButton(item, true))}
                </div>
                {idx < sections.length - 1 && <Separator className="w-8 opacity-10 bg-sidebar-border" />}
              </div>
            ))}
          </div>
        )}
      </nav>

      <div className="mt-auto flex flex-col gap-1.5 w-full">
        <Button
          variant={activePath === ACTIVITY_PATH ? 'secondary' : 'outline'}
          size={collapsed ? 'icon-sm' : 'sm'}
          className={cn(
            'w-full border-dashed bg-transparent hover:bg-sidebar-accent border-sidebar-border text-sidebar-foreground/80',
            !collapsed && 'justify-start px-3 h-8 text-xs font-semibold'
          )}
          onClick={() => onNavigate(ACTIVITY_PATH)}
          title="Activity"
          aria-label="Activity"
        >
          <ScrollText className="size-4" />
          {!collapsed && 'Activity'}
        </Button>

        {!collapsed && (
          <p className="text-sidebar-foreground/50 w-full px-2 mt-2 text-[9px] font-black uppercase tracking-[0.2em]">
            System
          </p>
        )}
        <Button
          variant={activePath === SETTINGS_PATH ? 'secondary' : 'ghost'}
          size={collapsed ? 'icon-sm' : 'sm'}
          className={cn(
            'w-full text-sidebar-foreground/80 hover:text-sidebar-foreground hover:bg-sidebar-accent',
            !collapsed && 'justify-start px-3 h-8 text-xs font-semibold',
            activePath === SETTINGS_PATH && 'bg-sidebar-primary/10 text-sidebar-primary font-bold border border-sidebar-primary/25'
          )}
          onClick={() => onNavigate(SETTINGS_PATH)}
          title="Settings"
          aria-label="Settings"
        >
          <FileCog className="size-4" />
          {!collapsed && 'Settings'}
        </Button>
      </div>
    </aside>
  );
}
