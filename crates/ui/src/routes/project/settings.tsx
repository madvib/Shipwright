import { createFileRoute, useNavigate } from '@tanstack/react-router';
import SettingsPanel from '@/features/agents/SettingsPanel';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { AGENTS_PROVIDERS_ROUTE, ISSUES_ROUTE } from '@/lib/constants/routes';

function SettingsRouteComponent() {
  const workspace = useWorkspace();
  const navigate = useNavigate();

  const { tab } = Route.useSearch();

  return (
    <SettingsPanel
      config={workspace.config}
      projectConfig={workspace.projectConfig}
      globalAgentConfig={workspace.globalAgentConfig}
      panelMode="settings-only"
      initialTab={tab as any}
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
        void navigate({ to: AGENTS_PROVIDERS_ROUTE });
      }}
    />
  );
}

export const Route = createFileRoute('/project/settings')({
  component: SettingsRouteComponent,
  validateSearch: (search: Record<string, unknown>) => {
    return {
      tab: (search.tab as string) || undefined,
    };
  },
});
