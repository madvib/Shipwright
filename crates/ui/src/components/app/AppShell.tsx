import { Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import Sidebar from '@/components/app/Sidebar';
import AgentModeControl from '@/features/agents/AgentModeControl';
import IssueDetail from '@/features/planning/IssueDetail';
import NewIssueModal from '@/features/planning/NewIssueModal';
import NewAdrModal from '@/features/planning/NewAdrModal';
import AdrDetail from '@/features/planning/AdrDetail';
import ProjectOnboarding from '@/features/planning/ProjectOnboarding';
import SpecDetail from '@/features/planning/SpecDetail';
import ReleaseDetail from '@/features/planning/ReleaseDetail';
import FeatureDetail from '@/features/planning/FeatureDetail';
import { Button } from '@/components/ui/button';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import {
  AppRoutePath,
  AGENTS_ROUTE,
  ROUTE_LABELS,
  SETTINGS_ROUTE,
  OVERVIEW_ROUTE,
  PROJECTS_ROUTE,
  normalizePath,
} from '@/lib/constants/routes';

export default function App() {
  const location = useLocation();
  const navigate = useNavigate();
  const workspace = useWorkspace();
  const routePath = normalizePath(location.pathname) as AppRoutePath;

  const navigateTo = (path: AppRoutePath) => {
    if (normalizePath(location.pathname) !== path) {
      void navigate({ to: path });
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
    routePath !== AGENTS_ROUTE;

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
          <div className="mx-auto mt-6 w-full max-w-6xl rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
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

  return (
    <div
      className="app-shell"
      style={{
        gridTemplateColumns: workspace.sidebarCollapsed
          ? '4.5rem minmax(0, 1fr)'
          : '16rem minmax(0, 1fr)',
      }}
    >
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
      />

      <main className="main-content">
        {workspace.error && (
          <div className="mx-auto mt-4 flex w-full max-w-6xl items-center justify-between gap-3 rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
            <span>{workspace.error}</span>
            <Button variant="ghost" size="icon-sm" onClick={() => workspace.setError(null)}>
              ✕
            </Button>
          </div>
        )}

        {(!workspace.noProject || routePath === PROJECTS_ROUTE) && (
          <div className="mx-auto mt-4 w-full max-w-6xl px-5 md:px-6">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <nav className="text-muted-foreground flex items-center gap-1 text-xs">
                {routePath === PROJECTS_ROUTE ? (
                  <span className="text-foreground">Projects</span>
                ) : (
                  <>
                    <Button
                      variant="ghost"
                      size="xs"
                      className="h-7 px-2 text-xs"
                      onClick={() => navigateTo(PROJECTS_ROUTE)}
                    >
                      Projects
                    </Button>
                    <span>/</span>
                    <Button
                      variant="ghost"
                      size="xs"
                      className="h-7 px-2 text-xs"
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

              {!workspace.noProject && (
                <AgentModeControl
                  modes={workspace.modes}
                  activeModeId={workspace.activeModeId}
                  aiProvider={workspace.aiProvider}
                  aiModel={workspace.aiModel}
                  switchingMode={workspace.switchingMode}
                  onSetMode={workspace.handleSetActiveMode}
                  onOpenAgents={() => navigateTo(AGENTS_ROUTE)}
                />
              )}
            </div>
          </div>
        )}

        <Outlet />
      </main>

      {workspace.selectedIssue && (
        <IssueDetail
          entry={workspace.selectedIssue}
          statuses={workspace.statuses}
          onClose={() => workspace.setSelectedIssue(null)}
          onStatusChange={workspace.handleStatusChange}
          onDelete={workspace.handleDeleteIssue}
          onSave={workspace.handleSaveIssue}
          tagSuggestions={workspace.tagSuggestions}
          mcpEnabled={workspace.mcpEnabled}
        />
      )}
      {workspace.showNewIssue && (
        <NewIssueModal
          onClose={() => workspace.setShowNewIssue(false)}
          statuses={workspace.statuses}
          onSubmit={workspace.handleCreateIssue}
          defaultStatus={workspace.config.default_status ?? workspace.statuses[0]?.id}
        />
      )}
      {workspace.showNewAdr && (
        <NewAdrModal
          onClose={() => workspace.setShowNewAdr(false)}
          onSubmit={workspace.handleCreateAdr}
        />
      )}
      {workspace.selectedAdr && (
        <AdrDetail
          entry={workspace.selectedAdr}
          onClose={() => workspace.setSelectedAdr(null)}
          onSave={workspace.handleSaveAdr}
          onDelete={workspace.handleDeleteAdr}
          mcpEnabled={workspace.mcpEnabled}
        />
      )}
      {workspace.selectedSpec && (
        <SpecDetail
          spec={workspace.selectedSpec}
        onClose={() => workspace.setSelectedSpec(null)}
        onSave={workspace.handleSaveSpec}
        onDelete={workspace.handleDeleteSpec}
        mcpEnabled={workspace.mcpEnabled}
      />
      )}
      {workspace.selectedRelease && (
        <ReleaseDetail
          release={workspace.selectedRelease}
          onClose={() => workspace.setSelectedRelease(null)}
          onSave={workspace.handleSaveRelease}
          mcpEnabled={workspace.mcpEnabled}
        />
      )}
      {workspace.selectedFeature && (
        <FeatureDetail
          feature={workspace.selectedFeature}
          onClose={() => workspace.setSelectedFeature(null)}
          onSave={workspace.handleSaveFeature}
          mcpEnabled={workspace.mcpEnabled}
        />
      )}
    </div>
  );
}
