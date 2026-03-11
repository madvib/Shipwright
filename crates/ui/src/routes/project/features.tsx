import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { Suspense, lazy } from 'react';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { RELEASES_ROUTE } from '@/lib/constants/routes';
import RouteFallback from '@/components/app/RouteFallback';

const FeaturesPage = lazy(() => import('@/features/planning/features/FeaturesPage'));

function FeaturesRouteComponent() {
  const workspace = useWorkspace();
  const ship = useShip();
  const navigate = useNavigate();

  return (
    <Suspense fallback={<RouteFallback label="Loading features..." />}>
      <FeaturesPage
        features={ship.features}
        releases={ship.releases}
        selectedFeature={ship.selectedFeature}
        onCloseFeatureDetail={() => ship.setSelectedFeature(null)}
        onSelectFeature={ship.handleSelectFeature}
        onStartFeature={ship.handleStartFeature}
        onDoneFeature={ship.handleDoneFeature}
        onSaveFeatureDocumentation={ship.handleSaveFeatureDocumentation}
        onSelectReleaseFromFeature={(name: string) => {
          const release = ship.releases.find(
            (entry) => entry.file_name === name || entry.version === name
          );
          if (!release) return;
          ship.setSelectedFeature(null);
          void navigate({ to: RELEASES_ROUTE });
          void ship.handleSelectRelease(release);
        }}
        onSaveFeature={ship.handleSaveFeature}
        onCreateFeature={ship.handleCreateFeature}
        tagSuggestions={ship.tagSuggestions}
        mcpEnabled={workspace.mcpEnabled}
      />
    </Suspense>
  );
}

export const Route = createFileRoute('/project/features')({
  component: FeaturesRouteComponent,
});
