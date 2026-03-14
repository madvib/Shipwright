import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { Suspense, lazy } from 'react';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { AppRoutePath } from '@/lib/constants/routes';
import RouteFallback from '@/components/app/RouteFallback';

const ProjectOverview = lazy(() => import('@/features/planning/common/ProjectOverview'));

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
    <Suspense fallback={<RouteFallback label="Loading overview..." />}>
      <ProjectOverview
        project={workspace.activeProject}
        adrs={ship.adrs}
        releases={ship.releases}
        features={ship.features}
        notes={ship.notes}
        events={workspace.eventEntries}
        onNavigate={handleNavigate}
      />
    </Suspense>
  );
}

export const Route = createFileRoute('/project/overview')({
  component: OverviewRouteComponent,
});
