import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { Suspense, lazy } from 'react';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { OVERVIEW_ROUTE } from '@/lib/constants/routes';
import type { SettingsSection } from '@/features/settings/SettingsLayout';
import RouteFallback from '@/components/app/RouteFallback';

const SettingsLayout = lazy(() => import('@/features/settings/SettingsLayout'));
const VALID_SETTINGS_SECTIONS: SettingsSection[] = [
  'global',
  'project',
  'appearance',
  'providers',
  'mcp',
  'skills',
  'rules',
  'permissions',
];

function SettingsRouteComponent() {
  const workspace = useWorkspace();
  const navigate = useNavigate();
  const { tab } = Route.useSearch();

  const activeSection = VALID_SETTINGS_SECTIONS.includes(tab as SettingsSection)
    ? (tab as SettingsSection)
    : 'global';

  return (
    <Suspense fallback={<RouteFallback label="Loading settings..." />}>
      <SettingsLayout
        config={workspace.config}
        activeProject={workspace.activeProject}
        projectConfig={workspace.projectConfig}
        globalAgentConfig={workspace.globalAgentConfig}
        recentProjects={workspace.recentProjects}
        onOpenProject={workspace.handleOpenProject}
        onSelectProject={workspace.handleSelectProject}
        activeSection={activeSection}
        onSectionChange={(section) => {
          void navigate({ to: '/project/settings', search: { tab: section } });
        }}
        onThemePreview={workspace.applyTheme}
        onSave={async (config) => {
          await workspace.handleSaveSettings(config);
        }}
        onSaveProject={async (config) => {
          await workspace.handleSaveProjectSettings(config);
        }}
        onSaveGlobalAgentConfig={async (config) => {
          await workspace.handleSaveGlobalAgentSettings(config);
        }}
        onDone={() => {
          void navigate({ to: OVERVIEW_ROUTE });
        }}
      />
    </Suspense>
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
