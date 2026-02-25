import { createRoute } from '@tanstack/react-router';
import AgentsPanel from '../components/AgentsPanel';
import { useWorkspace } from '../hooks/workspace/WorkspaceContext';
import { rootRoute } from './__root';

function AgentsRouteComponent() {
  const workspace = useWorkspace();

  return (
    <AgentsPanel
      projectConfig={workspace.projectConfig}
      globalAgentConfig={workspace.globalAgentConfig}
      onSaveProject={workspace.handleSaveProjectSettings}
      onSaveGlobalAgentConfig={workspace.handleSaveGlobalAgentSettings}
    />
  );
}

export const agentsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/agents',
  component: AgentsRouteComponent,
});
