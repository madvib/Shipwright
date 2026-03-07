import React, { useEffect, useMemo, useState } from 'react';
import { Bot, Plus, Shield, ShieldAlert, FileSearch, Trash2, Upload, Globe, Folder, Package, PenLine, ChevronDown, ChevronRight, Check, ScrollText, LockIcon, Info, Zap, BookOpen, Terminal, Link } from 'lucide-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { commands, ModeConfig, ProjectConfig, Permissions, McpServerConfig, McpServerType } from '@/bindings';
import { DEFAULT_STATUSES } from '@/lib/workspace-ui';
import { Alert, AlertDescription } from '@ship/ui';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Card, CardContent } from '@ship/ui';
import { Input } from '@ship/ui';
import { Label } from '@ship/ui';
import { PageFrame, PageHeader } from '@ship/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/ui';
import { Textarea } from '@ship/ui';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@ship/ui';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/ui';
import MarkdownEditor from '@/components/editor';
import { AutocompleteInput } from '@ship/ui';
import { cn } from '@/lib/utils';

interface AgentsPanelProps {
  projectConfig: ProjectConfig | null;
  globalAgentConfig: ProjectConfig | null;
  onSaveProject: (config: ProjectConfig) => void | Promise<void>;
  onSaveGlobalAgentConfig: (config: ProjectConfig) => void | Promise<void>;
  initialSection?: AgentSection;
}

type ScopeKey = 'global' | 'project';
type MarkdownDocKind = 'skills' | 'rules';

type AgentDoc = {
  id: string;
  title: string;
  content: string;
  updated: string;
};

export type AgentSection = 'providers' | 'mcp' | 'skills' | 'rules' | 'permissions';

const PROVIDER_LOGO: Record<string, { src: string; invertDark?: boolean }> = {
  claude: { src: '/provider-logos/claude.svg' },
  gemini: { src: '/provider-logos/googlegemini.svg' },
  codex: { src: '/provider-logos/OpenAI-black-monoblossom.svg', invertDark: true },
};

const EMPTY_AGENT_LAYER = {
  skills: [],
  prompts: [],
  context: [],
  rules: [],
};

const DEFAULT_MODE_VALUE = 'default';

const SECTION_META: Record<AgentSection, { title: string; description: string }> = {
  providers: {
    title: 'Providers',
    description: 'Choose provider/model defaults and mode controls.',
  },
  mcp: {
    title: 'MCP Servers',
    description: 'Connect tools and services to your agent via the Model Context Protocol.',
  },
  skills: {
    title: 'Skills',
    description: 'Skill SDK — compose agent capabilities from structured skill packages.',
  },
  rules: {
    title: 'Rules',
    description: 'Global instructions applied to every agent session in this scope.',
  },
  permissions: {
    title: 'Permissions',
    description: 'Security policy: control what tools and paths your agent can access.',
  },
};

const EMPTY_MODE: ModeConfig = {
  id: '',
  name: '',
  description: null,
  active_tools: [],
  mcp_servers: [],
};

const EMPTY_MCP_SERVER: McpServerConfig = {
  name: '',
  command: '',
  args: [],
  url: null,
  timeout_secs: null,
};

type McpEditDraft = {
  idx: number | null;
  server: McpServerConfig;
};

// ── Permission presets ──────────────────────────────────────────────────────

const PERMISSION_PRESETS: Array<{
  id: string;
  name: string;
  description: string;
  icon: React.ElementType;
  colorClass: string;
  apply: () => Permissions;
}> = [
  {
    id: 'readonly',
    name: 'Read-only',
    description: 'Read files and run read-only MCP tools. No writes, no shell execution.',
    icon: FileSearch,
    colorClass: 'text-blue-500',
    apply: () => ({
      tools: { allow: ['mcp__*__read*', 'mcp__*__list*', 'mcp__*__get*', 'mcp__*__search*'], deny: ['mcp__*__write*', 'mcp__*__delete*', 'mcp__*__create*', 'mcp__*__exec*'] },
      filesystem: { allow: ['**/*'], deny: [] },
      agent: { max_cost_per_session: 2.0, max_turns: 30 },
    }),
  },
  {
    id: 'standard',
    name: 'Standard',
    description: 'Balanced defaults — read/write allowed, sensitive paths blocked.',
    icon: Shield,
    colorClass: 'text-emerald-500',
    apply: () => ({
      tools: { allow: ['*'], deny: [] },
      filesystem: { allow: ['**/*'], deny: ['/etc/**', '/sys/**', '/proc/**', '~/.ssh/**', '~/.gnupg/**'] },
      agent: { max_cost_per_session: 5.0, max_turns: 50 },
    }),
  },
  {
    id: 'yolo',
    name: 'Full Access',
    description: 'No restrictions. Agent can do anything. Use only in trusted environments.',
    icon: ShieldAlert,
    colorClass: 'text-rose-500',
    apply: () => ({
      tools: { allow: ['*'], deny: [] },
      filesystem: { allow: ['**/*'], deny: [] },
      agent: { max_cost_per_session: null, max_turns: null },
    }),
  },
];

// ── McpServerForm ───────────────────────────────────────────────────────────

function McpServerForm({
  draft,
  onChange,
  onSave,
  onCancel,
  isNew,
}: {
  draft: McpServerConfig;
  onChange: (server: McpServerConfig) => void;
  onSave: () => void;
  onCancel: () => void;
  isNew?: boolean;
}) {
  const transport = draft.server_type ?? 'stdio';
  const argsStr = (draft.args ?? []).join(' ');
  const setField = <K extends keyof McpServerConfig>(key: K, value: McpServerConfig[K]) =>
    onChange({ ...draft, [key]: value });

  return (
    <div className="border-t bg-muted/20 px-4 py-4 space-y-3">
      <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
        {isNew ? 'New MCP Server' : 'Edit Server'}
      </p>

      <div className="grid gap-3 sm:grid-cols-[1fr_auto]">
        <div className="space-y-1.5">
          <Label className="text-xs">Name</Label>
          <Input
            value={draft.name}
            onChange={(e) => setField('name', e.target.value)}
            placeholder="e.g. shipwright"
            className="h-8 text-xs"
            autoFocus={isNew}
          />
        </div>
        <div className="space-y-1.5">
          <Label className="text-xs">Transport</Label>
          <Select value={transport} onValueChange={(v) => setField('server_type', v as McpServerType)}>
            <SelectTrigger size="sm" className="w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="stdio">stdio</SelectItem>
              <SelectItem value="sse">SSE</SelectItem>
              <SelectItem value="http">HTTP</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {transport === 'stdio' ? (
        <div className="grid gap-3 sm:grid-cols-[1fr_1fr]">
          <div className="space-y-1.5">
            <Label className="text-xs">Command</Label>
            <Input
              value={draft.command}
              onChange={(e) => setField('command', e.target.value)}
              placeholder="e.g. ship-mcp"
              className="h-8 text-xs font-mono"
            />
          </div>
          <div className="space-y-1.5">
            <div className="flex items-center gap-1.5">
              <Label className="text-xs">Arguments</Label>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="size-3 cursor-default text-muted-foreground" />
                </TooltipTrigger>
                <TooltipContent>Space-separated args passed to the command.</TooltipContent>
              </Tooltip>
            </div>
            <Input
              value={argsStr}
              onChange={(e) => setField('args', e.target.value.split(/\s+/).filter(Boolean))}
              placeholder="--port 3000"
              className="h-8 text-xs font-mono"
            />
          </div>
        </div>
      ) : (
        <div className="space-y-1.5">
          <Label className="text-xs">URL</Label>
          <Input
            value={draft.url ?? ''}
            onChange={(e) => setField('url', e.target.value || null)}
            placeholder="https://my-mcp-server.example.com"
            className="h-8 text-xs font-mono"
          />
        </div>
      )}

      {/* Env vars */}
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">Environment Variables</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>Injected into the server process. Use for API keys and secrets.</TooltipContent>
            </Tooltip>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="xs"
            className="h-5 px-1.5 text-[10px]"
            onClick={() => {
              const envCopy = { ...(draft.env ?? {}) };
              envCopy['NEW_KEY'] = '';
              setField('env', envCopy);
            }}
          >
            <Plus className="mr-0.5 size-3" />Add
          </Button>
        </div>
        {draft.env && Object.entries(draft.env).length > 0 && (
          <div className="space-y-2">
            {Object.entries(draft.env).map(([key, val], envIdx) => (
              <div key={envIdx} className="flex items-center gap-2">
                <Input
                  value={key}
                  onChange={(e) => {
                    const entries = Object.entries(draft.env ?? {});
                    entries[envIdx] = [e.target.value, val ?? ''];
                    setField('env', Object.fromEntries(entries));
                  }}
                  placeholder="KEY"
                  className="h-7 w-32 shrink-0 text-xs font-mono"
                />
                <span className="text-xs text-muted-foreground">=</span>
                <Input
                  value={val ?? ''}
                  onChange={(e) => {
                    const envCopy = { ...(draft.env ?? {}) };
                    envCopy[key] = e.target.value;
                    setField('env', envCopy);
                  }}
                  placeholder="value"
                  className="h-7 flex-1 text-xs font-mono"
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="xs"
                  className="h-7 w-7 shrink-0 p-0"
                  onClick={() => {
                    const envCopy = { ...(draft.env ?? {}) };
                    delete envCopy[key];
                    setField('env', envCopy);
                  }}
                >
                  <Trash2 className="size-3" />
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="flex items-center gap-2 pt-1">
        <Button
          size="sm"
          onClick={onSave}
          disabled={!draft.name.trim() || (transport === 'stdio' && !draft.command.trim()) || (transport !== 'stdio' && !draft.url?.trim())}
        >
          Save
        </Button>
        <Button size="sm" variant="ghost" onClick={onCancel}>Cancel</Button>
      </div>
    </div>
  );
}

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
        config?.git?.commit ?? ['releases', 'features', 'adrs', 'specs', 'ship.toml', 'templates'],
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

function formatUpdated(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
}

// ── AgentsPanel ─────────────────────────────────────────────────────────────

export default function AgentsPanel({
  projectConfig,
  globalAgentConfig,
  onSaveProject,
  onSaveGlobalAgentConfig,
  initialSection = 'providers',
}: AgentsPanelProps) {
  const queryClient = useQueryClient();
  const [localProject, setLocalProject] = useState<ProjectConfig>(normalizeProjectConfig(projectConfig));
  const [localGlobalAgent, setLocalGlobalAgent] = useState<ProjectConfig>(
    normalizeProjectConfig(globalAgentConfig)
  );
  const [agentScope, setAgentScope] = useState<ScopeKey>(projectConfig ? 'project' : 'global');
  const [newMode, setNewMode] = useState<ModeConfig>(EMPTY_MODE);
  const [expandedModeId, setExpandedModeId] = useState<string | null>(null);
  const [editingMode, setEditingMode] = useState<ModeConfig | null>(null);
  const [exportStatus, setExportStatus] = useState<Record<string, 'idle' | 'loading' | 'ok' | 'error'>>({});
  const [agentError, setAgentError] = useState<string | null>(null);
  const [mcpEditDraft, setMcpEditDraft] = useState<McpEditDraft | null>(null);

  const [selectedDocIds, setSelectedDocIds] = useState<Record<ScopeKey, Record<MarkdownDocKind, string | null>>>(
    () => ({
      global: { skills: null, rules: null },
      project: { skills: null, rules: null },
    })
  );

  const activeDocKind: MarkdownDocKind | null =
    initialSection === 'skills' || initialSection === 'rules' ? initialSection : null;

  // Providers Query
  const { data: providers = [] } = useQuery({
    queryKey: ['providers'],
    queryFn: async () => {
      const res = await commands.listProvidersCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'providers',
  });

  // Skills Query
  const { data: skills = [] } = useQuery({
    queryKey: ['skills', agentScope],
    queryFn: async () => {
      const res = await commands.listSkillsCmd(agentScope === 'project' ? 'project' : 'global');
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'skills' || initialSection === 'providers',
  });

  // Rules Query
  const { data: rules = [] } = useQuery({
    queryKey: ['rules'],
    queryFn: async () => {
      const res = await commands.listRulesCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'rules',
  });

  const activeDocs =
    activeDocKind === 'skills'
      ? skills.map((s) => ({ id: s.id, title: s.name, content: s.content, updated: '' }))
      : rules.map((r) => ({ id: r.file_name, title: r.file_name, content: r.content, updated: '' }));

  const activeSelectedDocId = activeDocKind ? selectedDocIds[agentScope][activeDocKind] : null;
  const activeDoc = activeDocs.find((doc) => doc.id === activeSelectedDocId) ?? activeDocs[0] ?? null;

  // Mutations
  const createSkillMut = useMutation({
    mutationFn: async (vars: { id: string; name: string; content: string }) => {
      const res = await commands.createSkillCmd(vars.id, vars.name, vars.content, agentScope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: (newSkill) => {
      queryClient.invalidateQueries({ queryKey: ['skills', agentScope] });
      setSelectedDocIds((curr) => ({
        ...curr,
        [agentScope]: { ...curr[agentScope], skills: newSkill.id },
      }));
    },
  });

  const updateSkillMut = useMutation({
    mutationFn: async (vars: { id: string; name?: string; content?: string }) => {
      const res = await commands.updateSkillCmd(vars.id, vars.name ?? null, vars.content ?? null, agentScope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills', agentScope] });
    },
  });

  const deleteSkillMut = useMutation({
    mutationFn: async (id: string) => {
      const res = await commands.deleteSkillCmd(id, agentScope);
      if (res.status === 'error') throw new Error(res.error);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills', agentScope] });
      setSelectedDocIds((curr) => ({ ...curr, [agentScope]: { ...curr[agentScope], skills: null } }));
    },
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
  useEffect(() => { if (!projectConfig) setAgentScope('global'); }, [projectConfig]);

  const hasActiveProject = !!projectConfig;
  const activeAgentConfig = useMemo(
    () => (agentScope === 'project' ? localProject : localGlobalAgent),
    [agentScope, localGlobalAgent, localProject]
  );

  // Permissions Query
  const { data: permissions } = useQuery({
    queryKey: ['permissions'],
    queryFn: async () => {
      const res = await commands.getPermissionsCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'permissions',
  });

  const savePermissionsMut = useMutation({
    mutationFn: async (p: Permissions) => {
      const res = await commands.savePermissionsCmd(p);
      if (res.status === 'error') throw new Error(res.error);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['permissions'] });
    },
  });

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

  const handleAddMode = () => {
    const id = newMode.id.trim();
    const name = newMode.name.trim();
    if (!id || !name) return;
    updateActiveAgentConfig({
      ...activeAgentConfig,
      modes: [...(activeAgentConfig.modes ?? []), { ...newMode, id, name }],
    });
    setNewMode(EMPTY_MODE);
  };

  const handleRemoveMode = (id: string) => {
    updateActiveAgentConfig({
      ...activeAgentConfig,
      modes: (activeAgentConfig.modes ?? []).filter((m: ModeConfig) => m.id !== id),
      active_mode: activeAgentConfig.active_mode === id ? null : activeAgentConfig.active_mode,
    });
  };

  const handleSetActiveMode = (id: string | null) => {
    const next = id === DEFAULT_MODE_VALUE ? null : id;
    updateActiveAgentConfig({ ...activeAgentConfig, active_mode: next });
  };

  const handleToggleProvider = (providerId: string, currentlyEnabled: boolean) => {
    const next = currentlyEnabled
      ? (activeAgentConfig.providers ?? []).filter((id) => id !== providerId)
      : [...(activeAgentConfig.providers ?? []), providerId];
    const updated = { ...activeAgentConfig, providers: next };
    updateActiveAgentConfig(updated);
    if (agentScope === 'project') void onSaveProject(updated);
    else void onSaveGlobalAgentConfig(updated);
  };

  const handleEditModeStart = (mode: ModeConfig) => {
    setExpandedModeId(mode.id);
    setEditingMode({ ...mode });
  };

  const handleEditModeCancel = () => {
    setExpandedModeId(null);
    setEditingMode(null);
  };

  const handleEditModeSave = () => {
    if (!editingMode) return;
    updateActiveAgentConfig({
      ...activeAgentConfig,
      modes: (activeAgentConfig.modes ?? []).map((m) => m.id === editingMode.id ? editingMode : m),
    });
    setExpandedModeId(null);
    setEditingMode(null);
  };

  const handleExport = async (target: string) => {
    setExportStatus((prev) => ({ ...prev, [target]: 'loading' }));
    setAgentError(null);
    try {
      const res = await commands.exportAgentConfigCmd(target);
      if (res.status === 'error') throw new Error(res.error);
      setExportStatus((prev) => ({ ...prev, [target]: 'ok' }));
    } catch (err) {
      setExportStatus((prev) => ({ ...prev, [target]: 'error' }));
      setAgentError(String(err));
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
    const servers = [...(activeAgentConfig.mcp_servers ?? [])];
    if (mcpEditDraft.idx === null) {
      servers.push(mcpEditDraft.server);
    } else {
      servers[mcpEditDraft.idx] = mcpEditDraft.server;
    }
    updateActiveAgentConfig({ ...activeAgentConfig, mcp_servers: servers });
    setMcpEditDraft(null);
  };

  const handleSave = () => {
    if (agentScope === 'global') return onSaveGlobalAgentConfig(localGlobalAgent);
    return onSaveProject(localProject);
  };

  const sectionMeta = SECTION_META[initialSection];

  const scopeToggle = (
    <div className="flex items-center gap-1 rounded-lg border bg-muted/50 p-1">
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
      <button
        type="button"
        disabled={!hasActiveProject}
        onClick={() => hasActiveProject && setAgentScope('project')}
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
    </div>
  );

  return (
    <PageFrame className="md:p-8">
      <PageHeader
        title={sectionMeta.title}
        description={sectionMeta.description}
        badge={<Badge variant="outline">Agents</Badge>}
        actions={scopeToggle}
      />

      <div className="grid gap-4">

        {/* ════════════════════════════════════════════════════════════════
            PROVIDERS
        ════════════════════════════════════════════════════════════════ */}
        {initialSection === 'providers' && (
          <div className="grid gap-4">

            {/* ── AI Clients ── */}
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center justify-between border-b px-4 py-3">
                <div className="flex items-center gap-3">
                  <div className="flex size-7 items-center justify-center rounded-lg border border-primary/20 bg-primary/10">
                    <Bot className="size-3.5 text-primary" />
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold">AI Clients</h3>
                    <p className="text-[11px] text-muted-foreground">Ship manages your agent config. Export anytime to take back control.</p>
                  </div>
                </div>
              </div>
              <div className="divide-y divide-border/50">
                {providers.length === 0 && (
                  <p className="px-4 py-6 text-center text-sm text-muted-foreground">Detecting providers…</p>
                )}
                {providers.map((provider) => {
                  const isEnabled = (activeAgentConfig.providers ?? []).includes(provider.id);
                  const logo = PROVIDER_LOGO[provider.id];
                  return (
                    <div key={provider.id} className="flex items-center gap-3 px-4 py-3">
                      <div className="flex size-8 shrink-0 items-center justify-center rounded-lg border bg-card">
                        {logo
                          ? <img src={logo.src} alt={provider.name} className={cn('size-5 object-contain', logo.invertDark && 'dark:invert')} />
                          : <Bot className="size-4 text-muted-foreground" />
                        }
                      </div>

                      <div className="min-w-0 flex-1">
                        <p className="text-sm font-medium">{provider.name}</p>
                        <p className="font-mono text-[11px] text-muted-foreground">{provider.binary}</p>
                      </div>

                      {provider.installed ? (
                        <Badge variant="outline" className="shrink-0 border-emerald-500/30 bg-emerald-500/10 text-[10px] text-emerald-600 dark:text-emerald-400">
                          {provider.version ?? 'installed'}
                        </Badge>
                      ) : (
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Badge variant="outline" className="shrink-0 cursor-default text-[10px] text-muted-foreground">
                              not found
                            </Badge>
                          </TooltipTrigger>
                          <TooltipContent>
                            {provider.binary} not found on PATH. Install this provider to enable it.
                          </TooltipContent>
                        </Tooltip>
                      )}

                      <Tooltip>
                        <TooltipTrigger asChild>
                          <button
                            type="button"
                            disabled={!hasActiveProject}
                            onClick={() => handleToggleProvider(provider.id, isEnabled)}
                            className={cn(
                              'relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-40',
                              isEnabled ? 'bg-primary' : 'bg-muted'
                            )}
                            role="switch"
                            aria-checked={isEnabled}
                          >
                            <span className={cn(
                              'pointer-events-none block size-4 rounded-full bg-background shadow-sm ring-0 transition-transform',
                              isEnabled ? 'translate-x-4' : 'translate-x-0'
                            )} />
                          </button>
                        </TooltipTrigger>
                        <TooltipContent>
                          {isEnabled
                            ? 'Disable — remove from exported agent configs'
                            : 'Enable — include in exported agent configs'}
                        </TooltipContent>
                      </Tooltip>

                      <div className="flex shrink-0 items-center gap-1">
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              type="button"
                              variant="outline"
                              size="xs"
                              disabled={!isEnabled || !hasActiveProject || exportStatus[provider.id] === 'loading'}
                              onClick={() => void handleExport(provider.id)}
                              className="h-6 px-2 text-[10px]"
                            >
                              <Upload className="mr-1 size-3" />
                              {exportStatus[provider.id] === 'loading' ? 'Exporting…'
                                : exportStatus[provider.id] === 'ok' ? 'Exported ✓'
                                : 'Export'}
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>
                            Push Ship's unified config to {provider.name}'s native config files
                          </TooltipContent>
                        </Tooltip>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              type="button"
                              variant="outline"
                              size="xs"
                              disabled
                              className="h-6 px-2 text-[10px] opacity-40"
                            >
                              Import
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>
                            Pull {provider.name}'s existing config into Ship — coming soon
                          </TooltipContent>
                        </Tooltip>
                      </div>
                    </div>
                  );
                })}
              </div>
            </Card>

            {/* ── Ship Generation ── */}
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-muted bg-muted/50">
                  <Bot className="size-3.5 text-muted-foreground" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">Ship Generation</h3>
                  <p className="text-[11px] text-muted-foreground">Provider Ship uses for its own AI features — descriptions, generation, analysis.</p>
                </div>
              </div>
              <CardContent className="grid gap-3 !pt-4 sm:grid-cols-[1fr_1fr_1fr]">
                <div className="space-y-1.5">
                  <div className="flex items-center gap-1.5">
                    <Label className="text-xs">Provider</Label>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent>Which installed AI client Ship uses for generation features</TooltipContent>
                    </Tooltip>
                  </div>
                  <Select
                    value={activeAgentConfig.ai?.provider ?? 'claude'}
                    onValueChange={(value) =>
                      updateActiveAgentConfig({
                        ...activeAgentConfig,
                        ai: { ...normalizeAiConfig(activeAgentConfig.ai), provider: value, model: null },
                      })
                    }
                  >
                    <SelectTrigger size="sm">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {providers.filter((p) => p.installed).map((p) => (
                        <SelectItem key={p.id} value={p.id}>
                          <div className="flex items-center gap-2">
                            {PROVIDER_LOGO[p.id] && <img src={PROVIDER_LOGO[p.id].src} alt="" className={cn('size-3.5 object-contain', PROVIDER_LOGO[p.id].invertDark && 'dark:invert')} />}
                            {p.name}
                          </div>
                        </SelectItem>
                      ))}
                      {providers.filter((p) => p.installed).length === 0 && (
                        <SelectItem value="claude">Claude (default)</SelectItem>
                      )}
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-1.5">
                  <div className="flex items-center gap-1.5">
                    <Label className="text-xs">Model</Label>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent>Leave blank to use the provider's default model. Type a custom model ID if needed.</TooltipContent>
                    </Tooltip>
                  </div>
                  <AutocompleteInput
                    value={activeAgentConfig.ai?.model ?? ''}
                    options={(
                      providers.find((p) => p.id === (activeAgentConfig.ai?.provider ?? 'claude'))?.models ?? []
                    ).map((m) => ({ value: m.id, label: m.name }))}
                    placeholder="Default"
                    noResultsText="Type a custom model ID."
                    onValueChange={(value) =>
                      updateActiveAgentConfig({
                        ...activeAgentConfig,
                        ai: { ...normalizeAiConfig(activeAgentConfig.ai), model: value || null },
                      })
                    }
                  />
                </div>

                <div className="space-y-1.5">
                  <div className="flex items-center gap-1.5">
                    <Label className="text-xs">CLI Path Override</Label>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent>Absolute path to the provider binary. Leave blank to resolve from PATH.</TooltipContent>
                    </Tooltip>
                  </div>
                  <Input
                    value={activeAgentConfig.ai?.cli_path ?? ''}
                    onChange={(event) =>
                      updateActiveAgentConfig({
                        ...activeAgentConfig,
                        ai: { ...normalizeAiConfig(activeAgentConfig.ai), cli_path: event.target.value || null },
                      })
                    }
                    placeholder="Leave blank to use PATH"
                    className="h-8 text-xs"
                  />
                </div>
              </CardContent>
            </Card>

            {/* ── Modes ── */}
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center justify-between border-b px-4 py-3">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="text-sm font-semibold">Modes</h3>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent className="max-w-xs">
                        A mode bundles MCP servers, a skill (system prompt), and tool restrictions into a named capability preset. Your agent runs with the active mode's config — provider-agnostic.
                      </TooltipContent>
                    </Tooltip>
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    Capability presets — each mode bundles MCP servers, skills, and tool access.
                  </p>
                </div>
                {(activeAgentConfig.modes ?? []).length > 0 && (
                  <Badge variant="secondary" className="ml-3 shrink-0 text-[10px]">
                    {activeAgentConfig.active_mode
                      ? (activeAgentConfig.modes ?? []).find(m => m.id === activeAgentConfig.active_mode)?.name ?? 'Custom'
                      : 'Default'} active
                  </Badge>
                )}
              </div>

              <div className="divide-y divide-border/50">
                {(activeAgentConfig.modes ?? []).length === 0 && (
                  <div className="flex flex-col items-center gap-2 px-4 py-8 text-center">
                    <p className="text-sm text-muted-foreground">No modes defined — agent runs with all capabilities.</p>
                    <p className="text-[11px] text-muted-foreground/60">Create a mode to restrict or focus what your agent can access.</p>
                  </div>
                )}

                {(activeAgentConfig.modes ?? []).map((mode) => {
                  const isActive = mode.id === activeAgentConfig.active_mode;
                  const isExpanded = expandedModeId === mode.id;
                  const editing = isExpanded && editingMode?.id === mode.id ? editingMode : null;
                  const linkedSkill = skills.find((s) => s.id === mode.prompt_id);
                  const mcpCount = (mode.mcp_servers ?? []).length;
                  const toolCount = (mode.active_tools ?? []).length;

                  return (
                    <div key={mode.id} className={cn('transition-colors', isExpanded && 'bg-muted/30')}>
                      <div
                        className="flex cursor-pointer items-center gap-3 px-4 py-3 hover:bg-muted/20"
                        onClick={() => isExpanded ? handleEditModeCancel() : handleEditModeStart(mode)}
                      >
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2">
                            <p className="text-sm font-medium">{mode.name}</p>
                            {isActive && (
                              <Badge variant="outline" className="border-primary/30 bg-primary/10 px-1.5 py-0 text-[9px] text-primary">
                                active
                              </Badge>
                            )}
                          </div>
                          {mode.description && (
                            <p className="truncate text-[11px] text-muted-foreground">{mode.description}</p>
                          )}
                        </div>

                        <div className="flex shrink-0 items-center gap-1.5">
                          {linkedSkill && (
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Badge variant="secondary" className="cursor-default text-[10px]">
                                  <ScrollText className="mr-1 size-2.5" />{linkedSkill.name}
                                </Badge>
                              </TooltipTrigger>
                              <TooltipContent>Linked skill — used as this mode's system prompt</TooltipContent>
                            </Tooltip>
                          )}
                          {mcpCount > 0 && (
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Badge variant="secondary" className="cursor-default text-[10px]">
                                  <Package className="mr-1 size-2.5" />{mcpCount} MCP
                                </Badge>
                              </TooltipTrigger>
                              <TooltipContent>
                                {(mode.mcp_servers ?? []).join(', ')}
                              </TooltipContent>
                            </Tooltip>
                          )}
                          {toolCount > 0 && (
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Badge variant="secondary" className="cursor-default text-[10px]">{toolCount} tools</Badge>
                              </TooltipTrigger>
                              <TooltipContent>Active tool restrictions for this mode</TooltipContent>
                            </Tooltip>
                          )}
                        </div>

                        <div className="flex shrink-0 items-center gap-1">
                          {!isActive && (
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  variant="ghost"
                                  size="xs"
                                  className="h-6 px-2 text-[10px]"
                                  onClick={(e) => { e.stopPropagation(); handleSetActiveMode(mode.id); }}
                                >
                                  <Check className="mr-1 size-3" />Set active
                                </Button>
                              </TooltipTrigger>
                              <TooltipContent>Use this mode for agent sessions in this scope</TooltipContent>
                            </Tooltip>
                          )}
                          <Button
                            variant="ghost"
                            size="xs"
                            className="h-6 w-6 p-0 text-destructive hover:bg-destructive/10 hover:text-destructive"
                            onClick={(e) => { e.stopPropagation(); handleRemoveMode(mode.id); }}
                          >
                            <Trash2 className="size-3.5" />
                          </Button>
                          {isExpanded
                            ? <ChevronDown className="size-3.5 text-muted-foreground" />
                            : <ChevronRight className="size-3.5 text-muted-foreground" />
                          }
                        </div>
                      </div>

                      {isExpanded && editing && (
                        <div className="space-y-3 border-t bg-muted/20 px-4 py-4">
                          <div className="grid gap-3 sm:grid-cols-2">
                            <div className="space-y-1.5">
                              <Label className="text-xs">Name</Label>
                              <Input
                                value={editing.name}
                                onChange={(e) => setEditingMode({ ...editing, name: e.target.value })}
                                className="h-8 text-xs"
                              />
                            </div>
                            <div className="space-y-1.5">
                              <Label className="text-xs">Description</Label>
                              <Input
                                value={editing.description ?? ''}
                                onChange={(e) => setEditingMode({ ...editing, description: e.target.value || null })}
                                placeholder="What this mode is for"
                                className="h-8 text-xs"
                              />
                            </div>
                          </div>

                          <div className="space-y-1.5">
                            <div className="flex items-center gap-1.5">
                              <Label className="text-xs">Linked Skill</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  The skill used as this mode's system prompt. Skills can contain instructions, context, and tool config for the agent.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <Select
                              value={editing.prompt_id ?? 'none'}
                              onValueChange={(v) => setEditingMode({ ...editing, prompt_id: v === 'none' ? null : v })}
                            >
                              <SelectTrigger size="sm" className="w-full sm:w-72">
                                <SelectValue placeholder="No linked skill" />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="none">
                                  <span className="text-muted-foreground">None — no linked skill</span>
                                </SelectItem>
                                {skills.length === 0 && (
                                  <div className="px-2 py-1.5 text-[11px] text-muted-foreground">
                                    No skills yet — create one in the Skills tab
                                  </div>
                                )}
                                {skills.map((s) => (
                                  <SelectItem key={s.id} value={s.id}>{s.name}</SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                          </div>

                          {(activeAgentConfig.mcp_servers ?? []).length > 0 && (
                            <div className="space-y-1.5">
                              <div className="flex items-center gap-1.5">
                                <Label className="text-xs">MCP Servers</Label>
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Info className="size-3 cursor-default text-muted-foreground" />
                                  </TooltipTrigger>
                                  <TooltipContent>Select which MCP servers are active in this mode. Unselected servers won't be started.</TooltipContent>
                                </Tooltip>
                              </div>
                              <div className="flex flex-wrap gap-2">
                                {(activeAgentConfig.mcp_servers ?? []).map((server) => {
                                  const serverId = server.id ?? server.name;
                                  const checked = (editing.mcp_servers ?? []).includes(serverId);
                                  return (
                                    <button
                                      key={serverId}
                                      type="button"
                                      onClick={() => {
                                        const next = checked
                                          ? (editing.mcp_servers ?? []).filter((id) => id !== serverId)
                                          : [...(editing.mcp_servers ?? []), serverId];
                                        setEditingMode({ ...editing, mcp_servers: next });
                                      }}
                                      className={cn(
                                        'flex items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs transition-colors',
                                        checked
                                          ? 'border-primary/40 bg-primary/10 text-primary'
                                          : 'border-border/60 text-muted-foreground hover:border-primary/30 hover:text-foreground'
                                      )}
                                    >
                                      <Package className="size-3" />
                                      {server.name || server.id}
                                    </button>
                                  );
                                })}
                              </div>
                            </div>
                          )}

                          <div className="flex items-center gap-2 pt-1">
                            <Button size="sm" onClick={handleEditModeSave}>Save</Button>
                            <Button size="sm" variant="ghost" onClick={handleEditModeCancel}>Cancel</Button>
                          </div>
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>

              <div className="border-t px-4 py-3">
                <div className="flex items-center gap-2">
                  <Input
                    value={newMode.name}
                    onChange={(e) => setNewMode({
                      ...newMode,
                      name: e.target.value,
                      id: e.target.value.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, ''),
                    })}
                    placeholder="New mode name…"
                    className="h-8 text-xs"
                    onKeyDown={(e) => e.key === 'Enter' && newMode.name.trim() && handleAddMode()}
                  />
                  <Button
                    size="sm"
                    onClick={handleAddMode}
                    disabled={!newMode.name.trim()}
                    className="shrink-0"
                  >
                    <Plus className="mr-1 size-3.5" />Add
                  </Button>
                </div>
              </div>
            </Card>
          </div>
        )}

        {/* ════════════════════════════════════════════════════════════════
            MCP SERVERS
        ════════════════════════════════════════════════════════════════ */}
        {initialSection === 'mcp' && (
          <div className="grid gap-4">
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-violet-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-violet-500/20 bg-violet-500/10">
                  <Package className="size-3.5 text-violet-500" />
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="text-sm font-semibold">MCP Servers</h3>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent className="max-w-xs">
                        MCP (Model Context Protocol) servers extend your agent with tools — file systems, databases, APIs, browser control, and more. Stored in your ship.toml and synced to provider configs on export.
                      </TooltipContent>
                    </Tooltip>
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    Connect tools and services. Stored in ship.toml — exported to each provider on sync.
                  </p>
                </div>
                <Badge variant="secondary" className="shrink-0 text-[10px]">
                  {(activeAgentConfig.mcp_servers ?? []).length} server{(activeAgentConfig.mcp_servers ?? []).length !== 1 ? 's' : ''}
                </Badge>
              </div>

              <div className="divide-y divide-border/50">
                {(activeAgentConfig.mcp_servers ?? []).length === 0 && mcpEditDraft === null && (
                  <div className="flex flex-col items-center gap-2 px-4 py-8 text-center">
                    <Package className="size-8 text-muted-foreground opacity-30" />
                    <p className="text-sm text-muted-foreground">No MCP servers configured.</p>
                    <p className="text-[11px] text-muted-foreground/60">
                      Add servers to give your agent access to tools, APIs, and local services.
                    </p>
                  </div>
                )}

                {(activeAgentConfig.mcp_servers ?? []).map((server, idx) => {
                  const serverId = server.id ?? server.name;
                  const isEditing = mcpEditDraft?.idx === idx;
                  const transport = server.server_type ?? 'stdio';
                  const envCount = server.env ? Object.keys(server.env).length : 0;
                  return (
                    <div key={`${serverId}-${idx}`} className={cn('transition-colors', isEditing && 'bg-muted/30')}>
                      <div className="flex items-center gap-3 px-4 py-3">
                        <div className="flex size-7 shrink-0 items-center justify-center rounded-lg border bg-muted/40">
                          <Package className="size-3.5 text-muted-foreground" />
                        </div>
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2">
                            <p className="text-sm font-medium">{server.name}</p>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Badge variant="outline" className="cursor-default px-1.5 py-0 text-[9px]">
                                  {transport}
                                </Badge>
                              </TooltipTrigger>
                              <TooltipContent>
                                {transport === 'stdio' ? 'Spawned as a local process over stdin/stdout'
                                  : transport === 'sse' ? 'Connected via Server-Sent Events (SSE) stream'
                                  : 'Connected via HTTP request/response'}
                              </TooltipContent>
                            </Tooltip>
                            {server.disabled && (
                              <Badge variant="outline" className="px-1.5 py-0 text-[9px] text-muted-foreground">
                                disabled
                              </Badge>
                            )}
                          </div>
                          <p className="truncate font-mono text-[11px] text-muted-foreground">
                            {transport === 'stdio'
                              ? [server.command, ...(server.args ?? [])].join(' ')
                              : server.url ?? server.command}
                          </p>
                        </div>

                        {envCount > 0 && (
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Badge variant="secondary" className="shrink-0 cursor-default text-[10px]">
                                {envCount} env
                              </Badge>
                            </TooltipTrigger>
                            <TooltipContent>
                              Env vars: {Object.keys(server.env ?? {}).join(', ')}
                            </TooltipContent>
                          </Tooltip>
                        )}

                        <div className="flex shrink-0 items-center gap-1">
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                variant="ghost"
                                size="xs"
                                className="h-6 w-6 p-0"
                                onClick={() => isEditing
                                  ? setMcpEditDraft(null)
                                  : setMcpEditDraft({ idx, server: { ...server } })}
                              >
                                <PenLine className="size-3.5" />
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent>{isEditing ? 'Cancel edit' : 'Edit server'}</TooltipContent>
                          </Tooltip>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                variant="ghost"
                                size="xs"
                                className="h-6 w-6 p-0 text-destructive hover:bg-destructive/10 hover:text-destructive"
                                onClick={() => handleRemoveMcpServer(idx)}
                              >
                                <Trash2 className="size-3.5" />
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent>Remove server</TooltipContent>
                          </Tooltip>
                        </div>
                      </div>

                      {isEditing && mcpEditDraft && (
                        <McpServerForm
                          draft={mcpEditDraft.server}
                          onChange={(s) => setMcpEditDraft({ ...mcpEditDraft, server: s })}
                          onSave={handleSaveMcpServer}
                          onCancel={() => setMcpEditDraft(null)}
                        />
                      )}
                    </div>
                  );
                })}

                {mcpEditDraft?.idx === null && (
                  <McpServerForm
                    draft={mcpEditDraft.server}
                    onChange={(s) => setMcpEditDraft({ idx: null, server: s })}
                    onSave={handleSaveMcpServer}
                    onCancel={() => setMcpEditDraft(null)}
                    isNew
                  />
                )}
              </div>

              {mcpEditDraft === null && (
                <div className="border-t px-4 py-3">
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full border-dashed"
                    onClick={() => setMcpEditDraft({ idx: null, server: { ...EMPTY_MCP_SERVER } })}
                  >
                    <Plus className="mr-1.5 size-3.5" />
                    Add MCP Server
                  </Button>
                </div>
              )}
            </Card>
          </div>
        )}

        {/* ════════════════════════════════════════════════════════════════
            SKILLS / RULES
        ════════════════════════════════════════════════════════════════ */}
        {(initialSection === 'skills' || initialSection === 'rules') && activeDocKind && (
          <div className="grid gap-4">
            {initialSection === 'skills' && (
              <Alert className="border-cyan-500/20 bg-cyan-500/5">
                <Zap className="size-4 text-cyan-500" />
                <AlertDescription className="space-y-1 text-xs">
                  <p>
                    <span className="font-semibold">Skills are a full SDK</span>, not just markdown. A skill package can include YAML config, prompt templates, MCP tool bindings, hooks, and multi-file logic — similar to a lightweight app.
                  </p>
                  <p className="text-muted-foreground">
                    The editor below lets you author and edit skill documents. The full <span className="text-foreground font-medium">Skill Studio</span> with file tree editing is coming soon.
                  </p>
                  <p className="text-muted-foreground">
                    💡 Want to build a skill? Ask <span className="text-foreground font-medium">Ship AI</span> — it understands the skill format and SDK, and can scaffold one for you.
                  </p>
                </AlertDescription>
              </Alert>
            )}

            <div className="grid gap-4 xl:grid-cols-[300px_minmax(0,1fr)]">
              <Card size="sm" className="overflow-hidden xl:h-[640px]">
                <div className="flex items-center gap-3 border-b bg-gradient-to-r from-cyan-500/10 via-card/80 to-card/50 px-4 py-3">
                  <div className="flex size-7 items-center justify-center rounded-lg border border-cyan-500/20 bg-cyan-500/10">
                    {initialSection === 'skills' ? <BookOpen className="size-3.5 text-cyan-500" /> : <ScrollText className="size-3.5 text-cyan-500" />}
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold">{initialSection === 'skills' ? 'Skills' : 'Rules'}</h3>
                    <p className="text-[11px] text-muted-foreground">
                      {initialSection === 'skills' ? `${agentScope} scope` : 'global scope'}
                    </p>
                  </div>
                </div>
                <CardContent className="space-y-2 !pt-5">
                  <Button variant="outline" size="sm" className="w-full" onClick={() => handleCreateDoc(activeDocKind)}>
                    <Plus className="size-3.5" />
                    New {initialSection === 'skills' ? 'Skill' : 'Rule'}
                  </Button>
                  <div className="max-h-[500px] space-y-1 overflow-auto pr-1">
                    {activeDocs.length === 0 && (
                      <p className="py-4 text-center text-xs text-muted-foreground">
                        No {initialSection === 'skills' ? 'skills' : 'rules'} yet.
                      </p>
                    )}
                    {activeDocs.map((doc) => {
                      const selected = activeDoc?.id === doc.id;
                      return (
                        <button
                          key={doc.id}
                          type="button"
                          className={`w-full rounded-md border px-2.5 py-2 text-left transition-colors ${selected ? 'border-primary/40 bg-primary/10' : 'hover:bg-muted/50'}`}
                          onClick={() =>
                            setSelectedDocIds((current) => ({
                              ...current,
                              [agentScope]: { ...current[agentScope], [activeDocKind]: doc.id },
                            }))
                          }
                        >
                          <p className="truncate text-sm font-medium">{doc.title || 'Untitled'}</p>
                          <p className="text-xs text-muted-foreground">{formatUpdated(doc.updated)}</p>
                        </button>
                      );
                    })}
                  </div>
                </CardContent>
              </Card>

              <Card size="sm" className="overflow-hidden xl:h-[640px]">
                <div className="flex items-center gap-3 border-b bg-gradient-to-r from-indigo-500/10 via-card/80 to-card/50 px-4 py-3">
                  <div className="flex size-7 items-center justify-center rounded-lg border border-indigo-500/20 bg-indigo-500/10">
                    <PenLine className="size-3.5 text-indigo-500" />
                  </div>
                  <div className="flex-1">
                    <h3 className="text-sm font-semibold">{initialSection === 'skills' ? 'Skill Editor' : 'Rules Editor'}</h3>
                    <p className="text-[11px] text-muted-foreground">
                      {initialSection === 'skills'
                        ? 'Edit skill content — instructions, context, and tool hints for the agent.'
                        : 'Edit rule content — global instructions applied to every session.'}
                    </p>
                  </div>
                  {activeDoc && (
                    <Button
                      variant="ghost"
                      size="xs"
                      className="text-destructive hover:bg-destructive/10 hover:text-destructive"
                      onClick={() => handleDeleteDoc(activeDocKind, activeDoc.id)}
                    >
                      <Trash2 className="mr-1 size-3.5" />
                      Delete
                    </Button>
                  )}
                </div>
                <CardContent className="space-y-3 !pt-5">
                  {!activeDoc ? (
                    <div className="flex h-[400px] flex-col items-center justify-center gap-2 text-center">
                      <ScrollText className="size-8 text-muted-foreground opacity-30" />
                      <p className="text-sm text-muted-foreground">Select or create a document to start editing.</p>
                    </div>
                  ) : (
                    <>
                      <Input
                        value={activeDoc.title}
                        onChange={(event) => handleUpsertDoc(activeDocKind, activeDoc.id, { title: event.target.value })}
                        placeholder="Document title"
                      />
                      <MarkdownEditor
                        label={undefined}
                        value={activeDoc.content}
                        onChange={(value) => handleUpsertDoc(activeDocKind, activeDoc.id, { content: value })}
                        placeholder={initialSection === 'skills' ? '# Skill' : '# Rule'}
                        rows={18}
                        defaultMode="edit"
                        showFrontmatter={false}
                        showStats={false}
                        fillHeight
                      />
                    </>
                  )}
                </CardContent>
              </Card>
            </div>
          </div>
        )}

        {/* ════════════════════════════════════════════════════════════════
            PERMISSIONS
        ════════════════════════════════════════════════════════════════ */}
        {initialSection === 'permissions' && (
          <div className="grid gap-4 lg:grid-cols-[1fr_300px]">
            <div className="space-y-4">
              {/* Rule Sets / Presets */}
              <Card size="sm" className="overflow-hidden">
                <div className="flex items-center gap-3 border-b px-4 py-3">
                  <div className="flex size-7 items-center justify-center rounded-lg border border-primary/20 bg-primary/10">
                    <Zap className="size-3.5 text-primary" />
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <h3 className="text-sm font-semibold">Rule Sets</h3>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Info className="size-3 cursor-default text-muted-foreground" />
                        </TooltipTrigger>
                        <TooltipContent className="max-w-xs">
                          Presets apply a curated bundle of tool allow/deny rules and limits. They overwrite your current permissions — customize further after applying.
                        </TooltipContent>
                      </Tooltip>
                    </div>
                    <p className="text-[11px] text-muted-foreground">Apply a preset, then fine-tune below.</p>
                  </div>
                </div>
                <CardContent className="grid gap-3 !pt-4 sm:grid-cols-3">
                  {PERMISSION_PRESETS.map((preset) => {
                    const Icon = preset.icon;
                    return (
                      <Tooltip key={preset.id}>
                        <TooltipTrigger asChild>
                          <button
                            type="button"
                            className="flex flex-col gap-1.5 rounded-lg border p-3 text-left transition-colors hover:border-primary/40 hover:bg-primary/5"
                            onClick={() => savePermissionsMut.mutate(preset.apply())}
                          >
                            <div className="flex items-center gap-2">
                              <Icon className={cn('size-3.5', preset.colorClass)} />
                              <span className="text-xs font-semibold">{preset.name}</span>
                            </div>
                            <p className="text-[11px] leading-relaxed text-muted-foreground">{preset.description}</p>
                          </button>
                        </TooltipTrigger>
                        <TooltipContent>Apply {preset.name} preset — overwrites current permissions</TooltipContent>
                      </Tooltip>
                    );
                  })}
                </CardContent>
              </Card>

              {/* Capabilities */}
              <Card size="sm" className="overflow-hidden">
                <div className="flex items-center gap-3 border-b bg-gradient-to-r from-rose-500/10 via-card/80 to-card/50 px-4 py-3">
                  <div className="flex size-7 items-center justify-center rounded-lg border border-rose-500/20 bg-rose-500/10">
                    <Shield className="size-3.5 text-rose-500" />
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold">Capabilities</h3>
                    <p className="text-[11px] text-muted-foreground">Fine-grained policy for tools, filesystem access, and session limits.</p>
                  </div>
                </div>
                <CardContent className="space-y-6 !pt-5">
                  {!permissions ? (
                    <p className="py-10 text-center text-sm text-muted-foreground">Loading permissions...</p>
                  ) : (
                    <Tabs defaultValue="tools">
                      <TabsList className="mb-4">
                        <TabsTrigger value="tools">Tools</TabsTrigger>
                        <TabsTrigger value="filesystem">Filesystem</TabsTrigger>
                        <TabsTrigger value="limits">Limits</TabsTrigger>
                      </TabsList>

                      <TabsContent value="tools" className="space-y-6">
                        <div className="grid gap-6 md:grid-cols-2">
                          <div className="space-y-3">
                            <div className="flex items-center gap-2">
                              <Shield className="size-4 text-emerald-500" />
                              <Label>Allow List</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Glob patterns for tools the agent is allowed to use. Use <code>*</code> to allow all, or <code>mcp__server__tool</code> to target specific tools. Allow list is checked first — deny takes precedence.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <p className="text-xs text-muted-foreground">e.g. <code className="font-mono">mcp__*__read*</code> or <code className="font-mono">*</code></p>
                            <div className="space-y-2">
                              {(permissions.tools?.allow || []).map((p, idx) => (
                                <div key={idx} className="flex items-center gap-2">
                                  <Input
                                    value={p || ''}
                                    onChange={(e) => {
                                      const next = [...(permissions.tools?.allow || [])];
                                      next[idx] = e.target.value;
                                      savePermissionsMut.mutate({ ...permissions, tools: { ...permissions.tools, allow: next, deny: permissions.tools?.deny || [] } });
                                    }}
                                    className="font-mono text-xs"
                                  />
                                  <Button
                                    variant="ghost"
                                    size="xs"
                                    onClick={() => {
                                      const next = (permissions.tools?.allow || []).filter((_, i) => i !== idx);
                                      savePermissionsMut.mutate({ ...permissions, tools: { ...permissions.tools, allow: next, deny: permissions.tools?.deny || [] } });
                                    }}
                                  >
                                    <Trash2 className="size-3.5" />
                                  </Button>
                                </div>
                              ))}
                              <Button
                                variant="outline"
                                size="xs"
                                className="w-full border-dashed"
                                onClick={() => {
                                  savePermissionsMut.mutate({
                                    ...permissions,
                                    tools: { ...permissions.tools, allow: [...(permissions.tools?.allow || []), ''], deny: permissions.tools?.deny || [] },
                                  });
                                }}
                              >
                                <Plus className="mr-1 size-3.5" /> Add Pattern
                              </Button>
                            </div>
                          </div>

                          <div className="space-y-3">
                            <div className="flex items-center gap-2">
                              <ShieldAlert className="size-4 text-destructive" />
                              <Label>Deny List</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Deny always overrides allow. Blocked tools will never execute even if they match an allow pattern. Use this to hard-block dangerous operations.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <p className="text-xs text-muted-foreground">e.g. <code className="font-mono">mcp__*__exec*</code> or <code className="font-mono">mcp__*__delete*</code></p>
                            <div className="space-y-2">
                              {(permissions.tools?.deny || []).map((p, idx) => (
                                <div key={idx} className="flex items-center gap-2">
                                  <Input
                                    value={p || ''}
                                    onChange={(e) => {
                                      const next = [...(permissions.tools?.deny || [])];
                                      next[idx] = e.target.value;
                                      savePermissionsMut.mutate({ ...permissions, tools: { ...permissions.tools, deny: next, allow: permissions.tools?.allow || ['*'] } });
                                    }}
                                    className="font-mono text-xs"
                                  />
                                  <Button
                                    variant="ghost"
                                    size="xs"
                                    onClick={() => {
                                      const next = (permissions.tools?.deny || []).filter((_, i) => i !== idx);
                                      savePermissionsMut.mutate({ ...permissions, tools: { ...permissions.tools, deny: next, allow: permissions.tools?.allow || ['*'] } });
                                    }}
                                  >
                                    <Trash2 className="size-3.5" />
                                  </Button>
                                </div>
                              ))}
                              <Button
                                variant="outline"
                                size="xs"
                                className="w-full border-dashed"
                                onClick={() => {
                                  savePermissionsMut.mutate({
                                    ...permissions,
                                    tools: { ...permissions.tools, deny: [...(permissions.tools?.deny || []), ''], allow: permissions.tools?.allow || ['*'] },
                                  });
                                }}
                              >
                                <Plus className="mr-1 size-3.5" /> Add Restriction
                              </Button>
                            </div>
                          </div>
                        </div>
                      </TabsContent>

                      <TabsContent value="filesystem" className="space-y-6">
                        <div className="grid gap-6 md:grid-cols-2">
                          <div className="space-y-3">
                            <div className="flex items-center gap-2">
                              <FileSearch className="size-4 text-emerald-500" />
                              <Label>Read/Write Allow</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Glob patterns for paths the agent can read and write. Use <code>**/*</code> to allow all paths, or scope to specific directories like <code>~/projects/**</code>.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <p className="text-xs text-muted-foreground">e.g. <code className="font-mono">~/projects/**</code> or <code className="font-mono">**/*</code></p>
                            <div className="space-y-2">
                              {(permissions.filesystem?.allow || []).map((p, idx) => (
                                <div key={idx} className="flex items-center gap-2">
                                  <Input
                                    value={p || ''}
                                    onChange={(e) => {
                                      const next = [...(permissions.filesystem?.allow || [])];
                                      next[idx] = e.target.value;
                                      savePermissionsMut.mutate({ ...permissions, filesystem: { ...permissions.filesystem, allow: next, deny: permissions.filesystem?.deny || [] } });
                                    }}
                                    className="font-mono text-xs"
                                  />
                                  <Button
                                    variant="ghost"
                                    size="xs"
                                    onClick={() => {
                                      const next = (permissions.filesystem?.allow || []).filter((_, i) => i !== idx);
                                      savePermissionsMut.mutate({ ...permissions, filesystem: { ...permissions.filesystem, allow: next, deny: permissions.filesystem?.deny || [] } });
                                    }}
                                  >
                                    <Trash2 className="size-3.5" />
                                  </Button>
                                </div>
                              ))}
                              <Button
                                variant="outline"
                                size="xs"
                                className="w-full border-dashed"
                                onClick={() => {
                                  savePermissionsMut.mutate({
                                    ...permissions,
                                    filesystem: { ...permissions.filesystem, allow: [...(permissions.filesystem?.allow || []), ''], deny: permissions.filesystem?.deny || [] },
                                  });
                                }}
                              >
                                <Plus className="mr-1 size-3.5" /> Add Path
                              </Button>
                            </div>
                          </div>

                          <div className="space-y-3">
                            <div className="flex items-center gap-2">
                              <LockIcon className="size-4 text-destructive" />
                              <Label>Block List</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Paths that can never be accessed, even if they match an allow pattern. Block sensitive directories like <code>~/.ssh/**</code> or <code>/etc/**</code>.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <p className="text-xs text-muted-foreground">e.g. <code className="font-mono">~/.ssh/**</code> or <code className="font-mono">/etc/**</code></p>
                            <div className="space-y-2">
                              {(permissions.filesystem?.deny || []).map((p, idx) => (
                                <div key={idx} className="flex items-center gap-2">
                                  <Input
                                    value={p || ''}
                                    onChange={(e) => {
                                      const next = [...(permissions.filesystem?.deny || [])];
                                      next[idx] = e.target.value;
                                      savePermissionsMut.mutate({ ...permissions, filesystem: { ...permissions.filesystem, deny: next, allow: permissions.filesystem?.allow || [] } });
                                    }}
                                    className="font-mono text-xs"
                                  />
                                  <Button
                                    variant="ghost"
                                    size="xs"
                                    onClick={() => {
                                      const next = (permissions.filesystem?.deny || []).filter((_, i) => i !== idx);
                                      savePermissionsMut.mutate({ ...permissions, filesystem: { ...permissions.filesystem, deny: next, allow: permissions.filesystem?.allow || [] } });
                                    }}
                                  >
                                    <Trash2 className="size-3.5" />
                                  </Button>
                                </div>
                              ))}
                              <Button
                                variant="outline"
                                size="xs"
                                className="w-full border-dashed"
                                onClick={() => {
                                  savePermissionsMut.mutate({
                                    ...permissions,
                                    filesystem: { ...permissions.filesystem, deny: [...(permissions.filesystem?.deny || []), ''], allow: permissions.filesystem?.allow || [] },
                                  });
                                }}
                              >
                                <Plus className="mr-1 size-3.5" /> Add Exclusion
                              </Button>
                            </div>
                          </div>
                        </div>
                      </TabsContent>

                      <TabsContent value="limits" className="space-y-6">
                        <div className="grid gap-6 md:grid-cols-2">
                          <div className="space-y-3">
                            <div className="flex items-center gap-2">
                              <Label>Max Cost per Session (USD)</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Spending cap per agent session. The session stops when this limit is reached. Leave blank for unlimited.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <Input
                              type="number"
                              step="0.01"
                              value={permissions.agent?.max_cost_per_session ?? ''}
                              onChange={(e) => savePermissionsMut.mutate({ ...permissions, agent: { ...permissions.agent, max_cost_per_session: parseFloat(e.target.value) || null } })}
                              placeholder="Unlimited"
                            />
                          </div>
                          <div className="space-y-3">
                            <div className="flex items-center gap-2">
                              <Label>Max Turns per Session</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Maximum number of agent steps (tool calls + responses) before the session is halted. Leave blank for unlimited.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <Input
                              type="number"
                              value={permissions.agent?.max_turns ?? ''}
                              onChange={(e) => savePermissionsMut.mutate({ ...permissions, agent: { ...permissions.agent, max_turns: parseInt(e.target.value, 10) || null } })}
                              placeholder="Unlimited"
                            />
                          </div>
                        </div>
                      </TabsContent>
                    </Tabs>
                  )}
                </CardContent>
              </Card>
            </div>

            {/* Reference sidebar */}
            <Card size="sm" className="h-fit overflow-hidden bg-muted/10">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-slate-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-slate-500/20 bg-slate-500/10">
                  <Info className="size-3.5 text-slate-500" />
                </div>
                <h3 className="text-sm font-semibold">Reference</h3>
              </div>
              <CardContent className="space-y-4 text-xs leading-relaxed !pt-5">
                <p>Permissions define the security sandbox for all AI agents in this scope.</p>

                <div className="rounded-md border bg-card p-3 space-y-2">
                  <p className="font-semibold">How rules apply</p>
                  <div className="space-y-1 text-muted-foreground">
                    <p><span className="text-emerald-500 font-medium">Allow</span> patterns are checked first. <code className="font-mono">*</code> allows everything.</p>
                    <p><span className="text-destructive font-medium">Deny</span> always wins — it overrides any matching allow rule.</p>
                    <p>Filesystem rules are separate from tool rules.</p>
                  </div>
                </div>

                <div className="rounded-md border bg-card p-3 space-y-2">
                  <p className="font-semibold">Tool pattern format</p>
                  <div className="space-y-1 text-muted-foreground font-mono">
                    <p>mcp__<span className="text-primary">{'{server}'}</span>__<span className="text-cyan-500">{'{tool}'}</span></p>
                    <p className="not-italic text-[10px] text-muted-foreground/70">e.g. mcp__filesystem__read_file</p>
                    <p className="not-italic text-[10px] text-muted-foreground/70">e.g. mcp__*__write* (all write tools)</p>
                  </div>
                </div>

                <div className="rounded-md border bg-card p-3 space-y-1">
                  <p className="font-semibold">Runtime enforcement</p>
                  <p className="text-muted-foreground">
                    Rules are enforced by the Ship core runtime. An agent cannot bypass these policies even if instructed to.
                  </p>
                </div>

                <div className="rounded-md border bg-card p-3 space-y-1">
                  <p className="font-semibold">Scope</p>
                  <p className="text-muted-foreground">
                    Global permissions apply to all projects. Project permissions layer on top — project deny rules are always honored.
                  </p>
                </div>
              </CardContent>
            </Card>
          </div>
        )}
      </div>

      <footer className="flex items-center justify-end gap-2 border-t pt-4">
        {agentScope === 'global' ? (
          <Button onClick={() => void handleSave()}>Save Global Agent Config</Button>
        ) : (
          <Button onClick={() => void handleSave()} disabled={!projectConfig}>
            Save Project Agent Config
          </Button>
        )}
      </footer>
    </PageFrame>
  );
}
