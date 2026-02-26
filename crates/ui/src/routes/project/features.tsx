import { createFileRoute } from '@tanstack/react-router';
import FeaturesPage from '@/features/planning/FeaturesPage';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';

function FeaturesRouteComponent() {
  const workspace = useWorkspace();

  return (
    <FeaturesPage
      features={workspace.features}
      releases={workspace.releases}
      specs={workspace.specs}
      onSelectFeature={workspace.handleSelectFeature}
      onCreateFeature={workspace.handleCreateFeature}
    />
  );
}

export const Route = createFileRoute('/project/features')({
  component: FeaturesRouteComponent,
});
