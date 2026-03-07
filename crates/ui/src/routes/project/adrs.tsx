import { useCallback } from 'react';
import { createFileRoute, useNavigate } from '@tanstack/react-router';
import AdrList from '@/features/planning/adrs/AdrList';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { WORKFLOW_WORKSPACE_ROUTE } from '@/lib/constants/routes';

function AdrsRouteComponent() {
  const workspace = useWorkspace();
  const ship = useShip();
  const navigate = useNavigate();

  const onBack = useCallback(() => {
    void navigate({ to: WORKFLOW_WORKSPACE_ROUTE });
  }, [navigate]);

  return (
    <AdrList
      adrs={ship.adrs}
      selectedAdr={ship.selectedAdr}
      onCreateAdr={ship.handleCreateAdr}
      onSelectAdr={ship.handleSelectAdr}
      onMoveAdr={ship.handleMoveAdr}
      onSaveAdr={ship.handleSaveAdr}
      onDeleteAdr={ship.handleDeleteAdr}
      specSuggestions={ship.specSuggestions}
      tagSuggestions={ship.tagSuggestions}
      adrSuggestions={ship.adrSuggestions}
      mcpEnabled={workspace.mcpEnabled}
      onBackToGlobal={onBack}
    />
  );
}

export const Route = createFileRoute('/project/adrs')({
  component: AdrsRouteComponent,
});
