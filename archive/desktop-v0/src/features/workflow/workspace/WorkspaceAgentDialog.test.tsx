import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { WorkspaceAgentDialog } from './WorkspaceAgentDialog';
import type { ProviderInfo } from '@/bindings';

const mockUseAgentAssetInventory = vi.fn();
const mockUseMcpRegistrySearch = vi.fn();

vi.mock('@/features/agents/shared/useAgentAssetInventory', () => ({
  useAgentAssetInventory: (...args: unknown[]) => mockUseAgentAssetInventory(...args),
}));

vi.mock('@/features/agents/shared/useMcpRegistrySearch', () => ({
  useMcpRegistrySearch: (...args: unknown[]) => mockUseMcpRegistrySearch(...args),
}));

const PROVIDER: ProviderInfo = {
  id: 'codex',
  name: 'Codex CLI',
  binary: 'codex',
  project_config: '.codex/config.toml',
  global_config: '~/.codex/config.toml',
  config_format: 'toml',
  prompt_output: 'AGENTS.md',
  skills_output: '.agents/skills',
  enabled: true,
  installed: true,
  version: '0.1.0',
  models: [],
};

describe('WorkspaceAgentDialog', () => {
  beforeEach(() => {
    mockUseAgentAssetInventory.mockReturnValue({
      providers: [],
      mcpServers: [{ id: 'ship', label: 'Ship MCP' }],
      skills: [
        { id: 'ship-workflow', name: 'Ship Workflow', description: 'Workflow policy and guardrails' },
        { id: 'task-policy', name: 'Task Policy', description: 'Execution policy' },
        { id: 'start-session', name: 'Start Session', description: 'Session bootstrap' },
        { id: 'workspace-session-lifecycle', name: 'Workspace Session Lifecycle', description: 'Lifecycle operations' },
        { id: 'custom-skill', name: 'Custom Skill', description: 'User skill' },
      ],
      loading: false,
      error: null,
      providersPending: false,
      providersError: false,
      refreshProviders: vi.fn(),
    });
    mockUseMcpRegistrySearch.mockReturnValue({
      data: [],
      isFetching: false,
    });
  });

  it('uses fixed dialog sizing and workspace required skill chips', () => {
    render(
      <WorkspaceAgentDialog
        open
        branch="feature/agent-config"
        workspaceType="feature"
        providerInfos={[PROVIDER]}
        currentProviders={['codex']}
        currentMcpServers={['ship']}
        currentSkills={[]}
        saving={false}
        onOpenChange={vi.fn()}
        onSave={vi.fn()}
      />,
    );

    const dialog = document.querySelector('[data-slot="dialog-content"]');
    expect(dialog).not.toBeNull();
    expect(dialog?.className).toContain('h-[min(84vh,760px)]');
    expect(dialog?.className).toContain('w-[min(1120px,calc(100vw-1.5rem))]');

    expect(screen.getAllByText('ship-workflow').length).toBeGreaterThan(0);
    expect(screen.getAllByText('task-policy').length).toBeGreaterThan(0);
    expect(screen.getAllByText('start-session').length).toBeGreaterThan(0);
  });

  it('saves required skills for feature workspaces and closes dialog', async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    const onOpenChange = vi.fn();
    render(
      <WorkspaceAgentDialog
        open
        branch="feature/agent-config"
        workspaceType="feature"
        providerInfos={[PROVIDER]}
        currentProviders={['codex']}
        currentMcpServers={['ship']}
        currentSkills={['custom-skill']}
        saving={false}
        onOpenChange={onOpenChange}
        onSave={onSave}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: /save workspace agent/i }));

    await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
    expect(onSave).toHaveBeenCalledWith({
      providers: ['codex'],
      mcpServers: ['ship'],
      skills: expect.arrayContaining(['ship-workflow', 'task-policy', 'start-session', 'custom-skill']),
    });
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('links to MCP settings and permissions from manual setup actions', () => {
    const onOpenChange = vi.fn();
    const onOpenMcpSettings = vi.fn();
    const onOpenPermissionsSettings = vi.fn();
    render(
      <WorkspaceAgentDialog
        open
        branch="feature/agent-config"
        workspaceType="feature"
        providerInfos={[PROVIDER]}
        currentProviders={['codex']}
        currentMcpServers={['ship']}
        currentSkills={[]}
        saving={false}
        onOpenChange={onOpenChange}
        onOpenMcpSettings={onOpenMcpSettings}
        onOpenPermissionsSettings={onOpenPermissionsSettings}
        onSave={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: /mcp settings/i }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onOpenMcpSettings).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByRole('button', { name: /permissions/i }));
    expect(onOpenPermissionsSettings).toHaveBeenCalledTimes(1);
  });
});
