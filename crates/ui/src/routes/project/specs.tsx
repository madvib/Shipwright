import { createFileRoute } from '@tanstack/react-router';
import SpecsPage from '@/features/planning/SpecsPage';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';

function SpecsRouteComponent() {
  const workspace = useWorkspace();

  return (
    <SpecsPage
      specs={workspace.specs}
      onSelectSpec={workspace.handleSelectSpec}
      onCreateSpec={workspace.handleCreateSpec}
    />
  );
}

export const Route = createFileRoute('/project/specs')({
  component: SpecsRouteComponent,
});
