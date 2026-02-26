import { createFileRoute, useNavigate } from '@tanstack/react-router';
import ProjectsDashboard from '@/features/planning/ProjectsDashboard';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { OVERVIEW_ROUTE } from '@/lib/constants/routes';

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
    <ProjectsDashboard
      projects={workspace.recentProjects}
      activeProject={workspace.activeProject}
      onSelectProject={handleSelectProject}
      onOpenProject={workspace.handleOpenProject}
      onNewProject={workspace.handleNewProject}
    />
  );
}

export const Route = createFileRoute('/projects')({
  component: ProjectsRouteComponent,
});
