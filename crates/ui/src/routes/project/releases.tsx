import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { Suspense, lazy } from 'react';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { FEATURES_ROUTE } from '@/lib/constants/routes';
import RouteFallback from '@/components/app/RouteFallback';

const ReleasesPage = lazy(() => import('@/features/planning/releases/ReleasesPage'));

function ReleasesRouteComponent() {
  const workspace = useWorkspace();
  const ship = useShip();
  const navigate = useNavigate();

  return (
    <Suspense fallback={<RouteFallback label="Loading releases..." />}>
      <ReleasesPage
        releases={ship.releases}
        features={ship.features}
        selectedRelease={ship.selectedRelease}
        onCloseReleaseDetail={() => ship.setSelectedRelease(null)}
        onSelectRelease={ship.handleSelectRelease}
        onSelectFeatureFromRelease={(feature) => {
          ship.setSelectedRelease(null);
          void navigate({ to: FEATURES_ROUTE });
          void ship.handleSelectFeature(feature);
        }}
        onSaveRelease={ship.handleSaveRelease}
        onCreateRelease={ship.handleCreateRelease}
        mcpEnabled={workspace.mcpEnabled}
      />
    </Suspense>
  );
}

export const Route = createFileRoute('/project/releases')({
  component: ReleasesRouteComponent,
});
