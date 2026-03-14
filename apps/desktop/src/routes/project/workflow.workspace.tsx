import { createFileRoute } from '@tanstack/react-router';
import { Suspense, lazy } from 'react';
import RouteFallback from '@/components/app/RouteFallback';

const WorkspacePanel = lazy(() => import('@/features/workflow/WorkspacePanel'));

export const Route = createFileRoute('/project/workflow/workspace')({
    component: () => (
      <Suspense fallback={<RouteFallback label="Loading workspaces..." />}>
        <WorkspacePanel />
      </Suspense>
    ),
});
