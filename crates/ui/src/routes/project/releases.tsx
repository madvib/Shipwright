import { createFileRoute, useNavigate } from '@tanstack/react-router';
import ReleasesPage from '@/features/planning/ReleasesPage';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { FEATURES_ROUTE } from '@/lib/constants/routes';

function ReleasesRouteComponent() {
  const workspace = useWorkspace();
  const ship = useShip();
  const navigate = useNavigate();

  return (
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
  );
}

export const Route = createFileRoute('/project/releases')({
  component: ReleasesRouteComponent,
});
