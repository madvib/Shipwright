import { useMemo, useState, useEffect, useCallback } from 'react';
import { Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import { useUpdateChecker } from '@/lib/hooks/useUpdateChecker';
import Sidebar from '@/components/app/Sidebar';
import { PageChromeProvider, PageChromeContextValue } from '@ship/ui';
import AgentModeControl from '@/features/agents/AgentModeControl.tsx';
import ProjectOnboarding from '@/features/planning/common/ProjectOnboarding';
import { SearchModal } from '@/components/app/SearchModal';
import { Button } from '@ship/ui';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import {
  AppRoutePath,
  AGENTS_PROVIDERS_ROUTE,
  NOTES_ROUTE,
  ROUTE_LABELS,
  SETTINGS_ROUTE,
  OVERVIEW_ROUTE,
  PROJECTS_ROUTE,
  WORKFLOW_WORKSPACE_ROUTE,
  normalizePath,
} from '@/lib/constants/routes';
import {
  MessageCircle,
  Search,
} from 'lucide-react';
import { SHIP_NAV_SECTIONS } from '@/lib/modules/ship';
import { cn } from '@/lib/utils';

const DEFAULT_SIDEBAR_WIDTH = 280;
const MIN_SIDEBAR_WIDTH = 220;
const MAX_SIDEBAR_WIDTH = 380;
const COLLAPSED_RAIL_WIDTH = '3.25rem';

export default function App() {
  useUpdateChecker();
  const location = useLocation();
  const navigate = useNavigate();
  const workspace = useWorkspace();
  const routePath = normalizePath(location.pathname) as AppRoutePath;

  const [pageChrome, setPageChrome] = useState<Partial<PageChromeContextValue>>({});
  const [sidebarWidth, setSidebarWidth] = useState(() => {
    const saved = localStorage.getItem('sidebar-width');
    if (!saved) return DEFAULT_SIDEBAR_WIDTH;
    const parsed = parseInt(saved, 10);
    if (Number.isNaN(parsed)) return DEFAULT_SIDEBAR_WIDTH;
    return Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, parsed));
  });
  const [isResizing, setIsResizing] = useState(false);

  const navigateTo = useCallback((path: AppRoutePath) => {
    if (path === NOTES_ROUTE) {
      workspace.setNotesScope('project');
    }
    if (normalizePath(location.pathname) !== path) {
      void navigate({ to: path });
    }
  }, [location.pathname, navigate, workspace]);

  const defaultChrome = useMemo((): Partial<PageChromeContextValue> =>
  (!workspace.noProject || routePath === PROJECTS_ROUTE
    ? {
      breadcrumb: (
        <nav className="text-muted-foreground flex items-center gap-1 text-xs">
          {routePath === PROJECTS_ROUTE ? (
            <span className="text-foreground">Projects</span>
          ) : (
            <>
              <Button
                variant="ghost"
                size="xs"
                className="h-6 px-1.5 text-xs"
                onClick={() => navigateTo(PROJECTS_ROUTE)}
              >
                Projects
              </Button>
              <span>/</span>
              <Button
                variant="ghost"
                size="xs"
                className="h-6 px-1.5 text-xs"
                onClick={() => navigateTo(WORKFLOW_WORKSPACE_ROUTE)}
              >
                {workspace.activeProject?.name ?? 'Project'}
              </Button>
              {routePath !== OVERVIEW_ROUTE && (
                <>
                  <span>/</span>
                  <span className="text-foreground">{ROUTE_LABELS[routePath]}</span>
                </>
              )}
            </>
          )}
        </nav>
      ),
    }
    : {}), [workspace.noProject, workspace.activeProject, routePath, navigateTo]);

  const activeChrome = useMemo(() => ({
    ...defaultChrome,
    ...pageChrome,
  }), [defaultChrome, pageChrome]);

  const handleUpdateChrome = useCallback((updates: Partial<PageChromeContextValue>) => {
    setPageChrome(prev => {
      const hasChange = Object.entries(updates).some(([key, val]) => prev[key as keyof PageChromeContextValue] !== val);
      if (!hasChange) return prev;
      return { ...prev, ...updates };
    });
  }, []);

  const openGlobalNotes = () => {
    workspace.setNotesScope('global');
    if (normalizePath(location.pathname) !== NOTES_ROUTE) {
      void navigate({ to: NOTES_ROUTE });
    }
  };

  const handleSelectProject = async (project: Parameters<typeof workspace.handleSelectProject>[0]) => {
    const selected = await workspace.handleSelectProject(project);
    if (selected) {
      navigateTo(WORKFLOW_WORKSPACE_ROUTE);
    }
  };

  const openCommandPalette = useCallback(() => {
    const event = new KeyboardEvent('keydown', {
      key: 'k',
      metaKey: true,
      bubbles: true,
    });
    document.dispatchEvent(event);
  }, []);

  const [chatOpen, setChatOpen] = useState(false);

  const agentControl = useMemo(() => {
    if (workspace.noProject) return null;

    return (
      <AgentModeControl
        modes={workspace.modes}
        activeModeId={workspace.activeModeId}
        aiProvider={workspace.aiProvider}
        aiModel={workspace.aiModel}
        switchingMode={workspace.switchingMode}
        onSetMode={(modeId: string | null) => {
          void workspace.handleSetActiveMode(modeId);
        }}
        onOpenAgents={() => {
          void navigate({ to: AGENTS_PROVIDERS_ROUTE });
        }}
      />
    );
  }, [workspace, navigate]);

  // Keyboard Shortcuts
  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if ((e.key === 'b' || e.key === 'B') && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        workspace.setSidebarCollapsed((prev) => !prev);
      }
    };

    document.addEventListener('keydown', down);
    return () => document.removeEventListener('keydown', down);
  }, [workspace.setSidebarCollapsed]);

  // Resizing Logic
  const startResizing = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  }, []);

  const stopResizing = useCallback(() => {
    setIsResizing(false);
  }, []);

  const resize = useCallback((e: MouseEvent) => {
    if (isResizing) {
      const newWidth = Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, e.clientX));
      setSidebarWidth(newWidth);
      localStorage.setItem('sidebar-width', newWidth.toString());
    }
  }, [isResizing]);

  useEffect(() => {
    window.addEventListener('mousemove', resize);
    window.addEventListener('mouseup', stopResizing);
    return () => {
      window.removeEventListener('mousemove', resize);
      window.removeEventListener('mouseup', stopResizing);
    };
  }, [resize, stopResizing]);

  const showProjectOnboarding =
    workspace.noProject &&
    !workspace.loading &&
    routePath !== SETTINGS_ROUTE;

  if (workspace.loading) {
    return (
      <main className="main-content">
        <div className="flex h-full items-center justify-center p-8">
          <div className="text-muted-foreground text-sm">Loading workspace...</div>
        </div>
      </main>
    );
  }

  // Settings or Focus Mode: full-viewport, skip main shell chrome
  if (routePath === SETTINGS_ROUTE || workspace.isWorkspaceFocusMode) {
    return (
      <div className="h-full bg-background overflow-hidden">
        <SearchModal />
        <PageChromeProvider value={activeChrome} onUpdate={handleUpdateChrome}>
          <Outlet />
        </PageChromeProvider>
      </div>
    );
  }

  if (showProjectOnboarding) {
    return (
      <main className="main-content">
        {workspace.error && (
          <div className="mx-auto mt-3 w-full max-w-[min(86vw,1560px)] rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {workspace.error}
          </div>
        )}
        <ProjectOnboarding
          detectedProject={workspace.detectedProject}
          detectingProject={workspace.detectingProject}
          creatingProject={workspace.creatingProject}
          recentProjects={workspace.recentProjects}
          onRefreshDetection={workspace.refreshDetectedProject}
          onOpenProject={workspace.handleOpenProject}
          onCreateProject={workspace.handleCreateProjectFromForm}
          onPickDirectory={workspace.handlePickProjectDirectory}
          onSelectProject={handleSelectProject}
          onOpenSettings={() => navigateTo(SETTINGS_ROUTE)}
        />
      </main>
    );
  }

  return (
    <div
      className="app-shell"
      style={{
        gridTemplateColumns: workspace.sidebarCollapsed
          ? `${COLLAPSED_RAIL_WIDTH} minmax(0, 1fr)`
          : `${sidebarWidth}px minmax(0, 1fr)`,
      }}
    >
      <SearchModal />
      <div className="relative h-full flex overflow-hidden">
        <Sidebar
          collapsed={workspace.sidebarCollapsed}
          onToggleCollapse={() => workspace.setSidebarCollapsed((current) => !current)}
          activePath={routePath}
          onNavigate={navigateTo}
          activeProject={workspace.activeProject}
          recentProjects={workspace.recentProjects}
          onOpenProject={workspace.handleOpenProject}
          onNewProject={workspace.handleNewProject}
          onSelectProject={handleSelectProject}
          onOpenGlobalNotes={openGlobalNotes}
          sections={SHIP_NAV_SECTIONS}
          theme={workspace.config.theme as 'light' | 'dark'}
          onThemeChange={workspace.applyTheme}
          contextualContent={activeChrome.sidebar}
          onBackToGlobal={activeChrome.onBack}
          agentControl={agentControl}
        />
        {!workspace.sidebarCollapsed && (
          <div
            onMouseDown={startResizing}
            className={cn(
              "absolute right-0 top-0 bottom-0 w-1 cursor-col-resize z-50 transition-colors group",
              isResizing ? "bg-primary" : "bg-transparent hover:bg-border/50"
            )}
          >
            <div className={cn(
              "absolute right-0 top-1/2 -translate-y-1/2 w-4 h-12 -mr-2 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity",
              isResizing && "opacity-100"
            )}>
              <div className="w-1 h-6 bg-border rounded-full" />
            </div>
          </div>
        )}
      </div>

      <div className="flex h-full min-h-0 flex-col">
        {/* Top Command Bar */}
        <header className="flex h-10 shrink-0 items-center justify-between gap-3 border-b border-border/50 px-4">
          <div className="flex min-w-0 items-center gap-1">
            {activeChrome.breadcrumb}
          </div>
          <div className="flex items-center gap-1.5">
            <Button
              variant="outline"
              size="xs"
              className="h-7 gap-2 px-2.5 text-muted-foreground hover:text-foreground border-border/60"
              onClick={openCommandPalette}
            >
              <Search className="size-3" />
              <span className="text-[11px]">Search</span>
              <kbd className="pointer-events-none ml-1 inline-flex h-4 select-none items-center rounded border border-border/80 bg-muted/60 px-1 font-mono text-[9px] font-medium text-muted-foreground">
                ⌘K
              </kbd>
            </Button>
            <Button
              variant={chatOpen ? 'secondary' : 'outline'}
              size="icon-xs"
              className="size-7 border-border/60"
              onClick={() => setChatOpen((prev) => !prev)}
              title="AI Chat"
            >
              <MessageCircle className="size-3.5" />
            </Button>
          </div>
        </header>

        <div className="flex flex-1 min-h-0">
          <main className="main-content flex-1 min-w-0">
            {workspace.error && (
              <div className="mx-auto mt-2 flex w-full max-w-[min(86vw,1560px)] items-center justify-between gap-3 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                <span>{workspace.error}</span>
                <Button variant="ghost" size="icon-sm" onClick={() => workspace.setError(null)}>
                  ✕
                </Button>
              </div>
            )}
            <PageChromeProvider
              value={activeChrome}
              onUpdate={handleUpdateChrome}
            >
              <Outlet />
            </PageChromeProvider>
          </main>

          {chatOpen && (
            <aside className="flex w-80 shrink-0 flex-col border-l border-border/50 bg-card/50">
              <div className="flex h-10 items-center justify-between border-b border-border/50 px-3">
                <span className="text-xs font-semibold">AI Chat</span>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="size-6"
                  onClick={() => setChatOpen(false)}
                >
                  ✕
                </Button>
              </div>
              <div className="flex flex-1 flex-col items-center justify-center p-4 text-center">
                <MessageCircle className="size-8 text-muted-foreground/30 mb-3" />
                <p className="text-sm font-medium text-muted-foreground">AI Chat</p>
                <p className="mt-1 text-xs text-muted-foreground/70">Coming soon. Ask questions about your project, generate specs, and plan work.</p>
              </div>
            </aside>
          )}
        </div>
      </div>
    </div>
  );
}
