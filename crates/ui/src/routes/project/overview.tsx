import { createFileRoute, useNavigate } from '@tanstack/react-router';
import ProjectOverview from '@/features/planning/ProjectOverview';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { AppRoutePath } from '@/lib/constants/routes';

function OverviewRouteComponent() {
  const workspace = useWorkspace();
  const navigate = useNavigate();

  if (!workspace.activeProject) {
    return null;
  }

  const handleNavigate = (to: AppRoutePath) => {
    void navigate({ to });
  };

  return (
    <ProjectOverview
      project={workspace.activeProject}
      issues={workspace.issues}
      specs={workspace.specs}
      adrs={workspace.adrs}
      releases={workspace.releases}
      features={workspace.features}
      events={workspace.eventEntries}
      statuses={workspace.statuses}
      onNavigate={handleNavigate}
    />
  );
}

export const Route = createFileRoute('/project/overview')({
  component: OverviewRouteComponent,
});
