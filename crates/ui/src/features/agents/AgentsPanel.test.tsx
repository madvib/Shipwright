import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import AgentsPanel, { type AgentSection } from './AgentsPanel';
import { TooltipProvider } from '@ship/ui';

const mockUseAgentAssetInventory = vi.fn();
const mockUseMcpRegistrySearch = vi.fn();
const mockGetPermissionsCmd = vi.fn();

vi.mock('@/features/agents/shared/useAgentAssetInventory', () => ({
  useAgentAssetInventory: (...args: unknown[]) => mockUseAgentAssetInventory(...args),
}));

vi.mock('@/features/agents/shared/useMcpRegistrySearch', () => ({
  useMcpRegistrySearch: (...args: unknown[]) => mockUseMcpRegistrySearch(...args),
}));

vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: vi.fn(),
}));

vi.mock('@/bindings', async () => {
  const actual = await vi.importActual<typeof import('@/bindings')>('@/bindings');
  return {
    ...actual,
    commands: {
      ...actual.commands,
      listCatalogCmd: vi.fn(async () => []),
      listRulesCmd: vi.fn(async () => ({ status: 'ok', data: [] })),
      listPermissionToolVocabularyCmd: vi.fn(async () => ({ status: 'ok', data: [] })),
      validateMcpServersCmd: vi.fn(async () => ({
        status: 'ok',
        data: {
          ok: true,
          checked_servers: 0,
          checked_provider_configs: 0,
          issues: [],
        },
      })),
      getPermissionsCmd: (...args: unknown[]) => mockGetPermissionsCmd(...args),
      getAgentDiscoveryCacheCmd: vi.fn(async () => ({
        status: 'ok',
        data: {
          shell_commands: ['git', 'rg'],
          filesystem_paths: ['src/**'],
          mcp_tools: {},
        },
      })),
      savePermissionsCmd: vi.fn(async () => ({ status: 'ok', data: null })),
      refreshAgentDiscoveryCacheCmd: vi.fn(async () => ({
        status: 'ok',
        data: {
          shell_commands: ['git', 'rg'],
          filesystem_paths: ['src/**'],
          mcp_tools: {},
        },
      })),
    },
  };
});

function renderPanel(section: AgentSection) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <AgentsPanel
          activeProject={null}
          projectConfig={null}
          globalAgentConfig={null}
          initialSection={section}
          onSaveProject={vi.fn()}
          onSaveGlobalAgentConfig={vi.fn()}
        />
      </TooltipProvider>
    </QueryClientProvider>,
  );
}

describe('AgentsPanel render', () => {
  beforeEach(() => {
    mockGetPermissionsCmd.mockResolvedValue({
      status: 'ok',
      data: {
        tools: { allow: [], deny: [] },
        filesystem: { allow: [], deny: [] },
        commands: { allow: [], deny: [] },
        network: { policy: 'none', allow_hosts: [] },
        agent: { require_confirmation: [] },
      },
    });
    mockUseAgentAssetInventory.mockReturnValue({
      providers: [],
      skills: [],
      providersPending: false,
      providersError: false,
      refreshProviders: vi.fn(),
    });
    mockUseMcpRegistrySearch.mockReturnValue({
      data: [],
      isFetching: false,
      isError: false,
    });
  });

  it.each<AgentSection>(['providers', 'mcp', 'skills', 'rules', 'hooks', 'permissions'])(
    'renders %s section without crashing',
    (section) => {
      renderPanel(section);
      const title =
        section === 'providers'
          ? /providers/i
          : section === 'mcp'
            ? /mcp servers/i
            : section === 'skills'
              ? /skills/i
              : section === 'rules'
                ? /rules/i
                : section === 'hooks'
                  ? /hooks/i
            : /permissions/i;
      expect(screen.getByRole('heading', { level: 1, name: title })).toBeInTheDocument();
    },
  );

});
