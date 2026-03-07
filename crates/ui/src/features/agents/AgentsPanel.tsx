import React, { useEffect, useMemo, useState } from 'react';
import { Bot, Plus, Shield, ShieldAlert, FileSearch, Trash2, Upload, LockIcon, ScrollText, Zap, Globe, Folder, Layers, Package, PenLine, Info } from 'lucide-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { commands, ModeConfig, ProjectConfig, Permissions } from '@/bindings';
import { DEFAULT_STATUSES } from '@/lib/workspace-ui';
import { Alert, AlertDescription } from '@ship/ui';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Card, CardContent } from '@ship/ui';
import { Input } from '@ship/ui';
import { Label } from '@ship/ui';
import { PageFrame, PageHeader } from '@ship/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/ui';
import { Separator } from '@ship/ui';
import { Textarea } from '@ship/ui';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@ship/ui';
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

const AI_PROVIDERS = [
  { id: 'claude', label: 'Claude', icon: '🤖' },
  { id: 'gemini', label: 'Gemini', icon: '✦' },
  { id: 'codex', label: 'Codex', icon: '⚡' },
];

const MODEL_SUGGESTIONS: Record<string, string[]> = {
  claude: [
    'claude-opus-4-5',
    'claude-sonnet-4-5',
    'claude-haiku-4-5',
    'claude-3-5-haiku-latest',
    'claude-3-5-sonnet-latest',
    'claude-3-opus-latest',
  ],
  gemini: [
    'gemini-2.5-pro',
    'gemini-2.0-flash',
    'gemini-2.0-flash-thinking-exp',
    'gemini-1.5-pro',
    'gemini-1.5-flash',
  ],
  codex: [
    'gpt-4o',
    'gpt-4o-mini',
    'o1',
    'o1-mini',
    'o3',
    'o4-mini',
  ],
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
    description: 'Edit MCP server snippets and sync client configs.',
  },
  skills: {
    title: 'Skills',
    description: 'Markdown skill docs with list + editor workflow.',
  },
  rules: {
    title: 'Rules',
    description: 'Markdown rules docs with list + editor workflow.',
  },
  permissions: {
    title: 'Permissions',
    description: 'Draft per-scope allow/deny snippets (API integration pending).',
  },
};

const EMPTY_MODE: ModeConfig = {
  id: '',
  name: '',
  description: null,
  active_tools: [],
  mcp_servers: [],
};

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
  const [exportStatus, setExportStatus] = useState<Record<string, 'idle' | 'loading' | 'ok' | 'error'>>({});
  const [testStatus, setTestStatus] = useState<'idle' | 'loading' | 'ok' | 'error'>('idle');
  const [agentError, setAgentError] = useState<string | null>(null);
  const [mcpSnippet, setMcpSnippet] = useState('[]');
  const [mcpSnippetError, setMcpSnippetError] = useState<string | null>(null);

  const [selectedDocIds, setSelectedDocIds] = useState<Record<ScopeKey, Record<MarkdownDocKind, string | null>>>(
    () => ({
      global: {
        skills: null,
        rules: null,
      },
      project: {
        skills: null,
        rules: null,
      },
    })
  );

  const activeDocKind: MarkdownDocKind | null =
    initialSection === 'skills' || initialSection === 'rules' ? initialSection : null;

  // Skills Query
  const { data: skills = [] } = useQuery({
    queryKey: ['skills', agentScope],
    queryFn: async () => {
      const res = await commands.listSkillsCmd(agentScope === 'project' ? 'project' : 'global');
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'skills',
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
      setSelectedDocIds((curr: Record<ScopeKey, Record<MarkdownDocKind, string | null>>) => ({
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
      setSelectedDocIds((curr: Record<ScopeKey, Record<MarkdownDocKind, string | null>>) => ({
        ...curr,
        [agentScope]: { ...curr[agentScope], skills: null },
      }));
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
      setSelectedDocIds((curr: Record<ScopeKey, Record<MarkdownDocKind, string | null>>) => ({
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
      setSelectedDocIds((curr: Record<ScopeKey, Record<MarkdownDocKind, string | null>>) => ({
        ...curr,
        [agentScope]: { ...curr[agentScope], rules: null },
      }));
    },
  });

  useEffect(() => {
    setLocalProject(normalizeProjectConfig(projectConfig));
  }, [projectConfig]);

  useEffect(() => {
    setLocalGlobalAgent(normalizeProjectConfig(globalAgentConfig));
  }, [globalAgentConfig]);

  useEffect(() => {
    if (!projectConfig) {
      setAgentScope('global');
    }
  }, [projectConfig]);

  const hasActiveProject = !!projectConfig;
  const activeAgentConfig = useMemo(
    () => (agentScope === 'project' ? localProject : localGlobalAgent),
    [agentScope, localGlobalAgent, localProject]
  );

  useEffect(() => {
    setMcpSnippet(JSON.stringify(activeAgentConfig.mcp_servers ?? [], null, 2));
    setMcpSnippetError(null);
  }, [activeAgentConfig.mcp_servers, agentScope]);

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
    if (agentScope === 'project') {
      setLocalProject(next);
      return;
    }
    setLocalGlobalAgent(next);
  };

  const handleMcpSnippetChange = (value: string) => {
    setMcpSnippet(value);
    try {
      const parsed = JSON.parse(value) as unknown;
      if (!Array.isArray(parsed)) {
        throw new Error('MCP config must be a JSON array.');
      }
      updateActiveAgentConfig({
        ...activeAgentConfig,
        mcp_servers: parsed as ProjectConfig['mcp_servers'],
      });
      setMcpSnippetError(null);
    } catch (error) {
      setMcpSnippetError(error instanceof Error ? error.message : 'Invalid JSON.');
    }
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
      const id = `skill-${Date.now()}`;
      createSkillMut.mutate({ id, name: title, content: `# ${title}\n` });
    } else {
      const fileName = `rule-${Date.now()}.md`;
      createRuleMut.mutate({ fileName, content: `# ${title}\n` });
    }
  };

  const handleDeleteDoc = (kind: MarkdownDocKind, docId: string) => {
    if (kind === 'skills') {
      deleteSkillMut.mutate(docId);
    } else {
      deleteRuleMut.mutate(docId);
    }
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

  const handleTestProvider = async () => {
    setTestStatus('loading');
    setAgentError(null);
    try {
      const res = await commands.generateIssueDescriptionCmd('test task');
      if (res.status === 'error') throw new Error(res.error);
      setTestStatus('ok');
    } catch (err) {
      setTestStatus('error');
      setAgentError(String(err));
    }
  };

  const handleExport = async (target: string) => {
    setExportStatus((prev: Record<string, 'idle' | 'loading' | 'ok' | 'error'>) => ({ ...prev, [target]: 'loading' }));
    setAgentError(null);
    try {
      const res = await commands.exportAgentConfigCmd(target);
      if (res.status === 'error') throw new Error(res.error);
      setExportStatus((prev: Record<string, 'idle' | 'loading' | 'ok' | 'error'>) => ({ ...prev, [target]: 'ok' }));
    } catch (err) {
      setExportStatus((prev: Record<string, 'idle' | 'loading' | 'ok' | 'error'>) => ({ ...prev, [target]: 'error' }));
      setAgentError(String(err));
    }
  };

  const handleSave = () => {
    if (agentScope === 'global') {
      return onSaveGlobalAgentConfig(localGlobalAgent);
    }
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
        {initialSection === 'providers' && (
          <div className="grid gap-4">
            {/* Agent Selection Card */}
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-primary/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-8 items-center justify-center rounded-lg bg-primary/10 border border-primary/20">
                  <Bot className="size-4 text-primary" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">Agent Selection</h3>
                  <p className="text-[11px] text-muted-foreground">Choose provider and model for AI generation features.</p>
                </div>
              </div>
              <CardContent className="space-y-4 !pt-5">
                {/* Provider pill selector */}
                <div className="space-y-2">
                  <Label>Provider</Label>
                  <div className="flex flex-wrap gap-2">
                    {AI_PROVIDERS.map((provider) => {
                      const active = (activeAgentConfig.ai?.provider ?? 'claude') === provider.id;
                      return (
                        <button
                          key={provider.id}
                          type="button"
                          onClick={() =>
                            updateActiveAgentConfig({
                              ...activeAgentConfig,
                              ai: { ...normalizeAiConfig(activeAgentConfig.ai), provider: provider.id },
                            })
                          }
                          className={cn(
                            'flex items-center gap-2 rounded-lg border px-3 py-2 text-sm font-medium transition-all',
                            active
                              ? 'border-primary/50 bg-primary/10 text-primary shadow-sm'
                              : 'border-border/50 bg-card hover:border-primary/30 hover:bg-primary/5 text-muted-foreground'
                          )}
                        >
                          <span className="text-base leading-none">{provider.icon}</span>
                          {provider.label}
                        </button>
                      );
                    })}
                  </div>
                </div>

                <div className="grid gap-3 lg:grid-cols-2">
                  {/* Model autocomplete */}
                  <div className="space-y-2">
                    <Label htmlFor="agents-model">Model</Label>
                    <AutocompleteInput
                      id="agents-model"
                      value={activeAgentConfig.ai?.model ?? ''}
                      options={(
                        MODEL_SUGGESTIONS[activeAgentConfig.ai?.provider ?? 'claude'] ?? []
                      ).map((m) => ({ value: m }))}
                      placeholder="claude-sonnet-4-5 / gemini-2.0-flash / gpt-4o"
                      noResultsText="Type a custom model name."
                      onValueChange={(value) =>
                        updateActiveAgentConfig({
                          ...activeAgentConfig,
                          ai: {
                            ...normalizeAiConfig(activeAgentConfig.ai),
                            model: value || null,
                          },
                        })
                      }
                    />
                  </div>

                  {/* CLI path */}
                  <div className="space-y-2">
                    <Label htmlFor="agents-cli-path">CLI Path Override</Label>
                    <Input
                      id="agents-cli-path"
                      value={activeAgentConfig.ai?.cli_path ?? ''}
                      onChange={(event) =>
                        updateActiveAgentConfig({
                          ...activeAgentConfig,
                          ai: {
                            ...normalizeAiConfig(activeAgentConfig.ai),
                            cli_path: event.target.value || null,
                          },
                        })
                      }
                      placeholder="Leave blank to use PATH"
                    />
                  </div>
                </div>

                <div className="flex items-center gap-3">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={testStatus === 'loading' || !hasActiveProject}
                    onClick={() => void handleTestProvider()}
                  >
                    <Zap className="size-3.5" />
                    {testStatus === 'loading' ? 'Testing…' : 'Test Agent'}
                  </Button>
                  {testStatus === 'ok' && (
                    <span className="text-xs text-emerald-500">Agent working ✓</span>
                  )}
                  {testStatus === 'error' && (
                    <span className="text-xs text-destructive">Failed — check binary/model/path</span>
                  )}
                  {agentError && (
                    <span className="text-xs text-destructive truncate max-w-[280px]" title={agentError}>{agentError}</span>
                  )}
                </div>
                {!hasActiveProject && (
                  <p className="text-muted-foreground text-xs">Open a project to run provider tests.</p>
                )}
              </CardContent>
            </Card>

            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-amber-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-amber-500/20 bg-amber-500/10">
                  <Layers className="size-3.5 text-amber-500" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">Modes</h3>
                  <p className="text-[11px] text-muted-foreground">Modes define explicit capability boundaries.</p>
                </div>
              </div>
              <CardContent className="space-y-3 !pt-5">
                <div className="space-y-2">
                  <Label>Active Mode</Label>
                  <Select
                    value={activeAgentConfig.active_mode ?? DEFAULT_MODE_VALUE}
                    onValueChange={handleSetActiveMode}
                  >
                    <SelectTrigger className="w-full sm:w-72">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={DEFAULT_MODE_VALUE}>Default (all capabilities)</SelectItem>
                      {(activeAgentConfig.modes ?? []).map((mode) => (
                        <SelectItem key={mode.id} value={mode.id}>
                          {mode.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {(activeAgentConfig.modes ?? []).length > 0 && (
                  <>
                    <Separator />
                    <div className="space-y-2">
                      {(activeAgentConfig.modes ?? []).map((mode) => (
                        <div
                          key={mode.id}
                          className="flex items-center justify-between rounded-md border px-3 py-2"
                        >
                          <div>
                            <p className="text-sm font-medium">{mode.name}</p>
                            <p className="text-muted-foreground text-xs font-mono">{mode.id}</p>
                          </div>
                          <Button variant="ghost" size="xs" onClick={() => handleRemoveMode(mode.id)}>
                            <Trash2 className="size-3.5" />
                          </Button>
                        </div>
                      ))}
                    </div>
                  </>
                )}

                <Separator />
                <div className="grid gap-2 sm:grid-cols-[1fr_1fr_auto]">
                  <Input
                    value={newMode.id}
                    onChange={(e) => setNewMode({ ...newMode, id: e.target.value })}
                    placeholder="mode-id"
                  />
                  <Input
                    value={newMode.name}
                    onChange={(e) => setNewMode({ ...newMode, name: e.target.value })}
                    placeholder="Display Name"
                  />
                  <Button onClick={handleAddMode} disabled={!newMode.id.trim() || !newMode.name.trim()}>
                    <Plus className="size-3.5" />
                    Add Mode
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        )}

        {initialSection === 'mcp' && (
          <div className="grid gap-4">
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-violet-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-violet-500/20 bg-violet-500/10">
                  <Package className="size-3.5 text-violet-500" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">MCP Servers</h3>
                  <p className="text-[11px] text-muted-foreground">JSON snippet editor for MCP server configuration.</p>
                </div>
              </div>
              <CardContent className="space-y-3 !pt-5">
                <div className="rounded-md border bg-card/50">
                  <div className="text-muted-foreground border-b px-3 py-2 text-[11px] font-medium uppercase tracking-wide">
                    JSON
                  </div>
                  <Textarea
                    value={mcpSnippet}
                    onChange={(event) => handleMcpSnippetChange(event.target.value)}
                    rows={16}
                    className="min-h-[340px] resize-y border-0 font-mono text-xs leading-6 shadow-none focus-visible:ring-0"
                    placeholder={'[\n  {\n    "id": "ship",\n    "name": "Shipwright",\n    "command": "ship-mcp",\n    "args": []\n  }\n]'}
                  />
                </div>
                {mcpSnippetError ? (
                  <Alert variant="destructive">
                    <AlertDescription>{mcpSnippetError}</AlertDescription>
                  </Alert>
                ) : (
                  <p className="text-muted-foreground text-xs">
                    Parsed {(activeAgentConfig.mcp_servers ?? []).length} server
                    {(activeAgentConfig.mcp_servers ?? []).length === 1 ? '' : 's'}.
                  </p>
                )}
              </CardContent>
            </Card>

            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-emerald-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-emerald-500/20 bg-emerald-500/10">
                  <Upload className="size-3.5 text-emerald-500" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">Sync to AI Clients</h3>
                  <p className="text-[11px] text-muted-foreground">Export MCP registry and agent docs to client configs.</p>
                </div>
              </div>
              <CardContent className="space-y-3 !pt-5">
                {agentError && (
                  <Alert variant="destructive">
                    <AlertDescription>{agentError}</AlertDescription>
                  </Alert>
                )}
                <div className="flex flex-wrap gap-2">
                  {(['claude', 'codex', 'gemini'] as const).map((target) => (
                    <Button
                      key={target}
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={exportStatus[target] === 'loading' || !hasActiveProject}
                      onClick={() => void handleExport(target)}
                    >
                      <Upload className="size-3.5" />
                      {exportStatus[target] === 'loading'
                        ? 'Syncing…'
                        : exportStatus[target] === 'ok'
                          ? `Synced to ${target} ✓`
                          : `Sync to ${target}`}
                    </Button>
                  ))}
                </div>
                {!hasActiveProject && (
                  <p className="text-muted-foreground text-xs">Open a project to export client config files.</p>
                )}
              </CardContent>
            </Card>
          </div>
        )}

        {(initialSection === 'skills' || initialSection === 'rules') && activeDocKind && (
          <div className="grid gap-4 xl:grid-cols-[300px_minmax(0,1fr)]">
            <Card size="sm" className="xl:h-[640px] overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-cyan-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-cyan-500/20 bg-cyan-500/10">
                  <ScrollText className="size-3.5 text-cyan-500" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">{initialSection === 'skills' ? 'Skill Docs' : 'Rule Docs'}</h3>
                  <p className="text-[11px] text-muted-foreground">Markdown file list (local stub until API integration).</p>
                </div>
              </div>
              <CardContent className="space-y-2 !pt-5">
                <Button variant="outline" size="sm" className="w-full" onClick={() => handleCreateDoc(activeDocKind)}>
                  <Plus className="size-3.5" />
                  New {initialSection === 'skills' ? 'Skill' : 'Rule'}
                </Button>
                <div className="max-h-[500px] space-y-1 overflow-auto pr-1">
                  {activeDocs.map((doc) => {
                    const selected = activeDoc?.id === doc.id;
                    return (
                      <button
                        key={doc.id}
                        type="button"
                        className={`w-full rounded-md border px-2.5 py-2 text-left transition-colors ${selected ? 'border-primary/40 bg-primary/10' : 'hover:bg-muted/50'
                          }`}
                        onClick={() =>
                          setSelectedDocIds((current: Record<ScopeKey, Record<MarkdownDocKind, string | null>>) => ({
                            ...current,
                            [agentScope]: {
                              ...current[agentScope],
                              [activeDocKind]: doc.id,
                            },
                          }))
                        }
                      >
                        <p className="truncate text-sm font-medium">{doc.title || 'Untitled'}</p>
                        <p className="text-muted-foreground text-xs">{formatUpdated(doc.updated)}</p>
                      </button>
                    );
                  })}
                </div>
              </CardContent>
            </Card>

            <Card size="sm" className="xl:h-[640px] overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-indigo-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-indigo-500/20 bg-indigo-500/10">
                  <PenLine className="size-3.5 text-indigo-500" />
                </div>
                <div className="flex-1">
                  <h3 className="text-sm font-semibold">{initialSection === 'skills' ? 'Skill Editor' : 'Rules Editor'}</h3>
                  <p className="text-[11px] text-muted-foreground">Markdown editor for selected {initialSection === 'skills' ? 'skill' : 'rule'}.</p>
                </div>
                {activeDoc && (
                  <Button
                    variant="ghost"
                    size="xs"
                    className="text-destructive hover:bg-destructive/10 hover:text-destructive"
                    onClick={() => handleDeleteDoc(activeDocKind, activeDoc.id)}
                  >
                    <Trash2 className="size-3.5 mr-1" />
                    Delete
                  </Button>
                )}
              </div>
              <CardContent className="space-y-3 !pt-5">
                {!activeDoc ? (
                  <div className="flex h-[400px] flex-col items-center justify-center gap-2 text-center">
                    <ScrollText className="size-8 text-muted-foreground opacity-30" />
                    <p className="text-muted-foreground text-sm">Select or create a document to start editing.</p>
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
        )}

        {initialSection === 'permissions' && (
          <div className="grid gap-4 lg:grid-cols-[1fr_300px]">
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-rose-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-rose-500/20 bg-rose-500/10">
                  <Shield className="size-3.5 text-rose-500" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">Capabilities</h3>
                  <p className="text-[11px] text-muted-foreground">Structured policy for agent tool usage and access.</p>
                </div>
              </div>
              <CardContent className="space-y-6 !pt-5">
                {!permissions ? (
                  <p className="text-muted-foreground py-10 text-center text-sm">Loading permissions...</p>
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
                          </div>
                          <p className="text-muted-foreground text-xs">Explicity permitted community tools.</p>
                          <div className="space-y-2">
                            {(permissions.tools?.allow || []).map((p, idx) => (
                              <div key={idx} className="flex items-center gap-2">
                                <Input
                                  value={p || ''}
                                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                                    const next = [...(permissions.tools?.allow || [])];
                                    next[idx] = e.target.value;
                                    savePermissionsMut.mutate({ ...permissions, tools: { ...permissions.tools, allow: next, deny: permissions.tools?.deny || [] } });
                                  }}
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
                              <Plus className="size-3.5 mr-1" /> Add Permission
                            </Button>
                          </div>
                        </div>

                        <div className="space-y-3">
                          <div className="flex items-center gap-2">
                            <ShieldAlert className="size-4 text-destructive" />
                            <Label>Deny List</Label>
                          </div>
                          <p className="text-muted-foreground text-xs">Blocked community tools.</p>
                          <div className="space-y-2">
                            {(permissions.tools?.deny || []).map((p, idx) => (
                              <div key={idx} className="flex items-center gap-2">
                                <Input
                                  value={p || ''}
                                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                                    const next = [...(permissions.tools?.deny || [])];
                                    next[idx] = e.target.value;
                                    savePermissionsMut.mutate({ ...permissions, tools: { ...permissions.tools, deny: next, allow: permissions.tools?.allow || ["*"] } });
                                  }}
                                />
                                <Button
                                  variant="ghost"
                                  size="xs"
                                  onClick={() => {
                                    const next = (permissions.tools?.deny || []).filter((_, i) => i !== idx);
                                    savePermissionsMut.mutate({ ...permissions, tools: { ...permissions.tools, deny: next, allow: permissions.tools?.allow || ["*"] } });
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
                                  tools: { ...permissions.tools, deny: [...(permissions.tools?.deny || []), ''], allow: permissions.tools?.allow || ["*"] },
                                });
                              }}
                            >
                              <Plus className="size-3.5 mr-1" /> Add Restriction
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
                          </div>
                          <div className="space-y-2">
                            {(permissions.filesystem?.allow || []).map((p, idx) => (
                              <div key={idx} className="flex items-center gap-2">
                                <Input
                                  value={p || ''}
                                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                                    const next = [...(permissions.filesystem?.allow || [])];
                                    next[idx] = e.target.value;
                                    savePermissionsMut.mutate({ ...permissions, filesystem: { ...permissions.filesystem, allow: next, deny: permissions.filesystem?.deny || [] } });
                                  }}
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
                              <Plus className="size-3.5 mr-1" /> Add Path
                            </Button>
                          </div>
                        </div>

                        <div className="space-y-3">
                          <div className="flex items-center gap-2">
                            <LockIcon className="size-4 text-destructive" />
                            <Label>Block List</Label>
                          </div>
                          <div className="space-y-2">
                            {(permissions.filesystem?.deny || []).map((p, idx) => (
                              <div key={idx} className="flex items-center gap-2">
                                <Input
                                  value={p || ''}
                                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                                    const next = [...(permissions.filesystem?.deny || [])];
                                    next[idx] = e.target.value;
                                    savePermissionsMut.mutate({ ...permissions, filesystem: { ...permissions.filesystem, deny: next, allow: permissions.filesystem?.allow || [] } });
                                  }}
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
                              <Plus className="size-3.5 mr-1" /> Add Exclusion
                            </Button>
                          </div>
                        </div>
                      </div>
                    </TabsContent>

                    <TabsContent value="limits" className="space-y-6">
                      <div className="grid gap-6 md:grid-cols-2">
                        <div className="space-y-3">
                          <Label>Max Cost per Session (USD)</Label>
                          <Input
                            type="number"
                            step="0.01"
                            value={permissions.agent?.max_cost_per_session ?? ''}
                            onChange={(e) => savePermissionsMut.mutate({ ...permissions, agent: { ...permissions.agent, max_cost_per_session: parseFloat(e.target.value) || null } })}
                            placeholder="Unlimited"
                          />
                        </div>
                        <div className="space-y-3">
                          <Label>Max Turns per Session</Label>
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

            <Card size="sm" className="bg-muted/10 overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-slate-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-slate-500/20 bg-slate-500/10">
                  <Info className="size-3.5 text-slate-500" />
                </div>
                <h3 className="text-sm font-semibold">Information</h3>
              </div>
              <CardContent className="space-y-4 text-xs leading-relaxed !pt-5">
                <p>
                  Permissions define the security sandbox for AI agents operating in this project.
                </p>
                <p className="text-muted-foreground">
                  The <span className="text-emerald-500 font-medium">Allow List</span> uses glob patterns to permit
                  specific tools or file access.
                </p>
                <p className="text-muted-foreground">
                  The <span className="text-destructive font-medium">Deny List</span> takes precedence and blocks
                  matching operations even if allowed elsewhere.
                </p>
                <div className="rounded-md border bg-card p-3">
                  <p className="font-medium mb-1">Security Guardrails</p>
                  <p className="text-muted-foreground leading-normal">
                    These rules are enforced by the core runtime. Even if an AI suggests a change, it will be blocked if it violates these policies.
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
