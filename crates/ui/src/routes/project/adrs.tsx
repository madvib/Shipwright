import { createFileRoute } from '@tanstack/react-router';
import AdrList from '@/features/planning/AdrList';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';

function AdrsRouteComponent() {
  const workspace = useWorkspace();

  return (
    <AdrList
      adrs={workspace.adrs}
      selectedAdr={workspace.selectedAdr}
      onCreateAdr={workspace.handleCreateAdr}
      onSelectAdr={workspace.handleSelectAdr}
      onMoveAdr={workspace.handleMoveAdr}
      onSaveAdr={workspace.handleSaveAdr}
      onDeleteAdr={workspace.handleDeleteAdr}
      specSuggestions={workspace.specSuggestions}
      tagSuggestions={workspace.tagSuggestions}
      adrSuggestions={workspace.adrSuggestions}
      mcpEnabled={workspace.mcpEnabled}
    />
  );
}

export const Route = createFileRoute('/project/adrs')({
  component: AdrsRouteComponent,
});
