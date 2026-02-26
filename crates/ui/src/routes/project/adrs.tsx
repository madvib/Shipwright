import { createFileRoute } from '@tanstack/react-router';
import AdrList from '@/features/planning/AdrList';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';

function AdrsRouteComponent() {
  const workspace = useWorkspace();

  return (
    <AdrList
      adrs={workspace.adrs}
      onNewAdr={() => workspace.setShowNewAdr(true)}
      onSelectAdr={workspace.handleSelectAdr}
    />
  );
}

export const Route = createFileRoute('/project/adrs')({
  component: AdrsRouteComponent,
});
