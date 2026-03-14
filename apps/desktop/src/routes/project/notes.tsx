import { createFileRoute } from '@tanstack/react-router';
import { Suspense, lazy } from 'react';
import RouteFallback from '@/components/app/RouteFallback';

const NotesPage = lazy(() => import('@/features/planning/notes/NotesPage'));

export const Route = createFileRoute('/project/notes')({
    component: () => (
      <Suspense fallback={<RouteFallback label="Loading notes..." />}>
        <NotesPage />
      </Suspense>
    ),
});
