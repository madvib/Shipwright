import { useMemo, useState } from 'react';
import { Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import { useUpdateChecker } from '@/lib/hooks/useUpdateChecker';
import Sidebar from '@/components/app/Sidebar';
import AgentModeControl from '@/features/agents/AgentModeControl';
import { PageChromeProvider, PageChromeContextValue } from '@/components/app/PageFrame';
import IssueDetail from '@/features/planning/IssueDetail';
import NewIssueModal from '@/features/planning/NewIssueModal';
import ProjectOnboarding from '@/features/planning/ProjectOnboarding';
import SpecDetail from '@/features/planning/SpecDetail';
import { SearchModal } from '@/components/app/SearchModal';
import { Button } from '@ship/ui';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import {
  AppRoutePath,
  AGENTS_MCP_ROUTE,
  AGENTS_PERMISSIONS_ROUTE,
  AGENTS_PROVIDERS_ROUTE,
  AGENTS_RULES_ROUTE,
  AGENTS_ROUTE,
  AGENTS_SKILLS_ROUTE,
  FEATURES_ROUTE,
  NOTES_ROUTE,
  ROUTE_LABELS,
  SETTINGS_ROUTE,
  OVERVIEW_ROUTE,
  PROJECTS_ROUTE,
  WORKFLOW_WORKSPACE_ROUTE,
  normalizePath,
} from '@/lib/constants/routes';
import {
  Bot,
  FileCode2,
  FileCog,
  FileStack,
  Package,
  Workflow,
} from 'lucide-react';
import { NavSection } from '@/lib/types/navigation';
import { SHIP_NAV_SECTIONS } from '@/lib/modules/ship';

export default function App() {
  useUpdateChecker();
  const location = useLocation();
  const navigate = useNavigate();
  const workspace = useWorkspace();
  const ship = useShip();
  const routePath = normalizePath(location.pathname) as AppRoutePath;

  const SHELL_SECTIONS: NavSection[] = [
    {
      id: 'workflow',
      label: 'Workflow',
      items: [
        { id: 'workspaces', path: WORKFLOW_WORKSPACE_ROUTE, label: 'Workspaces', icon: Workflow },
      ],
    },
    {
      id: 'agents',
      label: 'Agents',
      items: [
        { id: 'providers', path: AGENTS_PROVIDERS_ROUTE, label: 'Providers', icon: Bot },
        { id: 'mcp', path: AGENTS_MCP_ROUTE, label: 'MCP Servers', icon: Package },
        { id: 'skills', path: AGENTS_SKILLS_ROUTE, label: 'Skills', icon: FileStack },
        { id: 'rules', path: AGENTS_RULES_ROUTE, label: 'Rules', icon: FileCode2 },
        { id: 'permissions', path: AGENTS_PERMISSIONS_ROUTE, label: 'Permissions', icon: FileCog },
      ],
    },
  ];

  const COMBINED_SECTIONS = useMemo(() => {
    // Start with a clone of shell sections to avoid accidental mutation
    const sectionsMap = new Map<string, NavSection>();

    // Process ship modules first to establish primary section grouping
    SHIP_NAV_SECTIONS.forEach(shipSection => {
      sectionsMap.set(shipSection.id, { ...shipSection, items: [...shipSection.items] });
    });

    // Merge shell sections into the map
    SHELL_SECTIONS.forEach(shellSection => {
      const existing = sectionsMap.get(shellSection.id);
      if (existing) {
        // Merge items, deduplicating by ID
        const existingIds = new Set(existing.items.map(i => i.id));
        const newItems = shellSection.items.filter(i => !existingIds.has(i.id));
        existing.items = [...existing.items, ...newItems];
      } else {
        sectionsMap.set(shellSection.id, { ...shellSection, items: [...shellSection.items] });
      }
    });

    return Array.from(sectionsMap.values());
  }, [SHIP_NAV_SECTIONS, SHELL_SECTIONS]);

  const navigateTo = (path: AppRoutePath) => {
    if (path === NOTES_ROUTE) {
      workspace.setNotesScope('project');
    }
    if (normalizePath(location.pathname) !== path) {
      void navigate({ to: path });
    }
  };

  const openGlobalNotes = () => {
    workspace.setNotesScope('global');
    if (normalizePath(location.pathname) !== NOTES_ROUTE) {
      void navigate({ to: NOTES_ROUTE });
    }
  };

  const handleSelectProject = async (project: Parameters<typeof workspace.handleSelectProject>[0]) => {
    const selected = await workspace.handleSelectProject(project);
    if (selected) {
      navigateTo(OVERVIEW_ROUTE);
    }
  };

  const showProjectOnboarding =
    workspace.noProject &&
    !workspace.loading &&
    routePath !== SETTINGS_ROUTE &&
    routePath !== AGENTS_ROUTE &&
    routePath !== AGENTS_PROVIDERS_ROUTE &&
    routePath !== AGENTS_MCP_ROUTE &&
    routePath !== AGENTS_SKILLS_ROUTE &&
    routePath !== AGENTS_RULES_ROUTE &&
    routePath !== AGENTS_PERMISSIONS_ROUTE;

  if (workspace.loading) {
    return (
      <main className="main-content">
        <div className="flex h-full items-center justify-center p-8">
          <div className="text-muted-foreground text-sm">Loading workspace...</div>
        </div>
      </main>
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
          globalConfig={workspace.config}
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

  const [pageChrome, setPageChrome] = useState<Partial<PageChromeContextValue>>({});

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
                onClick={() => navigateTo(OVERVIEW_ROUTE)}
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
    : {}), [workspace.noProject, workspace.activeProject, routePath, PROJECTS_ROUTE, OVERVIEW_ROUTE, ROUTE_LABELS, navigateTo]);

  const activeChrome = useMemo(() => ({
    ...defaultChrome,
    ...pageChrome,
  }), [defaultChrome, pageChrome]);

  const handleUpdateChrome = (updates: Partial<PageChromeContextValue>) => {
    setPageChrome(prev => ({ ...prev, ...updates }));
  };

  return (
    <div
      className="app-shell"
      style={{
        gridTemplateColumns: workspace.sidebarCollapsed
          ? '4.5rem minmax(0, 1fr)'
          : '16rem minmax(0, 1fr)',
      }}
    >
      <SearchModal />
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
        sections={COMBINED_SECTIONS}
        theme={workspace.config.theme as 'light' | 'dark'}
        onThemeChange={workspace.applyTheme}
        contextualContent={activeChrome.sidebar}
        onBackToGlobal={activeChrome.onBack}
        agentControl={
          !workspace.noProject ? (
            <AgentModeControl
              modes={workspace.modes}
              activeModeId={workspace.activeModeId}
              aiProvider={workspace.aiProvider}
              aiModel={workspace.aiModel}
              switchingMode={workspace.switchingMode}
              onSetMode={workspace.handleSetActiveMode}
              onOpenAgents={() => navigateTo(AGENTS_PROVIDERS_ROUTE)}
            />
          ) : null
        }
      />

      <main className="main-content">
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

      {ship.selectedIssue && (
        <IssueDetail
          entry={ship.selectedIssue}
          statuses={workspace.statuses}
          onClose={() => ship.setSelectedIssue(null)}
          onStatusChange={ship.handleStatusChange}
          onDelete={ship.handleDeleteIssue}
          onSave={ship.handleSaveIssue}
          tagSuggestions={ship.tagSuggestions}
          specSuggestions={ship.specSuggestions}
          issueSuggestions={ship.issueFileSuggestions}
          mcpEnabled={workspace.mcpEnabled}
        />
      )}
      {ship.showNewIssue && (
        <NewIssueModal
          onClose={() => ship.setShowNewIssue(false)}
          statuses={workspace.statuses}
          tagSuggestions={ship.tagSuggestions}
          specSuggestions={ship.specSuggestions}
          onSubmit={ship.handleCreateIssue}
          defaultStatus={workspace.config.default_status ?? workspace.statuses[0]?.id}
        />
      )}
      {ship.selectedSpec && (
        <SpecDetail
          spec={ship.selectedSpec}
          features={ship.features}
          tagSuggestions={ship.tagSuggestions}
          onClose={() => ship.setSelectedSpec(null)}
          onSelectFeature={(f) => {
            ship.setSelectedSpec(null);
            void navigate({ to: FEATURES_ROUTE });
            void ship.handleSelectFeature(f);
          }}
          onSave={ship.handleSaveSpec}
          onDelete={ship.handleDeleteSpec}
          mcpEnabled={workspace.mcpEnabled}
        />
      )}
    </div>
  );
}
