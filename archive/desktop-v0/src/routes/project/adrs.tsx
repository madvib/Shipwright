import { Suspense, lazy, useCallback } from 'react';
import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { OVERVIEW_ROUTE } from '@/lib/constants/routes';
import RouteFallback from '@/components/app/RouteFallback';

const AdrList = lazy(() => import('@/features/planning/adrs/AdrList'));

function AdrsRouteComponent() {
  const workspace = useWorkspace();
  const ship = useShip();
  const navigate = useNavigate();

  const onBack = useCallback(() => {
    void navigate({ to: OVERVIEW_ROUTE });
  }, [navigate]);

  return (
    <Suspense fallback={<RouteFallback label="Loading decisions..." />}>
      <AdrList
        adrs={ship.adrs}
        selectedAdr={ship.selectedAdr}
        onCreateAdr={ship.handleCreateAdr}
        onSelectAdr={ship.handleSelectAdr}
        onMoveAdr={ship.handleMoveAdr}
        onSaveAdr={ship.handleSaveAdr}
        onDeleteAdr={ship.handleDeleteAdr}
        tagSuggestions={ship.tagSuggestions}
        adrSuggestions={ship.adrSuggestions}
        mcpEnabled={workspace.mcpEnabled}
        onBackToGlobal={onBack}
      />
    </Suspense>
  );
}

export const Route = createFileRoute('/project/adrs')({
  component: AdrsRouteComponent,
});
