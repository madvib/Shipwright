import { createFileRoute } from '@tanstack/react-router';
import ReleasesPage from '@/features/planning/ReleasesPage';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';

function ReleasesRouteComponent() {
  const workspace = useWorkspace();

  return (
    <ReleasesPage
      releases={workspace.releases}
      features={workspace.features}
      onSelectRelease={workspace.handleSelectRelease}
      onCreateRelease={workspace.handleCreateRelease}
    />
  );
}

export const Route = createFileRoute('/project/releases')({
  component: ReleasesRouteComponent,
});
