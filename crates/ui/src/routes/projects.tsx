import { createFileRoute, useNavigate } from '@tanstack/react-router';
import ProjectOnboarding from '@/features/planning/common/ProjectOnboarding';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { OVERVIEW_ROUTE, SETTINGS_ROUTE } from '@/lib/constants/routes';

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
  );
}

export const Route = createFileRoute('/projects')({
  component: ProjectsRouteComponent,
});
