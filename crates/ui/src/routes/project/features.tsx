import { createFileRoute, useNavigate } from '@tanstack/react-router';
import FeaturesPage from '@/features/planning/features/FeaturesPage';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { RELEASES_ROUTE } from '@/lib/constants/routes';

function FeaturesRouteComponent() {
  const workspace = useWorkspace();
  const ship = useShip();
  const navigate = useNavigate();

  return (
    <FeaturesPage
      features={ship.features}
      releases={ship.releases}
      specs={ship.specs}
      selectedFeature={ship.selectedFeature}
      onCloseFeatureDetail={() => ship.setSelectedFeature(null)}
      onSelectFeature={ship.handleSelectFeature}
      onSelectReleaseFromFeature={(name: string) => {
        const release = ship.releases.find(
          (entry) => entry.file_name === name || entry.version === name
        );
        if (!release) return;
        ship.setSelectedFeature(null);
        void navigate({ to: RELEASES_ROUTE });
        void ship.handleSelectRelease(release);
      }}
      onSelectSpecFromFeature={(name: string) => {
        const spec = ship.specs.find((entry) => entry.file_name === name);
        if (!spec) return;
        ship.setSelectedFeature(null);
        void ship.handleSelectSpec(spec);
      }}
      onSaveFeature={ship.handleSaveFeature}
      onCreateFeature={ship.handleCreateFeature}
      tagSuggestions={ship.tagSuggestions}
      mcpEnabled={workspace.mcpEnabled}
    />
  );
}

export const Route = createFileRoute('/project/features')({
  component: FeaturesRouteComponent,
});
