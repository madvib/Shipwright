import { createFileRoute, useNavigate } from '@tanstack/react-router';
import ProjectOverview from '@/features/planning/common/ProjectOverview';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { AppRoutePath } from '@/lib/constants/routes';

function OverviewRouteComponent() {
  const workspace = useWorkspace();
  const ship = useShip();
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
      specs={ship.specs}
      adrs={ship.adrs}
      releases={ship.releases}
      features={ship.features}
      notes={ship.notes}
      events={workspace.eventEntries}
      onNavigate={handleNavigate}
    />
  );
}

export const Route = createFileRoute('/project/overview')({
  component: OverviewRouteComponent,
});
