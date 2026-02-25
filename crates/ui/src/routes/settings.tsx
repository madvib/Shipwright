import { createRoute, useNavigate } from '@tanstack/react-router';
import SettingsPanel from '../components/SettingsPanel';
import { useWorkspace } from '../hooks/workspace/WorkspaceContext';
import { AGENTS_ROUTE, ISSUES_ROUTE, OVERVIEW_ROUTE } from '../lib/constants/routes';
import { rootRoute } from './__root';

function SettingsRouteComponent() {
  const workspace = useWorkspace();
  const navigate = useNavigate();

  return (
    <SettingsPanel
      config={workspace.config}
      projectConfig={workspace.projectConfig}
      globalAgentConfig={workspace.globalAgentConfig}
      panelMode="settings-only"
      onThemePreview={workspace.applyTheme}
      onSave={async (config) => {
        await workspace.handleSaveSettings(config);
        void navigate({ to: ISSUES_ROUTE });
      }}
      onSaveProject={async (config) => {
        await workspace.handleSaveProjectSettings(config);
        void navigate({ to: ISSUES_ROUTE });
      }}
      onSaveGlobalAgentConfig={async (config) => {
        await workspace.handleSaveGlobalAgentSettings(config);
        void navigate({ to: ISSUES_ROUTE });
      }}
      onOpenAgentsModule={() => {
        void navigate({ to: AGENTS_ROUTE });
      }}
      onBack={() => {
        void navigate({ to: OVERVIEW_ROUTE });
      }}
    />
  );
}

export const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/settings',
  component: SettingsRouteComponent,
});
