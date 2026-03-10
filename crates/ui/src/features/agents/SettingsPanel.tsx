import { useEffect, useMemo, useRef, useState } from 'react';
import { Settings, User as UserIcon, Palette, Globe2, GitBranch, Trash2, Upload, Sun, Moon, Cpu, Plus } from 'lucide-react';
import { GitConfig, McpServerConfig, ModeConfig, ProjectConfig, StatusConfig } from '@/bindings';
import {
  exportAgentConfigCmd,
  transformTextCmd,
} from '@/lib/platform/tauri/commands';
import { Config, DEFAULT_STATUSES } from '@/lib/workspace-ui';
import { Alert, AlertDescription } from '@ship/ui';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@ship/ui';
import { Checkbox } from '@ship/ui';
import { Input } from '@ship/ui';
import { Label } from '@ship/ui';
import { PageFrame, PageHeader } from '@ship/ui';
import { cn } from '@/lib/utils';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@ship/ui';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from '@ship/ui';
import { Separator } from '@ship/ui';
import { Switch } from '@ship/ui';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@ship/ui';
import { Textarea } from '@ship/ui';
import MarkdownEditor from '@/components/editor';
import AgentScopeCard from '@/features/agents/AgentScopeCard';

type SettingsTab = 'global' | 'project' | 'agents' | 'modules';
type SettingsPanelMode = 'full' | 'settings-only' | 'agents-only';

interface SettingsPanelProps {
  config: Config;
  projectConfig: ProjectConfig | null;
  globalAgentConfig: ProjectConfig | null;
  onThemePreview: (theme: 'light' | 'dark' | undefined) => void;
  onSave: (config: Config) => void;
  onSaveProject: (config: ProjectConfig) => void;
  onSaveGlobalAgentConfig: (config: ProjectConfig) => void;
  onOpenAgentsModule?: () => void;
  initialTab?: SettingsTab;
  panelMode?: SettingsPanelMode;
}

const GIT_CATEGORIES = [
  'releases',
  'features',
  'adrs',
  'specs',
  'vision',
  'ship.toml',
  'templates',
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
};
const DEFAULT_MODE_VALUE = 'default';

const STATUS_COLORS: { value: string; hex: string; label: string }[] = [
  { value: 'gray', hex: '#6b7280', label: 'Gray' },
  { value: 'slate', hex: '#64748b', label: 'Slate' },
  { value: 'red', hex: '#ef4444', label: 'Red' },
  { value: 'orange', hex: '#f97316', label: 'Orange' },
  { value: 'amber', hex: '#f59e0b', label: 'Amber' },
  { value: 'yellow', hex: '#eab308', label: 'Yellow' },
  { value: 'green', hex: '#22c55e', label: 'Green' },
  { value: 'teal', hex: '#14b8a6', label: 'Teal' },
  { value: 'cyan', hex: '#06b6d4', label: 'Cyan' },
  { value: 'blue', hex: '#3b82f6', label: 'Blue' },
  { value: 'violet', hex: '#8b5cf6', label: 'Violet' },
  { value: 'rose', hex: '#f43f5e', label: 'Rose' },
];

function StatusColorPicker({
  value,
  onChange,
}: {
  value: string;
  onChange: (color: string) => void;
}) {
  const current = STATUS_COLORS.find((c) => c.value === value);
  const currentHex = current?.hex ?? '#6b7280';

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        className="flex items-center gap-1.5 rounded-md border px-2 py-1.5 text-xs transition-colors hover:bg-accent/50"
      >
        <span
          className="size-4 rounded-full border border-border/50 shadow-sm"
          style={{ backgroundColor: currentHex }}
        />
        <span className="text-muted-foreground">{current?.label ?? (value || 'Pick')}</span>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-auto p-2">
        <div className="grid grid-cols-6 gap-1.5">
          {STATUS_COLORS.map((c) => (
            <button
              key={c.value}
              type="button"
              title={c.label}
              onClick={() => onChange(c.value)}
              className={cn(
                'relative size-7 rounded-full border-2 transition-all hover:scale-110 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-1',
                value === c.value ? 'border-foreground shadow-md scale-110' : 'border-transparent'
              )}
              style={{ backgroundColor: c.hex }}
            >
              {value === c.value && (
                <span className="absolute inset-0 flex items-center justify-center text-white text-[10px] font-bold drop-shadow">✓</span>
              )}
            </button>
          ))}
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

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
        config?.git?.commit ?? ['releases', 'features', 'adrs', 'specs', 'vision', 'ship.toml', 'templates'],
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
  onOpenAgentsModule: _onOpenAgentsModule,
  initialTab = 'global',
  panelMode = 'full',
}: SettingsPanelProps) {
  void _onOpenAgentsModule;
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
      await transformTextCmd('summarize', 'test connection');
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

  const agentsOnly = panelMode === 'agents-only';
  const settingsOnly = panelMode === 'settings-only';

  return (
    <PageFrame width="narrow" className="md:p-4">
      <PageHeader
        title={
          <span className="flex items-center gap-2">
            <Settings className="size-4 text-muted-foreground" />
            {agentsOnly ? 'Agents' : 'Settings'}
          </span>
        }
        description={agentsOnly ? 'Modes, MCP servers, and client sync' : 'Global and project configuration'}
        actions={<Badge variant="outline">Alpha</Badge>}
      />

      <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as typeof activeTab)} className="gap-2">
        {!agentsOnly && !settingsOnly && (
          <TabsList className="h-8 w-full justify-start rounded-lg border bg-muted/40 p-1">
            <TabsTrigger className="h-6 gap-1.5 px-3 text-xs" value="global">
              <Globe2 className="size-3" />Global
            </TabsTrigger>
            <TabsTrigger className="h-6 gap-1.5 px-3 text-xs" value="project" disabled={!projectConfig}>
              <GitBranch className="size-3" />Project
            </TabsTrigger>
            <TabsTrigger className="h-6 gap-1.5 px-3 text-xs" value="agents">
              <Cpu className="size-3" />Agents
            </TabsTrigger>
          </TabsList>
        )}

        {/* ── Global tab ──────────────────────────────────────────────────── */}
        <TabsContent value="global">
          <div className="grid gap-3 lg:grid-cols-2">
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-blue-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-blue-500/20 bg-blue-500/10">
                  <UserIcon className="size-3.5 text-blue-500" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">Identity</h3>
                  <p className="text-[11px] text-muted-foreground">Name and email for authorship metadata.</p>
                </div>
              </div>
              <CardContent className="space-y-2 !pt-5">
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

            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-violet-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-violet-500/20 bg-violet-500/10">
                  <Cpu className="size-3.5 text-violet-500" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">MCP Server</h3>
                  <p className="text-[11px] text-muted-foreground">Local bridge for AI clients and tooling.</p>
                </div>
              </div>
              <CardContent className="space-y-2 !pt-5">
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

            <Card size="sm" className="overflow-hidden lg:col-span-2">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-emerald-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-emerald-500/20 bg-emerald-500/10">
                  <Palette className="size-3.5 text-emerald-500" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold">Appearance &amp; Defaults</h3>
                  <p className="text-[11px] text-muted-foreground">Theme and creation defaults for new work items.</p>
                </div>
              </div>
              <CardContent className="grid gap-2 !pt-5 md:grid-cols-[1.2fr_1fr_1fr]">
                <div className="rounded-md border px-3 py-2">
                  <div className="mb-1.5 flex items-center justify-between gap-2">
                    <Label className="text-sm font-semibold tracking-tight">Appearance</Label>
                    <div className="flex items-center gap-3 rounded-full border bg-muted/20 p-1">
                      <div className={cn("flex items-center gap-1.5 px-2 py-1 rounded-full transition-all", (local.theme ?? 'dark') === 'light' ? "bg-background shadow-sm text-foreground" : "text-muted-foreground")}>
                        <Sun className="size-3.5" />
                        <span className="text-[10px] font-bold uppercase tracking-tighter">Light</span>
                      </div>
                      <Switch
                        checked={(local.theme ?? 'dark') === 'dark'}
                        onCheckedChange={(checked) => {
                          const theme = checked ? 'dark' : 'light';
                          setLocal({ ...local, theme });
                          onThemePreview(theme);
                        }}
                      />
                      <div className={cn("flex items-center gap-1.5 px-2 py-1 rounded-full transition-all", (local.theme ?? 'dark') === 'dark' ? "bg-background shadow-sm text-foreground" : "text-muted-foreground")}>
                        <Moon className="size-3.5" />
                        <span className="text-[10px] font-bold uppercase tracking-tighter">Dark</span>
                      </div>
                    </div>
                  </div>
                  <p className="text-muted-foreground text-[11px] opacity-70">Choose your preferred interface theme.</p>
                </div>
                <div className="space-y-2">
                  <Label>Default Workflow Status</Label>
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
            <div className="grid gap-2">
              <Card size="sm">
                <CardHeader className="pb-2">
                  <CardTitle>Project</CardTitle>
                  <CardDescription>Metadata stored in `.ship / ship.toml`.</CardDescription>
                </CardHeader>
                <CardContent className="space-y-2 !pt-5">
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

              <Card size="sm" className="overflow-hidden">
                <div className="flex items-center gap-3 border-b bg-gradient-to-r from-orange-500/10 via-card/80 to-card/50 px-4 py-3">
                  <div className="flex size-7 items-center justify-center rounded-lg border border-orange-500/20 bg-orange-500/10">
                    <span className="text-sm">🎨</span>
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold">Statuses</h3>
                    <p className="text-[11px] text-muted-foreground">Customize issue workflow columns for this project.</p>
                  </div>
                </div>
                <CardContent className="space-y-3 !pt-5">
                  <div className="hidden grid-cols-[1fr_1.2fr_auto_auto] gap-2 px-1 text-xs text-muted-foreground md:grid">
                    <span>ID</span>
                    <span>Name</span>
                    <span>Color</span>
                    <span />
                  </div>
                  {localProject.statuses.map((status, index) => (
                    <div key={`${status.id} -${index} `} className="grid items-start gap-2 md:grid-cols-[1fr_1.2fr_auto_auto]">
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
                      <StatusColorPicker
                        value={status.color ?? 'gray'}
                        onChange={(color) => updateStatus(index, { color })}
                      />
                      <Button variant="ghost" size="icon-xs" className="mt-0.5 text-destructive hover:text-destructive" onClick={() => removeStatus(index)}>
                        <Trash2 className="size-3.5" />
                      </Button>
                    </div>
                  ))}
                  <Separator />
                  <div className="grid items-start gap-2 md:grid-cols-[1fr_1.2fr_auto_auto]">
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
                    <StatusColorPicker
                      value={newStatus.color ?? 'gray'}
                      onChange={(color) => setNewStatus({ ...newStatus, color })}
                    />
                    <Button onClick={handleAddStatus} size="sm" className="mt-0.5">
                      <Plus className="size-3.5" />
                      Add
                    </Button>
                  </div>
                </CardContent>
              </Card>

              <Card size="sm">
                <CardHeader className="pb-2">
                  <CardTitle>Git Commit Categories</CardTitle>
                  <CardDescription>Choose which docs are staged by default for project commits.</CardDescription>
                </CardHeader>
                <CardContent className="grid gap-2 !pt-5 sm:grid-cols-2">
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
              <AgentScopeCard
                scope={agentScope}
                hasProject={hasActiveProject}
                onScopeChange={(next) => setAgentScope(next)}
              />

              <Card size="sm">
                <CardHeader>
                  <CardTitle>AI Provider</CardTitle>
                  <CardDescription>
                    Pass-through CLI provider used for generation features in the UI.
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3 !pt-5">
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
                  <CardTitle>Context Layer</CardTitle>
                  <CardDescription>
                    One place for skills, prompts, context, and rules.
                  </CardDescription>
                </CardHeader>
                <CardContent className="grid gap-3 !pt-5 lg:grid-cols-2">
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
                    <Label>Prompt Snippets</Label>
                    <MarkdownEditor
                      value={joinLines(activeAgentConfig.agent?.prompts)}
                      onChange={(value) =>
                        updateActiveAgentConfig({
                          ...activeAgentConfig,
                          agent: {
                            ...(activeAgentConfig.agent ?? EMPTY_AGENT_LAYER),
                            prompts: parseLines(value),
                          },
                        })
                      }
                      placeholder="Always produce patch-ready diffs"
                      rows={10}
                      defaultMode="doc"
                      showFrontmatter={false}
                      showStats={false}
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

                </CardContent>
              </Card>

              <Card size="sm">
                <CardHeader>
                  <CardTitle>Modes</CardTitle>
                  <CardDescription>
                    Mode switching is capability control. Keep this central and explicit.
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3 !pt-5">
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
                <CardContent className="space-y-3 !pt-5">
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
                            onClick={() => handleRemoveServer(server.id ?? server.name)}
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
                            : `Sync to ${target} `}
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
        {agentsOnly ? (
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
    </PageFrame>
  );
}
