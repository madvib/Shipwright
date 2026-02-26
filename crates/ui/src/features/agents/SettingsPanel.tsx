import { useEffect, useMemo, useRef, useState } from 'react';
import { ArrowLeft, Plus, Trash2, Upload } from 'lucide-react';
import { GitConfig, McpServerConfig, ModeConfig, ProjectConfig, StatusConfig } from '@/bindings';
import {
  exportAgentConfigCmd,
  generateIssueDescriptionCmd,
} from '@/lib/platform/tauri/commands';
import { Config, DEFAULT_STATUSES } from '@/lib/workspace-ui';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Textarea } from '@/components/ui/textarea';

type SettingsTab = 'global' | 'project' | 'agents' | 'modules';
type SettingsPanelMode = 'full' | 'settings-only' | 'agents-only';

interface SettingsPanelProps {
  config: Config;
  projectConfig: ProjectConfig | null;
  globalAgentConfig: ProjectConfig | null;
  onThemePreview: (theme?: string) => void;
  onSave: (config: Config) => void;
  onSaveProject: (config: ProjectConfig) => void;
  onSaveGlobalAgentConfig: (config: ProjectConfig) => void;
  onBack: () => void;
  onOpenAgentsModule?: () => void;
  initialTab?: SettingsTab;
  panelMode?: SettingsPanelMode;
}

const GIT_CATEGORIES = [
  'issues',
  'releases',
  'features',
  'adrs',
  'specs',
  'config.toml',
  'templates',
  'log.md',
  'events.ndjson',
];
const AI_PROVIDERS = [
  { id: 'claude', label: 'Claude (claude)' },
  { id: 'gemini', label: 'Gemini (gemini)' },
  { id: 'codex', label: 'Codex (codex)' },
];
const SCOPE_OPTIONS = ['global', 'project', 'mode'] as const;
const EMPTY_AGENT_LAYER = {
  skills: [],
  prompts: [],
  context: [],
  rules: [],
};
const DEFAULT_MODE_VALUE = 'default';

type NormalizedProjectConfig = ProjectConfig & {
  statuses: StatusConfig[];
  git: GitConfig;
  ai: NonNullable<ProjectConfig['ai']>;
  modes: ModeConfig[];
  mcp_servers: McpServerConfig[];
};

function parseLines(value: string): string[] {
  return value
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

function joinLines(values: string[] | undefined): string {
  return (values ?? []).join('\n');
}

function normalizeAiConfig(ai: ProjectConfig['ai']) {
  return {
    provider: ai?.provider ?? 'claude',
    model: ai?.model ?? null,
    cli_path: ai?.cli_path ?? null,
  };
}

function normalizeProjectConfig(config: ProjectConfig | null): NormalizedProjectConfig {
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
        config?.git?.commit ?? ['releases', 'features', 'adrs', 'specs', 'config.toml', 'templates'],
    },
    ai: normalizeAiConfig(config?.ai ?? null),
    modes: config?.modes ?? [],
    mcp_servers: config?.mcp_servers ?? [],
    active_mode: config?.active_mode ?? null,
    agent: {
      ...EMPTY_AGENT_LAYER,
      ...(config?.agent ?? {}),
    },
  };
}

const EMPTY_SERVER: {
  id: string;
  name: string;
  command: string;
  args_raw: string;
  scope: 'global' | 'project' | 'mode';
} = {
  id: '',
  name: '',
  command: '',
  args_raw: '',
  scope: 'global',
};

const EMPTY_MODE: ModeConfig = {
  id: '',
  name: '',
  description: null,
  active_tools: [],
  mcp_servers: [],
};

export default function SettingsPanel({
  config,
  projectConfig,
  globalAgentConfig,
  onThemePreview,
  onSave,
  onSaveProject,
  onSaveGlobalAgentConfig,
  onBack,
  onOpenAgentsModule,
  initialTab = 'global',
  panelMode = 'full',
}: SettingsPanelProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>(() =>
    panelMode === 'agents-only' ? 'agents' : initialTab
  );
  const [local, setLocal] = useState<Config>(config);
  const [localProject, setLocalProject] = useState<NormalizedProjectConfig>(normalizeProjectConfig(projectConfig));
  const [localGlobalAgent, setLocalGlobalAgent] = useState<NormalizedProjectConfig>(
    normalizeProjectConfig(globalAgentConfig)
  );
  const [agentScope, setAgentScope] = useState<'project' | 'global'>(
    projectConfig ? 'project' : 'global'
  );
  const [newStatus, setNewStatus] = useState<StatusConfig>({ id: '', name: '', color: 'gray' });
  const [newServer, setNewServer] = useState(EMPTY_SERVER);
  const [newMode, setNewMode] = useState<ModeConfig>(EMPTY_MODE);
  const [exportStatus, setExportStatus] = useState<Record<string, 'idle' | 'loading' | 'ok' | 'error'>>({});
  const [testStatus, setTestStatus] = useState<'idle' | 'loading' | 'ok' | 'error'>('idle');
  const [agentError, setAgentError] = useState<string | null>(null);
  const initialThemeRef = useRef<string | undefined>(config.theme);

  useEffect(() => {
    setLocal(config);
    initialThemeRef.current = config.theme;
  }, [config]);

  useEffect(() => {
    setLocalProject(normalizeProjectConfig(projectConfig));
  }, [projectConfig]);

  useEffect(() => {
    setLocalGlobalAgent(normalizeProjectConfig(globalAgentConfig));
  }, [globalAgentConfig]);

  useEffect(() => {
    if (panelMode !== 'agents-only' && !projectConfig && activeTab === 'project') {
      setActiveTab('global');
    }
  }, [activeTab, panelMode, projectConfig]);

  useEffect(() => {
    if (panelMode === 'agents-only') {
      setActiveTab('agents');
      return;
    }
    const nextTab =
      panelMode === 'settings-only' && (initialTab === 'agents' || initialTab === 'modules')
        ? 'global'
        : initialTab;
    setActiveTab(nextTab);
  }, [initialTab, panelMode]);

  useEffect(() => {
    if (!projectConfig) {
      setAgentScope('global');
    }
  }, [projectConfig]);

  const availableStatuses = useMemo(
    () => (localProject.statuses.length > 0 ? localProject.statuses : DEFAULT_STATUSES),
    [localProject.statuses]
  );
  const hasActiveProject = !!projectConfig;
  const activeAgentConfig = agentScope === 'project' ? localProject : localGlobalAgent;

  const updateActiveAgentConfig = (next: NormalizedProjectConfig) => {
    if (agentScope === 'project') {
      setLocalProject(next);
      return;
    }
    setLocalGlobalAgent(next);
  };

  const updateStatus = (index: number, patch: Partial<StatusConfig>) => {
    setLocalProject((current) => ({
      ...current,
      statuses: current.statuses.map((status, i) => (i === index ? { ...status, ...patch } : status)),
    }));
  };

  const removeStatus = (index: number) => {
    setLocalProject((current) => ({
      ...current,
      statuses: current.statuses.filter((_, i) => i !== index),
    }));
  };

  const toggleGitCategory = (category: string) => {
    setLocalProject((current) => {
      const git = {
        ignore: [...(current.git?.ignore ?? [])],
        commit: [...(current.git?.commit ?? [])],
      };
      const isCommitted = git.commit.includes(category);
      if (isCommitted) {
        git.commit = git.commit.filter((item) => item !== category);
        if (!git.ignore.includes(category)) git.ignore.push(category);
      } else {
        git.ignore = git.ignore.filter((item) => item !== category);
        git.commit.push(category);
      }
      return { ...current, git };
    });
  };

  const handleAddStatus = () => {
    const id = newStatus.id.trim();
    const name = newStatus.name.trim();
    if (!id || !name) return;
    setLocalProject((current) => ({
      ...current,
      statuses: [...current.statuses, { id, name, color: (newStatus.color ?? '').trim() || 'gray' }],
    }));
    setNewStatus({ id: '', name: '', color: 'gray' });
  };

  const handleAddServer = () => {
    const id = newServer.id.trim();
    const command = newServer.command.trim();
    if (!id || !command) return;
    const server: McpServerConfig = {
      id,
      name: newServer.name.trim() || id,
      command,
      args: newServer.args_raw.trim()
        ? newServer.args_raw.trim().split(/\s+/).filter(Boolean)
        : [],
      env: {},
      scope: newServer.scope,
      server_type: 'stdio',
      url: null,
      timeout_secs: null,
      disabled: false,
    };
    updateActiveAgentConfig({
      ...activeAgentConfig,
      mcp_servers: [...(activeAgentConfig.mcp_servers ?? []), server],
    });
    setNewServer(EMPTY_SERVER);
  };

  const handleRemoveServer = (id: string) => {
    updateActiveAgentConfig({
      ...activeAgentConfig,
      mcp_servers: (activeAgentConfig.mcp_servers ?? []).filter((s) => s.id !== id),
    });
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
      modes: (activeAgentConfig.modes ?? []).filter((m) => m.id !== id),
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
      await generateIssueDescriptionCmd('test task');
      setTestStatus('ok');
    } catch (err) {
      setTestStatus('error');
      setAgentError(String(err));
    }
  };

  const handleExport = async (target: 'claude' | 'codex' | 'gemini') => {
    setExportStatus((prev) => ({ ...prev, [target]: 'loading' }));
    setAgentError(null);
    try {
      await exportAgentConfigCmd(target);
      setExportStatus((prev) => ({ ...prev, [target]: 'ok' }));
    } catch (err) {
      setExportStatus((prev) => ({ ...prev, [target]: 'error' }));
      setAgentError(String(err));
    }
  };

  const handleBack = () => {
    onThemePreview(initialThemeRef.current);
    onBack();
  };

  const agentsOnly = panelMode === 'agents-only';
  const settingsOnly = panelMode === 'settings-only';

  return (
    <div className="mx-auto flex w-full max-w-5xl flex-col gap-5 p-5 md:p-8">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Button variant="ghost" onClick={handleBack}>
            <ArrowLeft className="size-4" />
            Back
          </Button>
          <div>
            <h1 className="text-xl font-semibold tracking-tight md:text-2xl">
              {agentsOnly ? 'Agents' : 'Settings'}
            </h1>
            <p className="text-muted-foreground text-sm">
              {agentsOnly ? 'Agent config, modes, MCP, and client sync' : 'Global and project configuration'}
            </p>
          </div>
        </div>
        <Badge variant="outline">Alpha</Badge>
      </header>

      <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as typeof activeTab)}>
        {!agentsOnly && (
          <TabsList className="w-full justify-start">
            <TabsTrigger value="global">Global</TabsTrigger>
            <TabsTrigger value="project" disabled={!projectConfig}>
              Project
            </TabsTrigger>
            {settingsOnly && (
              <TabsTrigger value="modules">Modules</TabsTrigger>
            )}
            {!settingsOnly && (
              <TabsTrigger value="agents">
                Agents
              </TabsTrigger>
            )}
          </TabsList>
        )}

        {/* ── Global tab ──────────────────────────────────────────────────── */}
        <TabsContent value="global">
          <div className="grid gap-4 lg:grid-cols-2">
            <Card size="sm">
              <CardHeader>
                <CardTitle>User</CardTitle>
                <CardDescription>Name and email used for authorship metadata.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="space-y-2">
                  <Label htmlFor="settings-author">Name</Label>
                  <Input
                    id="settings-author"
                    value={local.author ?? ''}
                    onChange={(event) => setLocal({ ...local, author: event.target.value })}
                    placeholder="Your name"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="settings-email">Email</Label>
                  <Input
                    id="settings-email"
                    value={local.email ?? ''}
                    onChange={(event) => setLocal({ ...local, email: event.target.value })}
                    placeholder="you@example.com"
                  />
                </div>
              </CardContent>
            </Card>

            <Card size="sm">
              <CardHeader>
                <CardTitle>MCP</CardTitle>
                <CardDescription>Local bridge for AI clients and tooling.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="space-y-2">
                  <Label htmlFor="settings-mcp-port">Port</Label>
                  <Input
                    id="settings-mcp-port"
                    type="number"
                    value={local.mcp_port ?? 7700}
                    onChange={(event) => {
                      const nextPort = Number.parseInt(event.target.value, 10);
                      setLocal({
                        ...local,
                        mcp_port: Number.isFinite(nextPort) ? nextPort : 7700,
                      });
                    }}
                  />
                </div>
                <div className="flex items-center justify-between rounded-lg border px-3 py-2">
                  <div className="space-y-0.5">
                    <p className="text-sm font-medium">Enable MCP</p>
                    <p className="text-muted-foreground text-xs">Allow AI clients to connect to Ship context.</p>
                  </div>
                  <Switch
                    checked={local.mcp_enabled !== false}
                    onCheckedChange={(checked) => setLocal({ ...local, mcp_enabled: checked })}
                  />
                </div>
              </CardContent>
            </Card>

            <Card size="sm" className="lg:col-span-2">
              <CardHeader>
                <CardTitle>Appearance & Defaults</CardTitle>
                <CardDescription>Theme and creation defaults for new issues.</CardDescription>
              </CardHeader>
              <CardContent className="grid gap-3 md:grid-cols-3">
                <div className="space-y-2">
                  <Label>Theme</Label>
                  <Select
                    value={local.theme ?? 'dark'}
                    onValueChange={(value) => {
                      const theme = value ?? undefined;
                      setLocal({ ...local, theme });
                      onThemePreview(theme);
                    }}
                  >
                    <SelectTrigger className="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="dark">Dark</SelectItem>
                      <SelectItem value="light">Light</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label>Default Issue Status</Label>
                  <Select
                    value={local.default_status ?? availableStatuses[0]?.id ?? 'backlog'}
                    onValueChange={(value) =>
                      setLocal({ ...local, default_status: value ?? undefined })
                    }
                  >
                    <SelectTrigger className="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {availableStatuses.map((status) => (
                        <SelectItem key={status.id} value={status.id}>
                          {status.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="settings-editor">Editor</Label>
                  <Input
                    id="settings-editor"
                    value={local.editor ?? 'code'}
                    onChange={(event) => setLocal({ ...local, editor: event.target.value })}
                    placeholder="code"
                  />
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {settingsOnly && (
          <TabsContent value="modules">
            <Card size="sm">
              <CardHeader>
                <CardTitle>Module Settings</CardTitle>
                <CardDescription>Open module-specific settings pages.</CardDescription>
              </CardHeader>
              <CardContent>
                <Button type="button" variant="outline" onClick={onOpenAgentsModule}>
                  Open Agents Module
                </Button>
              </CardContent>
            </Card>
          </TabsContent>
        )}

        {/* ── Project tab ─────────────────────────────────────────────────── */}
        <TabsContent value="project">
          {!projectConfig ? (
            <Card size="sm">
              <CardHeader>
                <CardTitle>Select a project first</CardTitle>
                <CardDescription>Project settings become available after opening or creating a project.</CardDescription>
              </CardHeader>
            </Card>
          ) : (
            <div className="grid gap-4">
              <Card size="sm">
                <CardHeader>
                  <CardTitle>Project</CardTitle>
                  <CardDescription>Metadata stored in `.ship/config.toml`.</CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div className="space-y-2">
                    <Label htmlFor="settings-project-name">Project Name</Label>
                    <Input
                      id="settings-project-name"
                      value={localProject.name ?? ''}
                      onChange={(event) =>
                        setLocalProject({ ...localProject, name: event.target.value || null })
                      }
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="settings-project-description">Description</Label>
                    <Textarea
                      id="settings-project-description"
                      rows={3}
                      value={localProject.description ?? ''}
                      onChange={(event) =>
                        setLocalProject({ ...localProject, description: event.target.value || null })
                      }
                    />
                  </div>
                </CardContent>
              </Card>

              <Card size="sm">
                <CardHeader>
                  <CardTitle>Statuses</CardTitle>
                  <CardDescription>Customize issue workflow columns for this project.</CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div className="hidden grid-cols-[1fr_1.2fr_1fr_auto] gap-2 px-1 text-xs text-muted-foreground md:grid">
                    <span>ID</span>
                    <span>Name</span>
                    <span>Color</span>
                    <span />
                  </div>
                  {localProject.statuses.map((status, index) => (
                    <div key={`${status.id}-${index}`} className="grid gap-2 md:grid-cols-[1fr_1.2fr_1fr_auto]">
                      <Input
                        value={status.id}
                        onChange={(event) => updateStatus(index, { id: event.target.value })}
                        placeholder="id"
                      />
                      <Input
                        value={status.name}
                        onChange={(event) => updateStatus(index, { name: event.target.value })}
                        placeholder="name"
                      />
                      <Input
                        value={status.color}
                        onChange={(event) => updateStatus(index, { color: event.target.value })}
                        placeholder="color"
                      />
                      <Button variant="destructive" size="xs" onClick={() => removeStatus(index)}>
                        <Trash2 className="size-3.5" />
                        Delete
                      </Button>
                    </div>
                  ))}
                  <Separator />
                  <div className="grid gap-2 md:grid-cols-[1fr_1.2fr_1fr_auto]">
                    <Input
                      value={newStatus.id}
                      onChange={(event) => setNewStatus({ ...newStatus, id: event.target.value })}
                      placeholder="new-id"
                    />
                    <Input
                      value={newStatus.name}
                      onChange={(event) => setNewStatus({ ...newStatus, name: event.target.value })}
                      placeholder="Display Name"
                    />
                    <Input
                      value={newStatus.color}
                      onChange={(event) => setNewStatus({ ...newStatus, color: event.target.value })}
                      placeholder="color"
                    />
                    <Button onClick={handleAddStatus}>
                      <Plus className="size-3.5" />
                      Add
                    </Button>
                  </div>
                </CardContent>
              </Card>

              <Card size="sm">
                <CardHeader>
                  <CardTitle>Git Commit Categories</CardTitle>
                  <CardDescription>Choose which docs are staged by default for project commits.</CardDescription>
                </CardHeader>
                <CardContent className="grid gap-2 sm:grid-cols-2">
                  {GIT_CATEGORIES.map((category) => {
                    const committed = localProject.git?.commit?.includes(category) ?? false;
                    return (
                      <label key={category} className="flex items-center gap-2 rounded-md border px-3 py-2">
                        <Checkbox checked={committed} onCheckedChange={() => toggleGitCategory(category)} />
                        <span className="flex-1 text-sm">{category}</span>
                        <Badge variant={committed ? 'default' : 'outline'} className="text-[10px]">
                          {committed ? 'Commit' : 'Ignore'}
                        </Badge>
                      </label>
                    );
                  })}
                </CardContent>
              </Card>
            </div>
          )}
        </TabsContent>

        {/* ── Agents tab ──────────────────────────────────────────────────── */}
        {!settingsOnly && (
        <TabsContent value="agents">
          <div className="grid gap-4">
            <Card size="sm">
              <CardHeader>
                <CardTitle>Scope</CardTitle>
                <CardDescription>
                  Configure agent defaults globally, or override per project.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-2">
                <Label>Agent Config Scope</Label>
                <Select
                  value={agentScope}
                  onValueChange={(value) => setAgentScope((value as 'project' | 'global') ?? 'global')}
                >
                  <SelectTrigger className="w-full sm:w-72">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="global">Global (~/.ship/config.toml)</SelectItem>
                    <SelectItem value="project" disabled={!projectConfig}>
                      Project (.ship/config.toml)
                    </SelectItem>
                  </SelectContent>
                </Select>
                {agentScope === 'project' && !projectConfig && (
                  <p className="text-xs text-destructive">Open a project to edit project-scoped agent config.</p>
                )}
              </CardContent>
            </Card>

            <Card size="sm">
              <CardHeader>
                <CardTitle>AI Provider</CardTitle>
                <CardDescription>
                  Pass-through CLI provider used for generation features in the UI.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="grid gap-3 sm:grid-cols-2">
                  <div className="space-y-2">
                    <Label>Provider</Label>
                    <Select
                      value={activeAgentConfig.ai?.provider ?? 'claude'}
                      onValueChange={(value) =>
                        updateActiveAgentConfig({
                          ...activeAgentConfig,
                          ai: { ...normalizeAiConfig(activeAgentConfig.ai), provider: value },
                        })
                      }
                    >
                      <SelectTrigger className="w-full">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {AI_PROVIDERS.map((p) => (
                          <SelectItem key={p.id} value={p.id}>
                            {p.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="settings-ai-cli">Binary Path Override</Label>
                    <Input
                      id="settings-ai-cli"
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
                    {testStatus === 'loading' ? 'Testing…' : 'Test Provider'}
                  </Button>
                  {testStatus === 'ok' && (
                    <span className="text-xs text-emerald-500">Provider working ✓</span>
                  )}
                  {testStatus === 'error' && (
                    <span className="text-xs text-destructive">Failed — check binary path</span>
                  )}
                </div>
                {!hasActiveProject && (
                  <p className="text-muted-foreground text-xs">
                    Open a project to run provider tests.
                  </p>
                )}
              </CardContent>
            </Card>

            <Card size="sm">
              <CardHeader>
                <CardTitle>Agent Context Layer</CardTitle>
                <CardDescription>
                  One place for skills, prompts, context, and rules.
                </CardDescription>
              </CardHeader>
              <CardContent className="grid gap-3 lg:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="settings-agent-skills">Skills (one per line)</Label>
                  <Textarea
                    id="settings-agent-skills"
                    rows={6}
                    value={joinLines(activeAgentConfig.agent?.skills)}
                    onChange={(event) =>
                      updateActiveAgentConfig({
                        ...activeAgentConfig,
                        agent: {
                          ...(activeAgentConfig.agent ?? EMPTY_AGENT_LAYER),
                          skills: parseLines(event.target.value),
                        },
                      })
                    }
                    placeholder="backend-rust&#10;frontend-react&#10;qa-regression"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="settings-agent-prompts">Prompt Snippets (one per line)</Label>
                  <Textarea
                    id="settings-agent-prompts"
                    rows={6}
                    value={joinLines(activeAgentConfig.agent?.prompts)}
                    onChange={(event) =>
                      updateActiveAgentConfig({
                        ...activeAgentConfig,
                        agent: {
                          ...(activeAgentConfig.agent ?? EMPTY_AGENT_LAYER),
                          prompts: parseLines(event.target.value),
                        },
                      })
                    }
                    placeholder="Always produce patch-ready diffs&#10;Summarize risks before coding"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="settings-agent-context">Context Paths (one per line)</Label>
                  <Textarea
                    id="settings-agent-context"
                    rows={6}
                    value={joinLines(activeAgentConfig.agent?.context)}
                    onChange={(event) =>
                      updateActiveAgentConfig({
                        ...activeAgentConfig,
                        agent: {
                          ...(activeAgentConfig.agent ?? EMPTY_AGENT_LAYER),
                          context: parseLines(event.target.value),
                        },
                      })
                    }
                    placeholder="AGENTS.md&#10;specs/&#10;adrs/"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="settings-agent-rules">Rules (one per line)</Label>
                  <Textarea
                    id="settings-agent-rules"
                    rows={6}
                    value={joinLines(activeAgentConfig.agent?.rules)}
                    onChange={(event) =>
                      updateActiveAgentConfig({
                        ...activeAgentConfig,
                        agent: {
                          ...(activeAgentConfig.agent ?? EMPTY_AGENT_LAYER),
                          rules: parseLines(event.target.value),
                        },
                      })
                    }
                    placeholder="Never rewrite git history&#10;Prefer rg for code search"
                  />
                </div>
              </CardContent>
            </Card>

            <Card size="sm">
              <CardHeader>
                <CardTitle>Modes</CardTitle>
                <CardDescription>
                  Mode switching is capability control. Keep this central and explicit.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                {(activeAgentConfig.modes ?? []).length > 0 && (
                  <>
                    <div className="space-y-2">
                      <Label>Active Mode</Label>
                      <Select
                        value={activeAgentConfig.active_mode ?? DEFAULT_MODE_VALUE}
                        onValueChange={handleSetActiveMode}
                      >
                        <SelectTrigger className="w-full sm:w-64">
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
                          <Button
                            variant="ghost"
                            size="xs"
                            onClick={() => handleRemoveMode(mode.id)}
                          >
                            <Trash2 className="size-3.5" />
                          </Button>
                        </div>
                      ))}
                    </div>
                    <Separator />
                  </>
                )}
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

            <Card size="sm">
              <CardHeader>
                <CardTitle>MCP Server Registry</CardTitle>
                <CardDescription>
                  Registry for MCP tools used by this scope.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                {(activeAgentConfig.mcp_servers ?? []).length > 0 && (
                  <>
                    {(activeAgentConfig.mcp_servers ?? []).map((server) => (
                      <div
                        key={server.id}
                        className="flex items-start justify-between gap-2 rounded-md border px-3 py-2"
                      >
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2">
                            <p className="text-sm font-medium">{server.name}</p>
                            <Badge variant="outline" className="text-[10px]">
                              {server.scope}
                            </Badge>
                          </div>
                          <p className="text-muted-foreground truncate font-mono text-xs">
                            {server.command} {(server.args ?? []).join(' ')}
                          </p>
                        </div>
                        <Button
                          variant="ghost"
                          size="xs"
                          onClick={() => handleRemoveServer(server.id)}
                        >
                          <Trash2 className="size-3.5" />
                        </Button>
                      </div>
                    ))}
                    <Separator />
                  </>
                )}
                <div className="grid gap-2">
                  <div className="grid gap-2 sm:grid-cols-2">
                    <Input
                      value={newServer.id}
                      onChange={(e) => setNewServer({ ...newServer, id: e.target.value })}
                      placeholder="server-id"
                    />
                    <Input
                      value={newServer.name}
                      onChange={(e) => setNewServer({ ...newServer, name: e.target.value })}
                      placeholder="Display Name"
                    />
                  </div>
                  <div className="grid gap-2 sm:grid-cols-[1fr_1fr_auto_auto]">
                    <Input
                      value={newServer.command}
                      onChange={(e) => setNewServer({ ...newServer, command: e.target.value })}
                      placeholder="command (e.g. ship-mcp)"
                    />
                    <Input
                      value={newServer.args_raw}
                      onChange={(e) => setNewServer({ ...newServer, args_raw: e.target.value })}
                      placeholder="args (space-separated)"
                    />
                    <Select
                      value={newServer.scope}
                      onValueChange={(v) =>
                        setNewServer({
                          ...newServer,
                          scope: v as typeof newServer.scope,
                        })
                      }
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {SCOPE_OPTIONS.map((scope) => (
                          <SelectItem key={scope} value={scope}>
                            {scope}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <Button
                      onClick={handleAddServer}
                      disabled={!newServer.id.trim() || !newServer.command.trim()}
                    >
                      <Plus className="size-3.5" />
                      Add
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card size="sm">
              <CardHeader>
                <CardTitle>Sync to AI Clients</CardTitle>
                <CardDescription>
                  Export current scope MCP registry and agent layer docs to client configs.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                {agentError && (
                  <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                    {agentError}
                  </p>
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
                  <p className="text-muted-foreground text-xs">
                    Open a project to export client config files.
                  </p>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>
        )}
      </Tabs>

      <footer className="flex items-center justify-end gap-2 border-t pt-4">
        <Button variant="ghost" onClick={handleBack}>
          {agentsOnly ? 'Close' : 'Cancel'}
        </Button>
        {settingsOnly && activeTab === 'modules' ? (
          <Button type="button" variant="outline" onClick={onOpenAgentsModule}>
            Open Agents Module
          </Button>
        ) : agentsOnly ? (
          agentScope === 'global' ? (
            <Button onClick={() => onSaveGlobalAgentConfig(localGlobalAgent)}>
              Save Global Agent Config
            </Button>
          ) : (
            <Button onClick={() => onSaveProject(localProject)} disabled={!projectConfig}>
              Save Project Agent Config
            </Button>
          )
        ) : activeTab === 'global' ? (
          <Button onClick={() => onSave(local)}>Save Global Settings</Button>
        ) : activeTab === 'agents' && agentScope === 'global' ? (
          <Button onClick={() => onSaveGlobalAgentConfig(localGlobalAgent)}>
            Save Global Agent Config
          </Button>
        ) : (
          <Button onClick={() => onSaveProject(localProject)} disabled={!projectConfig}>
            Save Project Settings
          </Button>
        )}
      </footer>
    </div>
  );
}
