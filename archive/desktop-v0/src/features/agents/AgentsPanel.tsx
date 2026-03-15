import { useEffect, useMemo, useRef, useState } from 'react';
import { ArrowLeft, Globe, Folder } from 'lucide-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { commands, AgentDiscoveryCache, HookConfig, McpProbeReport, McpServerConfig, McpValidationReport, Permissions, ProjectConfig } from '@/bindings';
import { DEFAULT_STATUSES } from '@/lib/workspace-ui';
import { Alert, AlertDescription } from '@ship/primitives';
import { Button } from '@ship/primitives';
import { PageFrame, PageHeader } from '@ship/primitives';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';
import { cn } from '@/lib/utils';
import { useAgentAssetInventory } from '@/features/agents/shared/useAgentAssetInventory';
import { useMcpRegistrySearch } from '@/features/agents/shared/useMcpRegistrySearch';
import {
  type AgentDoc,
  type AgentSection,
  type AgentsPanelProps,
  type McpEditDraft,
  type MarkdownDocKind,
  type ProviderRow,
  type ScopeKey,
  EMPTY_AGENT_LAYER,
  EMPTY_CATALOG,
  EMPTY_MCP_SERVER,
  EMPTY_RULES,
  HOOK_EVENTS,
  MCP_STDIO_ONLY_ALPHA,
  SECTION_META,
  SUPPORTED_PROVIDER_BASE,
  SUPPORTED_PROVIDER_IDS,
  inferMcpServerId,
  slugifyId,
  splitShellArgs,
} from './agents.types';

import { ProvidersSection } from './sections/ProvidersSection';
import { McpServersSection } from './sections/McpServersSection';
import { SkillsSection } from './sections/SkillsSection';
import { HooksSection } from './sections/HooksSection';
import { PermissionsSection } from './sections/PermissionsSection';

// Re-export AgentSection so existing consumers (SettingsLayout, tests) can still import it here.
export type { AgentSection } from './agents.types';

// ── Helpers ─────────────────────────────────────────────────────────────────

function normalizeAiConfig(ai: ProjectConfig['ai']) {
  return {
    provider: ai?.provider ?? 'claude',
    model: ai?.model ?? null,
    cli_path: ai?.cli_path ?? null,
  };
}

function normalizeProjectConfig(config: ProjectConfig | null): ProjectConfig {
  return {
    version: config?.version ?? '1',
    name: config?.name ?? null,
    description: config?.description ?? null,
    statuses: (config?.statuses?.length ? config.statuses : DEFAULT_STATUSES).map((status) => ({
      id: status.id,
      name: status.name,
      color: status.color ?? 'gray',
    })),
    git: {
      ignore: config?.git?.ignore ?? [],
      commit:
        config?.git?.commit ?? ['releases', 'features', 'adrs', 'ship.toml', 'templates'],
    },
    ai: normalizeAiConfig(config?.ai ?? null),
    modes: config?.modes ?? [],
    mcp_servers: config?.mcp_servers ?? [],
    active_mode: config?.active_mode ?? null,
    agent: {
      ...EMPTY_AGENT_LAYER,
      ...(config?.agent ?? {}),
    },
    hooks: config?.hooks ?? [],
    providers: config?.providers ?? ['claude'],
  };
}

type SkillSourceInstallSpec = {
  source: string;
  skillId: string;
  parseHint: string | null;
  canInstall: boolean;
};

function parseSkillSourceInstallSpec(rawSource: string): SkillSourceInstallSpec {
  const trimmed = rawSource.trim();
  if (!trimmed) {
    return { source: '', skillId: '', parseHint: null, canInstall: false };
  }

  const shellTokens = splitShellArgs(trimmed);
  const startsAsCommand = shellTokens.length > 0
    && ['npx', 'skills', 'skills.sh'].includes(shellTokens[0].toLowerCase());
  const addIndex = shellTokens.findIndex((token) => token.toLowerCase() === 'add');
  const maybeSkillsCommand = shellTokens.some((token) => {
    const normalized = token.toLowerCase();
    return normalized === 'skills'
      || normalized === 'skills.sh'
      || normalized === '@skills/cli'
      || normalized.endsWith('/skills');
  });
  if (startsAsCommand || maybeSkillsCommand || addIndex >= 0) {
    if (addIndex < 0) {
      return {
        source: trimmed,
        skillId: '',
        parseHint: 'skills.sh command must include `add` (example: npx skills add <skill-id>).',
        canInstall: false,
      };
    }
    let hasSkillTarget = false;
    for (let index = addIndex + 1; index < shellTokens.length; index += 1) {
      const token = shellTokens[index];
      if (!token) continue;
      if (token === '--skill' && shellTokens[index + 1]) {
        hasSkillTarget = true;
        break;
      }
      if (!token.startsWith('-')) {
        hasSkillTarget = true;
        break;
      }
    }
    if (!hasSkillTarget) {
      return {
        source: trimmed,
        skillId: '',
        parseHint: 'skills.sh command is missing a skill target after `add`.',
        canInstall: false,
      };
    }
    let parsedSkillId = '';
    for (let index = addIndex + 1; index < shellTokens.length; index += 1) {
      const token = shellTokens[index];
      if (!token) continue;
      const normalized = token.toLowerCase();
      if ((normalized === '--skill' || normalized === '-s') && shellTokens[index + 1]) {
        const candidate = shellTokens[index + 1].trim();
        if (candidate && !candidate.startsWith('-')) {
          parsedSkillId = slugifyId(candidate);
          break;
        }
      }
      if (normalized.startsWith('--skill=')) {
        const candidate = token.slice(token.indexOf('=') + 1).trim();
        if (candidate) {
          parsedSkillId = slugifyId(candidate);
          break;
        }
      }
    }
    return {
      source: trimmed,
      skillId: parsedSkillId,
      parseHint: parsedSkillId
        ? `Using skills.sh command for "${parsedSkillId}".`
        : 'Using skills.sh command. Ship will auto-detect the installed skill package.',
      canInstall: true,
    };
  }

  const ownerRepoWithSkill = trimmed.match(/^([^/\s]+\/[^@\s]+)@([a-z0-9-]+)$/i);
  if (ownerRepoWithSkill) {
    return {
      source: trimmed,
      skillId: slugifyId(ownerRepoWithSkill[2]),
      parseHint: 'Using skills.sh ID shorthand.',
      canInstall: true,
    };
  }

  const inferredId = slugifyId(trimmed.includes('@') ? trimmed.split('@').pop() ?? '' : trimmed);
  return {
    source: trimmed,
    skillId: inferredId,
    parseHint: inferredId
      ? 'Using skill ID shorthand.'
      : 'Invalid input. Paste a skills.sh command or enter a skill ID.',
    canInstall: inferredId.length > 0,
  };
}

function mcpToolPattern(serverId: string, toolName: string): string {
  return `mcp__${serverId}__${toolName}`;
}

function normalizePermissionsForEditor(input?: Permissions | null): Permissions {
  const normalizePatternList = (values?: string[] | null): string[] =>
    Array.from(
      new Set(
        (values ?? [])
          .map((value) => value.trim())
          .filter((value) => value.length > 0)
      )
    );
  return {
    tools: {
      allow: normalizePatternList(input?.tools?.allow),
      deny: normalizePatternList(input?.tools?.deny),
    },
    filesystem: {
      allow: normalizePatternList(input?.filesystem?.allow),
      deny: normalizePatternList(input?.filesystem?.deny),
    },
    commands: {
      allow: normalizePatternList(input?.commands?.allow),
      deny: normalizePatternList(input?.commands?.deny),
    },
    network: {
      policy: input?.network?.policy ?? 'none',
      allow_hosts: normalizePatternList(input?.network?.allow_hosts),
    },
    agent: {
      require_confirmation: normalizePatternList(input?.agent?.require_confirmation),
    },
  };
}

function extractMarkdownHeadingTitle(content: string): string | null {
  const lines = content.split(/\r?\n/);
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    const match = trimmed.match(/^#\s+(.+)$/);
    if (!match) continue;
    const title = match[1]?.trim();
    if (title) return title;
  }
  return null;
}

function humanizeRuleFileName(fileName: string): string {
  const withoutExtension = fileName.replace(/\.[^.]+$/, '');
  const normalized = withoutExtension.replace(/[-_.]+/g, ' ').trim();
  if (!normalized) return 'Untitled Rule';
  return normalized
    .split(/\s+/)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

function getRuleDisplayTitle(rule: { file_name: string; content: string }): string {
  return extractMarkdownHeadingTitle(rule.content) ?? humanizeRuleFileName(rule.file_name);
}

function mcpServerFromCatalog(entry: import('@/bindings').CatalogEntry): McpServerConfig {
  const inferredId = entry.id.startsWith('mcp-') ? entry.id.slice(4) : entry.id;
  const env: Record<string, string> = {};
  if (entry.id === 'mcp-github') env.GITHUB_TOKEN = '';
  if (entry.id === 'mcp-brave-search') env.BRAVE_API_KEY = '';
  if (entry.id === 'mcp-slack') env.SLACK_BOT_TOKEN = '';
  return {
    id: inferredId,
    name: entry.name,
    command: entry.command ?? '',
    args: entry.args ?? [],
    env,
    scope: 'project',
    server_type: 'stdio',
    url: null,
    disabled: false,
    timeout_secs: null,
  };
}

function mcpServerFromRegistry(entry: import('@/bindings').McpRegistryEntry): McpServerConfig {
  const env = Object.fromEntries((entry.required_env ?? []).map((key) => [key, '']));
  const transport = (entry.transport ?? 'stdio').toLowerCase();
  const serverType: import('@/bindings').McpServerType =
    transport === 'sse' ? 'sse'
      : transport === 'http' ? 'http'
      : 'stdio';

  return {
    id: slugifyId(entry.id || entry.server_name || entry.title || 'mcp-server'),
    name: entry.title || entry.server_name || entry.id || 'MCP Server',
    command: serverType === 'stdio' ? (entry.command ?? '') : '',
    args: serverType === 'stdio' ? (entry.args ?? []) : [],
    env,
    scope: 'project',
    server_type: serverType,
    url: serverType === 'stdio' ? null : (entry.url ?? null),
    disabled: false,
    timeout_secs: null,
  };
}

function normalizeProviderIds(providers: string[] | undefined): string[] {
  return (providers ?? []).map((p) => p.trim()).filter(Boolean);
}

function permissionsQueryKey(scope: ScopeKey) {
  return ['permissions', scope] as const;
}

// ── AgentsPanel ─────────────────────────────────────────────────────────────

export default function AgentsPanel({
  activeProject,
  projectConfig,
  globalAgentConfig,
  onSaveProject,
  onSaveGlobalAgentConfig,
  initialSection = 'providers',
  onBackToSettings,
}: AgentsPanelProps) {
  const queryClient = useQueryClient();
  const [localProject, setLocalProject] = useState<ProjectConfig>(normalizeProjectConfig(projectConfig));
  const [localGlobalAgent, setLocalGlobalAgent] = useState<ProjectConfig>(
    normalizeProjectConfig(globalAgentConfig)
  );
  const [agentScope, setAgentScope] = useState<ScopeKey>(projectConfig ? 'project' : 'global');
  const [expandedProviderId, setExpandedProviderId] = useState<string>('claude');
  const [exportStatus, setExportStatus] = useState<Record<string, 'idle' | 'loading' | 'ok' | 'error'>>({});
  const [importStatus, setImportStatus] = useState<Record<string, 'idle' | 'loading' | 'ok' | 'error'>>({});
  const [importSummary, setImportSummary] = useState<Record<string, string>>({});

  const [mcpEditDraft, setMcpEditDraft] = useState<McpEditDraft | null>(null);
  const [mcpCatalogInput, setMcpCatalogInput] = useState('');
  const [mcpExplorerOpen, setMcpExplorerOpen] = useState(false);
  const [mcpExplorerFilter, setMcpExplorerFilter] = useState<'recommended' | 'catalog' | 'registry' | 'all'>('recommended');
  const [mcpDiagnosticsOpen, setMcpDiagnosticsOpen] = useState(false);
  const [skillExplorerOpen, setSkillExplorerOpen] = useState(false);
  const [skillSourceInput, setSkillSourceInput] = useState('');
  const hasActiveProject = !!activeProject || !!projectConfig;
  const hasAutoScopedToProjectRef = useRef<boolean>(!!projectConfig);
  const permissionsSaveQueueRef = useRef<Promise<void>>(Promise.resolve());
  const discoveryRefreshedRef = useRef<Record<ScopeKey, boolean>>({
    global: false,
    project: false,
  });
  const mcpHydratedFromInventoryRef = useRef<Record<ScopeKey, boolean>>({
    global: false,
    project: false,
  });
  const skillScope = agentScope === 'project' ? 'project' : 'user';

  const [selectedDocIds, setSelectedDocIds] = useState<Record<ScopeKey, Record<MarkdownDocKind, string | null>>>(
    () => ({
      global: { skills: null, rules: null },
      project: { skills: null, rules: null },
    })
  );

  const activeDocKind: MarkdownDocKind | null =
    initialSection === 'skills' || initialSection === 'rules' ? initialSection : null;

  const providerAndAssetSectionActive =
    initialSection === 'providers'
    || initialSection === 'skills';
  const {
    providers,
    skills,
    providersPending,
    providersError,
    refreshProviders,
  } = useAgentAssetInventory({
    enabled: providerAndAssetSectionActive,
    includeProviders: true,
    includeMcpServers: false,
    includeSkills: true,
    skillScope,
  });
  const providerToolVocabularyQuery = useQuery({
    queryKey: ['permission-tool-vocabulary'],
    queryFn: async () => {
      const res = await commands.listPermissionToolVocabularyCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'permissions' || initialSection === 'hooks',
    staleTime: 60000,
  });
  const providerToolVocabulary = (providerToolVocabularyQuery.data ?? []) as import('@/bindings').ProviderToolVocabularyEntry[];

  // Catalog Query
  const catalogQuery = useQuery({
    queryKey: ['catalog'],
    queryFn: async () => commands.listCatalogCmd(),
    enabled:
      initialSection === 'mcp' ||
      initialSection === 'permissions',
  });
  const catalog = catalogQuery.data ?? EMPTY_CATALOG;

  // Rules Query
  const rulesQuery = useQuery({
    queryKey: ['rules'],
    queryFn: async () => {
      const res = await commands.listRulesCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'rules',
  });
  const rules = rulesQuery.data ?? EMPTY_RULES;
  const activeAgentConfig = useMemo(
    () => (agentScope === 'project' ? localProject : localGlobalAgent),
    [agentScope, localGlobalAgent, localProject]
  );

  const activeDocs: AgentDoc[] =
    activeDocKind === 'skills'
      ? skills.map((s) => ({
          id: s.id,
          title: s.name,
          content: s.content,
          updated: '',
          description: s.description ?? null,
          source: s.source ?? null,
          author: s.author ?? null,
          version: s.version ?? null,
        }))
      : rules.map((r) => ({
          id: r.file_name,
          title: getRuleDisplayTitle(r),
          content: r.content,
          updated: '',
        }));
  const mcpCatalogEntries = useMemo(
    () => catalog.filter((entry) => entry.kind === 'mcp-server'),
    [catalog]
  );

  const activeSelectedDocId = activeDocKind ? selectedDocIds[agentScope][activeDocKind] : null;
  const activeDoc = activeDocs.find((doc) => doc.id === activeSelectedDocId) ?? activeDocs[0] ?? null;
  const parsedSkillInstallSpec = useMemo(
    () => parseSkillSourceInstallSpec(skillSourceInput),
    [skillSourceInput]
  );
  const canInstallFromSource = parsedSkillInstallSpec.canInstall;

  const selectActiveDoc = (kind: MarkdownDocKind, docId: string) => {
    setSelectedDocIds((current) => ({
      ...current,
      [agentScope]: { ...current[agentScope], [kind]: docId },
    }));
  };

  // Mutations
  const createSkillMut = useMutation({
    mutationFn: async (vars: { id: string; name: string; content: string }) => {
      const res = await commands.createSkillCmd(vars.id, vars.name, vars.content, skillScope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: (newSkill) => {
      queryClient.invalidateQueries({ queryKey: ['skills', skillScope] });
      setSelectedDocIds((curr) => ({
        ...curr,
        [agentScope]: { ...curr[agentScope], skills: newSkill.id },
      }));
    },
  });

  const updateSkillMut = useMutation({
    mutationFn: async (vars: { id: string; name?: string; content?: string }) => {
      const res = await commands.updateSkillCmd(vars.id, vars.name ?? null, vars.content ?? null, skillScope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills', skillScope] });
    },
  });

  const deleteSkillMut = useMutation({
    mutationFn: async (id: string) => {
      const res = await commands.deleteSkillCmd(id, skillScope);
      if (res.status === 'error') throw new Error(res.error);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills', skillScope] });
      setSelectedDocIds((curr) => ({ ...curr, [agentScope]: { ...curr[agentScope], skills: null } }));
    },
  });

  const installSkillFromSourceMut = useMutation({
    mutationFn: async (vars: { source: string; skillId: string }) => {
      const res = await commands.installSkillFromSourceCmd(
        vars.source,
        vars.skillId,
        null,
        null,
        skillScope,
        false
      );
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: (installedSkill) => {
      queryClient.invalidateQueries({ queryKey: ['skills', skillScope] });
      setSelectedDocIds((curr) => ({
        ...curr,
        [agentScope]: { ...curr[agentScope], skills: installedSkill.id },
      }));
      setSkillSourceInput('');
      setSkillExplorerOpen(false);
    },
  });
  const mcpServersForValidation =
    (agentScope === 'project' ? localProject.mcp_servers : localGlobalAgent.mcp_servers) ?? [];

  const mcpValidationQuery = useQuery({
    queryKey: ['mcp-validation', agentScope, mcpServersForValidation],
    queryFn: async () => {
      const res = await commands.validateMcpServersCmd(mcpServersForValidation);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'providers' || initialSection === 'mcp',
  });
  const mcpValidationReport = (mcpValidationQuery.data ?? null) as McpValidationReport | null;
  const mcpProbeQuery = useQuery({
    queryKey: ['mcp-probe', agentScope, mcpServersForValidation],
    queryFn: async () => {
      const res = await commands.probeMcpServersCmd(mcpServersForValidation, agentScope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'mcp' && mcpServersForValidation.length > 0,
    staleTime: 60000,
  });
  const mcpProbeReport = (mcpProbeQuery.data ?? null) as McpProbeReport | null;
  const mcpDiagnosticsIssueCount =
    (mcpValidationReport?.issues.length ?? 0) +
    (mcpValidationQuery.isError ? 1 : 0) +
    (mcpProbeQuery.isError ? 1 : 0);
  const hasNoReachableServers =
    !!mcpProbeReport &&
    mcpProbeReport.checked_servers > 0 &&
    mcpProbeReport.reachable_servers === 0;
  const hasMcpSearchQuery = mcpCatalogInput.trim().length > 0;
  const discoveryCacheQuery = useQuery({
    queryKey: ['agent-discovery', agentScope],
    queryFn: async () => {
      const res = await commands.getAgentDiscoveryCacheCmd(agentScope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'mcp' || initialSection === 'permissions',
    staleTime: 60000,
  });
  const discoveryCache = (discoveryCacheQuery.data ?? null) as AgentDiscoveryCache | null;
  const refreshDiscoveryCacheMut = useMutation({
    mutationFn: async () => {
      const res = await commands.refreshAgentDiscoveryCacheCmd(agentScope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agent-discovery', agentScope] });
    },
  });
  const skillToolHintsQuery = useQuery({
    queryKey: ['skill-tool-hints', agentScope],
    queryFn: async () => {
      const scope = agentScope === 'project' ? 'project' : 'user';
      const res = await commands.listSkillToolHintsCmd(scope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'permissions' || initialSection === 'mcp',
    staleTime: 60000,
  });
  const skillToolHints = (skillToolHintsQuery.data ?? []) as import('@/bindings').SkillToolHint[];
  const mcpRegistryQuery = useMcpRegistrySearch({
    query: mcpCatalogInput,
    enabled: initialSection === 'mcp' && mcpExplorerOpen,
    limit: 20,
  });
  const mcpRegistryEntries = mcpRegistryQuery.data ?? [];
  const mcpServersInventoryQuery = useQuery({
    queryKey: ['mcp-servers-inventory', agentScope],
    queryFn: async () => {
      const res = await commands.listMcpServersCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled:
      initialSection === 'mcp'
      && hasActiveProject
      && agentScope === 'project'
      && ((activeAgentConfig.mcp_servers ?? []).length === 0),
    staleTime: 60_000,
  });

  const createRuleMut = useMutation({
    mutationFn: async (vars: { fileName: string; content: string }) => {
      const res = await commands.createRuleCmd(vars.fileName, vars.content);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: (newRule) => {
      queryClient.invalidateQueries({ queryKey: ['rules'] });
      setSelectedDocIds((curr) => ({
        ...curr,
        [agentScope]: { ...curr[agentScope], rules: newRule.file_name },
      }));
    },
  });

  const updateRuleMut = useMutation({
    mutationFn: async (vars: { fileName: string; content: string }) => {
      const res = await commands.updateRuleCmd(vars.fileName, vars.content);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['rules'] });
    },
  });

  const deleteRuleMut = useMutation({
    mutationFn: async (fileName: string) => {
      const res = await commands.deleteRuleCmd(fileName);
      if (res.status === 'error') throw new Error(res.error);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['rules'] });
      setSelectedDocIds((curr) => ({ ...curr, [agentScope]: { ...curr[agentScope], rules: null } }));
    },
  });

  useEffect(() => { setLocalProject(normalizeProjectConfig(projectConfig)); }, [projectConfig]);
  useEffect(() => { setLocalGlobalAgent(normalizeProjectConfig(globalAgentConfig)); }, [globalAgentConfig]);
  useEffect(() => {
    if (projectConfig) {
      if (!hasAutoScopedToProjectRef.current) {
        setAgentScope('project');
        hasAutoScopedToProjectRef.current = true;
      }
      return;
    }
    hasAutoScopedToProjectRef.current = false;
    setAgentScope('global');
  }, [projectConfig]);
  useEffect(() => {
    if (!(initialSection === 'mcp' || initialSection === 'permissions')) return;
    if (discoveryRefreshedRef.current[agentScope]) return;
    if (!discoveryCache) return;
    const hasShell = (discoveryCache.shell_commands ?? []).length > 0;
    const hasPaths = (discoveryCache.filesystem_paths ?? []).length > 0;
    if (hasShell && hasPaths) {
      discoveryRefreshedRef.current[agentScope] = true;
      return;
    }
    if (!refreshDiscoveryCacheMut.isPending) {
      discoveryRefreshedRef.current[agentScope] = true;
      refreshDiscoveryCacheMut.mutate();
    }
  }, [initialSection, agentScope, discoveryCache, refreshDiscoveryCacheMut]);
  useEffect(() => {
    if (!mcpProbeReport) return;
    queryClient.invalidateQueries({ queryKey: ['agent-discovery', agentScope] });
  }, [mcpProbeReport, queryClient, agentScope]);
  useEffect(() => {
    if (initialSection !== 'mcp') return;
    if (agentScope !== 'project' || !hasActiveProject) return;
    if (mcpHydratedFromInventoryRef.current[agentScope]) return;
    if ((activeAgentConfig.mcp_servers ?? []).length > 0) {
      mcpHydratedFromInventoryRef.current[agentScope] = true;
      return;
    }
    const fallbackServers = mcpServersInventoryQuery.data ?? [];
    if (fallbackServers.length === 0) return;
    mcpHydratedFromInventoryRef.current[agentScope] = true;
    updateActiveAgentConfig({
      ...activeAgentConfig,
      mcp_servers: fallbackServers,
    });
  }, [initialSection, agentScope, hasActiveProject, activeAgentConfig, mcpServersInventoryQuery.data]);

  const providerRows = useMemo<ProviderRow[]>(() => {
    const enabled = new Set(activeAgentConfig.providers ?? []);
    const detectedById = new Map(providers.map((provider) => [provider.id, provider]));
    const checking = providersPending;
    const supportedRows = SUPPORTED_PROVIDER_BASE.map((provider) => {
      const detected = detectedById.get(provider.id);
      if (detected) {
        return {
          ...detected,
          enabled: enabled.has(detected.id),
          checking,
        };
      }
      return {
        id: provider.id,
        name: provider.name,
        binary: provider.binary,
        project_config: '',
        global_config: '',
        config_format: 'json',
        prompt_output: 'none',
        skills_output: 'none',
        enabled: enabled.has(provider.id),
        installed: false,
        version: null,
        models: [],
        checking,
      };
    });
    const extraRows = providers
      .filter((provider) => !SUPPORTED_PROVIDER_IDS.has(provider.id))
      .map((provider) => ({
        ...provider,
        enabled: enabled.has(provider.id),
        checking,
      }));
    return [...supportedRows, ...extraRows];
  }, [providers, activeAgentConfig.providers, providersPending]);

  const recommendedMcpCatalogEntries = useMemo(() => {
    const official = mcpCatalogEntries
      .filter((entry) => (entry.tags ?? []).some((tag) => tag.toLowerCase() === 'official'))
      .sort((left, right) => left.name.localeCompare(right.name));
    if (official.length >= 8) return official.slice(0, 8);
    const seen = new Set(official.map((entry) => entry.id));
    const fallback = mcpCatalogEntries
      .filter((entry) => !seen.has(entry.id))
      .sort((left, right) => left.name.localeCompare(right.name));
    return [...official, ...fallback].slice(0, 8);
  }, [mcpCatalogEntries]);

  const filteredMcpCatalogEntries = useMemo(() => {
    const query = mcpCatalogInput.trim().toLowerCase();
    if (!query) {
      return [...mcpCatalogEntries].sort((left, right) => left.name.localeCompare(right.name));
    }
    return mcpCatalogEntries
      .filter((entry) => {
        const haystack = [entry.id, entry.name, entry.description, entry.author ?? '', ...(entry.tags ?? [])].join(' ').toLowerCase();
        return haystack.includes(query);
      })
      .sort((left, right) => left.name.localeCompare(right.name));
  }, [mcpCatalogEntries, mcpCatalogInput]);

  const skillFolderRows = useMemo(
    () => activeDocs.map((doc) => ({ id: doc.id, fileName: 'SKILL.md', title: doc.title || doc.id })),
    [activeDocs]
  );

  const mcpIdOptions = useMemo(() => {
    const fromCatalog = mcpCatalogEntries.map((entry) => ({
      value: entry.id.startsWith('mcp-') ? entry.id.slice(4) : entry.id,
      label: entry.name,
      keywords: entry.tags,
    }));
    const fromRegistry = mcpRegistryEntries.map((entry) => ({
      value: entry.id,
      label: entry.title,
      keywords: [entry.server_name, entry.transport, entry.version],
    }));
    const fromExisting = (activeAgentConfig.mcp_servers ?? [])
      .map((server) => (server.id ?? '').trim())
      .filter(Boolean)
      .map((value) => ({ value }));
    return [...fromCatalog, ...fromRegistry, ...fromExisting];
  }, [mcpCatalogEntries, mcpRegistryEntries, activeAgentConfig.mcp_servers]);

  const mcpCommandOptions = useMemo(() => {
    const seeded = ['npx', 'uvx', 'docker', 'ship', 'node'];
    const fromCatalog = mcpCatalogEntries.flatMap((entry) => [entry.command ?? '', entry.install_command ?? '']);
    const fromRegistry = mcpRegistryEntries.flatMap((entry) => [entry.command ?? '']);
    const fromExisting = (activeAgentConfig.mcp_servers ?? []).map((server) => server.command ?? '');
    const values = [...seeded, ...fromCatalog, ...fromRegistry, ...fromExisting].map((value) => value.trim()).filter(Boolean);
    return Array.from(new Set(values)).map((value) => ({ value }));
  }, [mcpCatalogEntries, mcpRegistryEntries, activeAgentConfig.mcp_servers]);

  const mcpEnvKeyOptions = useMemo(() => {
    const seeded = ['GITHUB_TOKEN', 'BRAVE_API_KEY', 'SLACK_BOT_TOKEN', 'OPENAI_API_KEY', 'ANTHROPIC_API_KEY'];
    const fromRegistry = mcpRegistryEntries.flatMap((entry) => entry.required_env ?? []);
    const fromExisting = (activeAgentConfig.mcp_servers ?? []).flatMap((server) => Object.keys(server.env ?? {}));
    const values = [...seeded, ...fromRegistry, ...fromExisting].filter(Boolean);
    return Array.from(new Set(values)).map((value) => ({ value }));
  }, [mcpRegistryEntries, activeAgentConfig.mcp_servers]);

  const installedMcpServerIdSet = useMemo(() => {
    const ids = new Set<string>();
    for (const server of activeAgentConfig.mcp_servers ?? []) {
      const candidate = ((server.id ?? server.name) || '').trim().toLowerCase();
      if (candidate) ids.add(candidate);
    }
    return ids;
  }, [activeAgentConfig.mcp_servers]);

  const mcpProbeByServerId = useMemo(
    () => new Map((mcpProbeReport?.results ?? []).map((result) => [result.server_id, result])),
    [mcpProbeReport]
  );

  const cachedMcpToolsByServerId = useMemo(() => {
    const map = new Map<string, Array<{ name: string; description?: string | null }>>();
    Object.entries(discoveryCache?.mcp_tools ?? {}).forEach(([serverId, tools]) => {
      map.set(serverId, (tools || []).map(t => ({ name: t.name, description: t.description })));
    });
    return map;
  }, [discoveryCache]);

  const discoveredMcpToolPatterns = useMemo(() => {
    const fromProbe = (mcpProbeReport?.results ?? []).flatMap((result) =>
      (result.discovered_tools ?? []).map((tool) => mcpToolPattern(result.server_id, tool.name))
    );
    const fromCache = Array.from(cachedMcpToolsByServerId.entries()).flatMap(([serverId, tools]) =>
      tools.map((tool) => mcpToolPattern(serverId, tool.name))
    );
    return Array.from(new Set([...fromProbe, ...fromCache]));
  }, [mcpProbeReport, cachedMcpToolsByServerId]);

  const skillToolAllowedPatterns = useMemo(() => {
    const direct = skillToolHints.flatMap((hint) => hint.allowed_tools ?? []);
    return Array.from(new Set(direct.map((value) => value.trim()).filter(Boolean)));
  }, [skillToolHints]);

  const providerNativeToolIds = useMemo(() => {
    const configured = new Set(normalizeProviderIds(activeAgentConfig.providers));
    const enabledRows = providerToolVocabulary.filter((entry) => entry.enabled);
    const configuredRows = providerToolVocabulary.filter((entry) => configured.has(entry.provider_id));
    const installedRows = providerToolVocabulary.filter((entry) => entry.installed);
    const sourceRows =
      configuredRows.length > 0 ? configuredRows
        : enabledRows.length > 0 ? enabledRows
        : installedRows.length > 0 ? installedRows
        : providerToolVocabulary;
    return Array.from(new Set(sourceRows.flatMap((entry) => entry.tool_ids ?? [])))
      .map((value) => value.trim())
      .filter((value) => value.length > 0 && value !== '*' && !value.startsWith('mcp__'));
  }, [activeAgentConfig.providers, providerToolVocabulary]);

  const permissionToolSuggestions = useMemo(() => {
    const providerNativeTools = providerNativeToolIds;
    const serverPatterns = (activeAgentConfig.mcp_servers ?? [])
      .map((server) => (server.id ?? '').trim())
      .filter(Boolean)
      .flatMap((id) => [`mcp__${id}__*`, `mcp__${id}__read*`, `mcp__${id}__write*`]);
    const catalogPatterns = mcpCatalogEntries
      .map((entry) => entry.id.startsWith('mcp-') ? entry.id.slice(4) : entry.id)
      .flatMap((id) => [`mcp__${id}__*`, `mcp__${id}__read*`, `mcp__${id}__write*`]);
    const baseline = ['*', 'mcp__*__*', 'mcp__*__read*', 'mcp__*__write*', 'mcp__*__delete*'];
    const builtInSet = new Set(providerNativeTools);
    const values = Array.from(new Set([...baseline, ...providerNativeTools, ...serverPatterns, ...catalogPatterns, ...discoveredMcpToolPatterns, ...skillToolAllowedPatterns]));
    const rank = (value: string) => {
      if (value === '*') return 0;
      if (builtInSet.has(value)) return 1;
      if (value.startsWith('mcp__')) return 2;
      return 3;
    };
    return values
      .sort((left, right) => {
        const bucket = rank(left) - rank(right);
        if (bucket !== 0) return bucket;
        return left.localeCompare(right);
      })
      .map((value) => {
        const type = value === '*' ? 'wildcard' : builtInSet.has(value) ? 'builtin' : value.startsWith('mcp__') ? 'mcp' : 'pattern';
        return {
          value,
          label: type === 'wildcard' ? 'Wildcard (all tools)' : type === 'builtin' ? 'Built-in provider tool' : type === 'mcp' ? 'MCP tool pattern' : 'Tool pattern',
          keywords: [type, 'tool'],
        };
      });
  }, [activeAgentConfig.mcp_servers, mcpCatalogEntries, discoveredMcpToolPatterns, skillToolAllowedPatterns, providerNativeToolIds]);

  const hookCommandSuggestions = useMemo(() => {
    const seeded = ['$SHIP_HOOKS_BIN', 'ship hooks run', 'node', 'bash'];
    const shellValues = (discoveryCache?.shell_commands ?? []).slice(0, 120);
    const values = [...seeded, ...mcpCommandOptions.map((option) => option.value), ...shellValues];
    return Array.from(new Set(values.filter(Boolean))).map((value) => ({ value }));
  }, [mcpCommandOptions, discoveryCache]);

  const hookMatcherSuggestions = useMemo(() => {
    const seeded = ['mcp__*', 'mcp__*__read*', 'mcp__*__write*'];
    const values = [...seeded, ...providerNativeToolIds, ...permissionToolSuggestions.map((option) => option.value)];
    return Array.from(new Set(values.filter(Boolean))).map((value) => ({ value }));
  }, [permissionToolSuggestions, providerNativeToolIds]);

  const filesystemPathSuggestions = useMemo(() => {
    const seeded = ['.ship/**', 'src/**', 'docs/**', 'tests/**', '~/.ssh/**', '~/.gnupg/**', '/etc/**', '/proc/**', '/sys/**'];
    const discovered = discoveryCache?.filesystem_paths ?? [];
    return Array.from(new Set([...seeded, ...discovered])).map((value) => ({ value }));
  }, [discoveryCache]);

  const commandPatternSuggestions = useMemo(() => {
    const seeded = ['ship', 'ship *', 'gh', 'gh *', 'git', 'git *', 'cargo', 'cargo *', 'npm', 'npm *', 'pnpm', 'pnpm *', 'python', 'python *', 'bash', 'bash *'];
    const discovered = (discoveryCache?.shell_commands ?? []).flatMap((command) => [command, `${command} *`]);
    return Array.from(new Set([...seeded, ...discovered])).map((value) => value.trim()).filter(Boolean).map((value) => ({ value }));
  }, [discoveryCache]);

  const connectedProviders = activeAgentConfig.providers ?? [];
  const providersForHookInference = connectedProviders.length
    ? connectedProviders
    : (providers.length > 0 ? providers.map((provider) => provider.id) : SUPPORTED_PROVIDER_BASE.map((provider) => provider.id));
  const activeHookEvents = useMemo(() => {
    const supportedProviders = new Set(providersForHookInference);
    return HOOK_EVENTS.filter((event) => event.providers.some((provider) => supportedProviders.has(provider)));
  }, [providersForHookInference]);
  const defaultHookTrigger = activeHookEvents[0]?.value ?? 'PreToolUse';
  const providersWithNativeHooks = providersForHookInference.filter((id) => id === 'claude' || id === 'gemini');
  const providersWithoutNativeHooks = providersForHookInference.filter((id) => id !== 'claude' && id !== 'gemini');

  const toHookId = (trigger: string, existingHooks: HookConfig[]) => {
    const base = trigger.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase();
    let candidate = base;
    let counter = 2;
    while (existingHooks.some((hook) => hook.id === candidate)) {
      candidate = `${base}-${counter}`;
      counter += 1;
    }
    return candidate;
  };

  const updateHooks = (hooks: HookConfig[]) => {
    updateActiveAgentConfig({ ...activeAgentConfig, hooks });
  };

  const handleAddHook = () => {
    const hooks = [...(activeAgentConfig.hooks ?? [])];
    hooks.push({
      id: toHookId(defaultHookTrigger, hooks),
      trigger: defaultHookTrigger as HookConfig['trigger'],
      matcher: null,
      timeout_ms: null,
      description: null,
      command: '$SHIP_HOOKS_BIN',
    });
    updateHooks(hooks);
  };

  const handleUpdateHook = (idx: number, patch: Partial<HookConfig>) => {
    const hooks = [...(activeAgentConfig.hooks ?? [])];
    const current = hooks[idx];
    if (!current) return;
    const next = { ...current, ...patch };
    if (!next.id.trim()) {
      next.id = toHookId(String(next.trigger), hooks.filter((_, i) => i !== idx));
    }
    hooks[idx] = next;
    updateHooks(hooks);
  };

  const handleRemoveHook = (idx: number) => {
    const hooks = (activeAgentConfig.hooks ?? []).filter((_, index) => index !== idx);
    updateHooks(hooks);
  };

  const [permissionsDraftByScope, setPermissionsDraftByScope] = useState<Record<ScopeKey, Permissions | null>>({
    global: null,
    project: null,
  });
  const [permissionsTab, setPermissionsTab] = useState<'tools' | 'commands' | 'filesystem'>('tools');
  const [permissionsDirtyByScope, setPermissionsDirtyByScope] = useState<Record<ScopeKey, boolean>>({
    global: false,
    project: false,
  });
  const permissionsDraft = permissionsDraftByScope[agentScope];
  const permissionsDirty = permissionsDirtyByScope[agentScope];

  // Permissions Query
  const { data: permissions } = useQuery({
    queryKey: permissionsQueryKey(agentScope),
    queryFn: async () => {
      const res = await commands.getPermissionsCmd();
      if (res.status === 'error') throw new Error(res.error);
      return normalizePermissionsForEditor(res.data);
    },
    enabled: initialSection === 'permissions' || initialSection === 'mcp',
  });

  const savePermissionsMut = useMutation({
    mutationFn: async (input: { permissions: Permissions; scope: ScopeKey }) => {
      const saveAttempt = async () => {
        const res = await commands.savePermissionsCmd(input.permissions);
        if (res.status === 'error') throw new Error(res.error);
      };
      const queued = permissionsSaveQueueRef.current.then(saveAttempt, saveAttempt);
      permissionsSaveQueueRef.current = queued.then(() => undefined, () => undefined);
      await queued;
      return {
        scope: input.scope,
        permissions: normalizePermissionsForEditor(input.permissions),
      };
    },
    onMutate: (input) => {
      queryClient.setQueryData(permissionsQueryKey(input.scope), normalizePermissionsForEditor(input.permissions));
    },
    onSuccess: ({ scope, permissions: savedPermissions }) => {
      const normalized = normalizePermissionsForEditor(savedPermissions);
      queryClient.setQueryData(permissionsQueryKey(scope), normalized);
      setPermissionsDraftByScope((current) => ({ ...current, [scope]: normalized }));
      setPermissionsDirtyByScope((current) => ({ ...current, [scope]: false }));
    },
    onError: (_error, input) => {
      queryClient.invalidateQueries({ queryKey: permissionsQueryKey(input.scope) });
    },
  });

  useEffect(() => {
    if (!permissions) return;
    setPermissionsDraftByScope((current) => {
      if (current[agentScope]) return current;
      return { ...current, [agentScope]: normalizePermissionsForEditor(permissions) };
    });
  }, [agentScope, permissions]);

  const activePermissions = permissionsDraft
    ? normalizePermissionsForEditor(permissionsDraft)
    : permissions
      ? normalizePermissionsForEditor(permissions)
      : null;

  const savePermissionsDraft = () => {
    if (!activePermissions || savePermissionsMut.isPending) return;
    savePermissionsMut.mutate({
      scope: agentScope,
      permissions: normalizePermissionsForEditor(activePermissions),
    });
  };

  const updatePermissions = (updater: (current: Permissions) => Permissions) => {
    setPermissionsDraftByScope((drafts) => {
      const fallback = normalizePermissionsForEditor(
        (queryClient.getQueryData(permissionsQueryKey(agentScope)) as Permissions | undefined) ?? permissions
      );
      const base = normalizePermissionsForEditor(drafts[agentScope] ?? fallback);
      const next = normalizePermissionsForEditor(updater(base));
      return { ...drafts, [agentScope]: next };
    });
    setPermissionsDirtyByScope((value) => ({ ...value, [agentScope]: true }));
  };

  const toolAllowPatterns = activePermissions?.tools?.allow ?? [];
  const toolDenyPatterns = activePermissions?.tools?.deny ?? [];

  const permissionValidationIssues = useMemo(() => {
    if (!activePermissions) return [] as string[];
    const issues: string[] = [];
    const checkToolPattern = (value: string, listName: 'allow' | 'deny') => {
      const trimmed = value.trim();
      if (!trimmed) { issues.push(`Tools ${listName}: empty pattern.`); return; }
      if (trimmed.includes(' ')) issues.push(`Tools ${listName}: "${trimmed}" contains spaces; use glob patterns/tool IDs.`);
      if (trimmed.startsWith('mcp__') && !trimmed.slice('mcp__'.length).includes('__')) {
        issues.push(`Tools ${listName}: "${trimmed}" is missing "__tool" segment.`);
      }
    };
    for (const value of activePermissions.tools?.allow ?? []) checkToolPattern(value, 'allow');
    for (const value of activePermissions.tools?.deny ?? []) checkToolPattern(value, 'deny');
    for (const value of activePermissions.commands?.allow ?? []) { if (!value.trim()) issues.push('Commands allow: empty pattern.'); }
    for (const value of activePermissions.commands?.deny ?? []) { if (!value.trim()) issues.push('Commands deny: empty pattern.'); }
    for (const value of activePermissions.filesystem?.allow ?? []) { if (!value.trim()) issues.push('Filesystem allow: empty path pattern.'); }
    for (const value of activePermissions.filesystem?.deny ?? []) { if (!value.trim()) issues.push('Filesystem deny: empty path pattern.'); }
    return issues;
  }, [activePermissions]);

  const updateActiveAgentConfig = (next: ProjectConfig) => {
    if (agentScope === 'project') { setLocalProject(next); return; }
    setLocalGlobalAgent(next);
  };

  const handleUpsertDoc = (kind: MarkdownDocKind, docId: string, patch: Partial<AgentDoc>) => {
    if (kind === 'skills') {
      updateSkillMut.mutate({ id: docId, name: patch.title, content: patch.content });
    } else {
      updateRuleMut.mutate({ fileName: docId, content: patch.content ?? '' });
    }
  };

  const handleCreateDoc = (kind: MarkdownDocKind) => {
    const title = kind === 'skills' ? 'Untitled Skill' : 'Untitled Rule';
    if (kind === 'skills') {
      createSkillMut.mutate({ id: `skill-${Date.now()}`, name: title, content: `# ${title}\n` });
    } else {
      createRuleMut.mutate({ fileName: `rule-${Date.now()}.md`, content: `# ${title}\n` });
    }
  };

  const handleDeleteDoc = (kind: MarkdownDocKind, docId: string) => {
    if (kind === 'skills') deleteSkillMut.mutate(docId);
    else deleteRuleMut.mutate(docId);
  };

  const appendMcpServerDefinition = (definition: McpServerConfig) => {
    const servers = [...(activeAgentConfig.mcp_servers ?? [])];
    const existingIds = new Set(servers.map((server) => (server.id ?? '').trim()).filter(Boolean));
    const baseId = inferMcpServerId(definition);
    let id = baseId;
    let index = 2;
    while (existingIds.has(id)) {
      id = `${baseId}-${index}`;
      index += 1;
    }
    const nextServer: McpServerConfig = { ...definition, id, name: definition.name.trim() || id };
    servers.push(nextServer);
    updateActiveAgentConfig({ ...activeAgentConfig, mcp_servers: servers });
    setMcpExplorerOpen(false);
    setMcpEditDraft({ idx: servers.length - 1, server: nextServer });
  };

  const handleInstallCatalogMcpEntry = (entry: import('@/bindings').CatalogEntry) => {
    appendMcpServerDefinition(mcpServerFromCatalog(entry));
  };

  const handleInstallRegistryEntry = (entry: import('@/bindings').McpRegistryEntry) => {
    const normalizedServer = mcpServerFromRegistry(entry);
    if (MCP_STDIO_ONLY_ALPHA && normalizedServer.server_type !== 'stdio') return;
    appendMcpServerDefinition(normalizedServer);
  };

  const handleInstallSkillFromSource = () => {
    const source = parsedSkillInstallSpec.source.trim();
    const skillId = parsedSkillInstallSpec.skillId;
    if (!source || !parsedSkillInstallSpec.canInstall) return;
    installSkillFromSourceMut.mutate({ source, skillId });
  };

  const handleExport = async (target: string) => {
    setExportStatus((prev) => ({ ...prev, [target]: 'loading' }));
    try {
      const res = await commands.exportAgentConfigCmd(target);
      if (res.status === 'error') throw new Error(res.error);
      setExportStatus((prev) => ({ ...prev, [target]: 'ok' }));
    } catch {
      setExportStatus((prev) => ({ ...prev, [target]: 'error' }));
    }
  };

  const handleImport = async (target: string) => {
    setImportStatus((prev) => ({ ...prev, [target]: 'loading' }));
    try {
      const res = await commands.importAgentConfigCmd(target, true);
      if (res.status === 'error') throw new Error(res.error);
      const importedMcp = res.data.imported_mcp_servers;
      const importedSkills = res.data.imported_skills;
      const importedPermissions = res.data.imported_permissions;
      const summaryParts = [
        importedMcp > 0 ? `${importedMcp} MCP server${importedMcp === 1 ? '' : 's'} imported` : 'No new MCP servers',
        importedSkills > 0 ? `${importedSkills} skill${importedSkills === 1 ? '' : 's'} imported` : 'No new skills',
      ];
      if (importedPermissions) summaryParts.push('permissions imported');
      setImportSummary((prev) => ({ ...prev, [target]: summaryParts.join(' • ') }));
      setImportStatus((prev) => ({ ...prev, [target]: 'ok' }));
      if (hasActiveProject) {
        const refreshed = await commands.getProjectConfig();
        if (refreshed.status === 'ok') {
          setLocalProject(normalizeProjectConfig(refreshed.data));
        }
      }
      refreshProviders();
      void mcpValidationQuery.refetch();
    } catch {
      setImportStatus((prev) => ({ ...prev, [target]: 'error' }));
    }
  };

  // MCP CRUD
  const handleRemoveMcpServer = (idx: number) => {
    const servers = [...(activeAgentConfig.mcp_servers ?? [])];
    servers.splice(idx, 1);
    updateActiveAgentConfig({ ...activeAgentConfig, mcp_servers: servers });
  };

  const handleSaveMcpServer = () => {
    if (!mcpEditDraft) return;
    const normalizedServer: McpServerConfig = {
      ...mcpEditDraft.server,
      id: inferMcpServerId(mcpEditDraft.server),
      name: mcpEditDraft.server.name.trim() || inferMcpServerId(mcpEditDraft.server),
      command: (mcpEditDraft.server.command ?? '').trim(),
      args: mcpEditDraft.server.server_type === 'stdio' ? (mcpEditDraft.server.args ?? []) : [],
      scope: mcpEditDraft.server.scope ?? 'project',
    };
    const servers = [...(activeAgentConfig.mcp_servers ?? [])];
    if (mcpEditDraft.idx === null) {
      servers.push(normalizedServer);
    } else {
      servers[mcpEditDraft.idx] = normalizedServer;
    }
    updateActiveAgentConfig({ ...activeAgentConfig, mcp_servers: servers });
    setMcpEditDraft(null);
  };

  const handleToggleDiscoveredToolPolicy = (serverId: string, toolName: string) => {
    const pattern = mcpToolPattern(serverId, toolName);
    updatePermissions((current) => {
      const deny = new Set(current.tools?.deny ?? []);
      if (deny.has(pattern)) { deny.delete(pattern); } else { deny.add(pattern); }
      return { ...current, tools: { ...current.tools, allow: current.tools?.allow ?? ['*'], deny: Array.from(deny) } };
    });
  };

  const handleToggleServerToolBlock = (serverId: string) => {
    const wildcard = `mcp__${serverId}__*`;
    updatePermissions((current) => {
      const deny = new Set(current.tools?.deny ?? []);
      if (deny.has(wildcard)) { deny.delete(wildcard); } else { deny.add(wildcard); }
      return { ...current, tools: { ...current.tools, allow: current.tools?.allow ?? ['*'], deny: Array.from(deny) } };
    });
  };

  const handleSave = () => {
    if (agentScope === 'global') return onSaveGlobalAgentConfig(localGlobalAgent);
    return onSaveProject(localProject);
  };

  const showFooterSaveCta = initialSection !== 'skills'
    && initialSection !== 'rules'
    && initialSection !== 'permissions';

  const scopeToggle = (
    <div className="flex items-center gap-1 rounded-lg border bg-muted/50 p-1">
      <Tooltip>
        <TooltipTrigger asChild>
          <button
            type="button"
            onClick={() => setAgentScope('global')}
            className={cn(
              'flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-medium transition-all',
              agentScope === 'global'
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'
            )}
          >
            <Globe className="size-3" />
            Global
          </button>
        </TooltipTrigger>
        <TooltipContent>Edit defaults shared across all projects on this machine.</TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger asChild>
          <button
            type="button"
            onClick={() => hasActiveProject && setAgentScope('project')}
            aria-disabled={!hasActiveProject}
            className={cn(
              'flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-medium transition-all',
              agentScope === 'project'
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground',
              !hasActiveProject && 'cursor-not-allowed opacity-40'
            )}
          >
            <Folder className="size-3" />
            Project
          </button>
        </TooltipTrigger>
        <TooltipContent>
          {hasActiveProject
            ? 'Edit overrides for the current project only.'
            : 'No active project selected. Open or create a project to use project scope.'}
        </TooltipContent>
      </Tooltip>
    </div>
  );

  const sectionMeta = SECTION_META[initialSection] ?? { title: initialSection, description: '' };
  const sectionTitle = onBackToSettings && initialSection === 'skills'
    ? 'Back to Settings'
    : sectionMeta.title;
  const frameClass = initialSection === 'skills' && onBackToSettings ? 'p-2 md:p-4' : 'md:p-8';

  const headerActions = (
    <div className="flex items-center gap-2">
      {onBackToSettings && (
        <Button variant="ghost" size="xs" onClick={onBackToSettings}>
          <ArrowLeft className="mr-1 size-3" /> Back
        </Button>
      )}
      {scopeToggle}
    </div>
  );

  return (
    <PageFrame className={frameClass}>
      <PageHeader
        title={sectionTitle}
        description={sectionMeta.description}
        actions={headerActions}
      />

      <div className="grid gap-4">
        {!hasActiveProject && (
          <Alert className="border-amber-500/30 bg-amber-500/5">
            <AlertDescription className="text-xs text-amber-800 dark:text-amber-200">
              No project is currently selected, so you are editing global defaults. Open or create a project to configure workspace-specific overrides.
            </AlertDescription>
          </Alert>
        )}

        {initialSection === 'providers' && (
          <ProvidersSection
            providerRows={providerRows}
            providersPending={providersPending}
            providersError={providersError ?? null}
            refreshProviders={refreshProviders}
            expandedProviderId={expandedProviderId}
            setExpandedProviderId={setExpandedProviderId}
            hasActiveProject={hasActiveProject}
            agentScope={agentScope}
            importStatus={importStatus}
            exportStatus={exportStatus}
            importSummary={importSummary}
            onImport={(target) => void handleImport(target)}
            onExport={(target) => void handleExport(target)}
            mcpValidationReport={mcpValidationReport}
            mcpValidationIsFetching={mcpValidationQuery.isFetching}
            onRefetchMcpValidation={() => void mcpValidationQuery.refetch()}
          />
        )}

        {initialSection === 'mcp' && (
          <McpServersSection
            activeAgentConfig={activeAgentConfig}
            agentScope={agentScope}
            mcpEditDraft={mcpEditDraft}
            setMcpEditDraft={setMcpEditDraft}
            mcpCatalogInput={mcpCatalogInput}
            setMcpCatalogInput={setMcpCatalogInput}
            mcpExplorerOpen={mcpExplorerOpen}
            setMcpExplorerOpen={setMcpExplorerOpen}
            mcpExplorerFilter={mcpExplorerFilter}
            setMcpExplorerFilter={setMcpExplorerFilter}
            mcpDiagnosticsOpen={mcpDiagnosticsOpen}
            setMcpDiagnosticsOpen={setMcpDiagnosticsOpen}
            recommendedMcpCatalogEntries={recommendedMcpCatalogEntries}
            filteredMcpCatalogEntries={filteredMcpCatalogEntries}
            mcpRegistryEntries={mcpRegistryEntries}
            mcpRegistryIsFetching={mcpRegistryQuery.isFetching}
            mcpRegistryIsError={mcpRegistryQuery.isError}
            installedMcpServerIdSet={installedMcpServerIdSet}
            hasMcpSearchQuery={hasMcpSearchQuery}
            mcpValidationReport={mcpValidationReport}
            mcpValidationIsFetching={mcpValidationQuery.isFetching}
            mcpValidationIsError={mcpValidationQuery.isError}
            mcpValidationError={mcpValidationQuery.error}
            mcpProbeReport={mcpProbeReport}
            mcpProbeIsFetching={mcpProbeQuery.isFetching}
            mcpProbeIsError={mcpProbeQuery.isError}
            mcpProbeError={mcpProbeQuery.error}
            mcpDiagnosticsIssueCount={mcpDiagnosticsIssueCount}
            hasNoReachableServers={hasNoReachableServers}
            mcpProbeByServerId={mcpProbeByServerId}
            cachedMcpToolsByServerId={cachedMcpToolsByServerId}
            activePermissions={activePermissions}
            savePermissionsIsPending={savePermissionsMut.isPending}
            mcpIdOptions={mcpIdOptions}
            mcpCommandOptions={mcpCommandOptions}
            mcpEnvKeyOptions={mcpEnvKeyOptions}
            onInstallCatalogMcpEntry={handleInstallCatalogMcpEntry}
            onInstallRegistryEntry={handleInstallRegistryEntry}
            onRemoveMcpServer={handleRemoveMcpServer}
            onSaveMcpServer={handleSaveMcpServer}
            onToggleDiscoveredToolPolicy={handleToggleDiscoveredToolPolicy}
            onToggleServerToolBlock={handleToggleServerToolBlock}
            onRefetchMcpValidation={() => void mcpValidationQuery.refetch()}
            onRefetchMcpProbe={() => void mcpProbeQuery.refetch()}
          />
        )}

        {(initialSection === 'skills' || initialSection === 'rules') && activeDocKind && (
          <SkillsSection
            initialSection={initialSection as 'skills' | 'rules'}
            activeDocKind={activeDocKind}
            agentScope={agentScope}
            activeDocs={activeDocs}
            activeDoc={activeDoc}
            skillFolderRows={skillFolderRows}
            skillExplorerOpen={skillExplorerOpen}
            setSkillExplorerOpen={setSkillExplorerOpen}
            skillSourceInput={skillSourceInput}
            setSkillSourceInput={setSkillSourceInput}
            parsedSkillInstallSpec={parsedSkillInstallSpec}
            canInstallFromSource={canInstallFromSource}
            installSkillFromSourceIsPending={installSkillFromSourceMut.isPending}
            installSkillFromSourceError={installSkillFromSourceMut.error}
            projectConfig={!!projectConfig}
            onSelectActiveDoc={selectActiveDoc}
            onCreateDoc={handleCreateDoc}
            onDeleteDoc={handleDeleteDoc}
            onUpsertDoc={handleUpsertDoc}
            onInstallSkillFromSource={handleInstallSkillFromSource}
            onSave={handleSave}
          />
        )}

        {initialSection === 'hooks' && (
          <HooksSection
            hooks={activeAgentConfig.hooks ?? []}
            agentScope={agentScope}
            activeHookEvents={activeHookEvents}
            defaultHookTrigger={defaultHookTrigger}
            hookCommandSuggestions={hookCommandSuggestions}
            hookMatcherSuggestions={hookMatcherSuggestions}
            providersWithNativeHooks={providersWithNativeHooks}
            providersWithoutNativeHooks={providersWithoutNativeHooks}
            onAddHook={handleAddHook}
            onUpdateHook={handleUpdateHook}
            onRemoveHook={handleRemoveHook}
          />
        )}

        {initialSection === 'permissions' && (
          <PermissionsSection
            agentScope={agentScope}
            activePermissions={activePermissions}
            permissionsDirty={permissionsDirty}
            permissionsTab={permissionsTab}
            setPermissionsTab={setPermissionsTab}
            permissionValidationIssues={permissionValidationIssues}
            toolAllowPatterns={toolAllowPatterns}
            toolDenyPatterns={toolDenyPatterns}
            permissionToolSuggestions={permissionToolSuggestions}
            commandPatternSuggestions={commandPatternSuggestions}
            filesystemPathSuggestions={filesystemPathSuggestions}
            discoveryCache={discoveryCache}
            savePermissionsIsPending={savePermissionsMut.isPending}
            refreshDiscoveryCacheIsPending={refreshDiscoveryCacheMut.isPending}
            onApplyPreset={(permissions) => updatePermissions(() => permissions)}
            onUpdatePermissions={updatePermissions}
            onSavePermissions={savePermissionsDraft}
            onRefreshDiscoveryCache={() => refreshDiscoveryCacheMut.mutate()}
          />
        )}
      </div>

      {showFooterSaveCta && (
        <footer className="flex items-center justify-end gap-2 border-t pt-4">
          {agentScope === 'global' ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button onClick={() => void handleSave()}>Save Global Agent Config</Button>
              </TooltipTrigger>
              <TooltipContent>
                Persist global defaults for all Ship projects on this machine.
              </TooltipContent>
            </Tooltip>
          ) : (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button onClick={() => void handleSave()} disabled={!projectConfig}>
                  Save Project Agent Config
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                Persist project-scoped agent overrides for this workspace.
              </TooltipContent>
            </Tooltip>
          )}
        </footer>
      )}
    </PageFrame>
  );
}
