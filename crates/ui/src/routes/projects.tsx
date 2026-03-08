import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { Suspense, lazy } from 'react';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { OVERVIEW_ROUTE, SETTINGS_ROUTE } from '@/lib/constants/routes';
import RouteFallback from '@/components/app/RouteFallback';

const ProjectOnboarding = lazy(() => import('@/features/planning/common/ProjectOnboarding'));

function ProjectsRouteComponent() {
  const workspace = useWorkspace();
  const navigate = useNavigate();

  const handleSelectProject = async (project: Parameters<typeof workspace.handleSelectProject>[0]) => {
    const selected = await workspace.handleSelectProject(project);
    if (selected) {
      void navigate({ to: OVERVIEW_ROUTE });
    }
  };

  return (
    <Suspense fallback={<RouteFallback label="Loading projects..." />}>
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
        onOpenSettings={(tab) => navigate({ to: SETTINGS_ROUTE, search: { tab } })}
      />
    </Suspense>
  );
}

export const Route = createFileRoute('/projects')({
  component: ProjectsRouteComponent,
});
