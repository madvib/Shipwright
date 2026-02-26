import { createRootRoute } from '@tanstack/react-router';
import { WorkspaceProvider } from '@/lib/hooks/workspace/WorkspaceContext';
import AppShell from '@/components/app/AppShell';

function RootLayout() {
  return (
    <WorkspaceProvider>
      <AppShell />
    </WorkspaceProvider>
  );
}

export const Route = createRootRoute({
  component: RootLayout,
});
