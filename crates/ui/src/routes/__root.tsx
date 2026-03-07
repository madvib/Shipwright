import { createRootRoute } from '@tanstack/react-router';
import { WorkspaceProvider } from '@/lib/hooks/workspace/WorkspaceContext';
import AppShell from '@/components/app/AppShell';
import { TooltipProvider } from '@ship/ui';

function RootLayout() {
  return (
    <TooltipProvider delayDuration={400}>
      <WorkspaceProvider>
        <AppShell />
      </WorkspaceProvider>
    </TooltipProvider>
  );
}

export const Route = createRootRoute({
  component: RootLayout,
});
