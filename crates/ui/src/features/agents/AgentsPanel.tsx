import { useEffect, useMemo, useState } from 'react';
import { Plus, Trash2, Upload } from 'lucide-react';
import { McpServerConfig, ModeConfig, ProjectConfig } from '@/bindings';
import { exportAgentConfigCmd, generateIssueDescriptionCmd } from '@/lib/platform/tauri/commands';
import { DEFAULT_STATUSES } from '@/lib/workspace-ui';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Textarea } from '@/components/ui/textarea';
import AgentScopeCard from '@/features/agents/AgentScopeCard';

interface AgentsPanelProps {
  projectConfig: ProjectConfig | null;
  globalAgentConfig: ProjectConfig | null;
  onSaveProject: (config: ProjectConfig) => void | Promise<void>;
  onSaveGlobalAgentConfig: (config: ProjectConfig) => void | Promise<void>;
  initialSection?: AgentSection;
}

export type AgentSection = 'providers' | 'mcp' | 'skills' | 'prompts';

const AI_PROVIDERS = [
  { id: 'claude', label: 'Claude' },
  { id: 'gemini', label: 'Gemini' },
  { id: 'codex', label: 'Codex' },
];
const SCOPE_OPTIONS = ['global', 'project', 'mode'] as const;
const EMPTY_AGENT_LAYER = {
  skills: [],
  prompts: [],
  context: [],
  rules: [],
};
const DEFAULT_MODE_VALUE = 'default';
const SECTION_META: Record<AgentSection, { title: string; description: string }> = {
  providers: {
    title: 'Agent Providers',
    description: 'Provider/model configuration and mode controls.',
  },
  mcp: {
    title: 'Agent MCP',
    description: 'MCP server registry and client sync for the active scope.',
  },
  skills: {
    title: 'Agent Skills',
    description: 'Skill lists, context paths, and operating rules.',
  },
  prompts: {
    title: 'Agent Prompts',
    description: 'Prompt snippets and reusable instructions.',
  },
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

export default function AgentsPanel({
  projectConfig,
  globalAgentConfig,
  onSaveProject,
  onSaveGlobalAgentConfig,
  initialSection = 'providers',
}: AgentsPanelProps) {
  const [localProject, setLocalProject] = useState<ProjectConfig>(normalizeProjectConfig(projectConfig));
  const [localGlobalAgent, setLocalGlobalAgent] = useState<ProjectConfig>(
    normalizeProjectConfig(globalAgentConfig)
  );
  const [agentScope, setAgentScope] = useState<'project' | 'global'>(
    projectConfig ? 'project' : 'global'
  );
  const [newServer, setNewServer] = useState(EMPTY_SERVER);
  const [newMode, setNewMode] = useState<ModeConfig>(EMPTY_MODE);
  const [exportStatus, setExportStatus] = useState<Record<string, 'idle' | 'loading' | 'ok' | 'error'>>({});
  const [testStatus, setTestStatus] = useState<'idle' | 'loading' | 'ok' | 'error'>('idle');
  const [agentError, setAgentError] = useState<string | null>(null);

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

  const updateActiveAgentConfig = (next: ProjectConfig) => {
    if (agentScope === 'project') {
      setLocalProject(next);
      return;
    }
    setLocalGlobalAgent(next);
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

  const handleSave = () => {
    if (agentScope === 'global') {
      return onSaveGlobalAgentConfig(localGlobalAgent);
    }
    return onSaveProject(localProject);
  };
  const sectionMeta = SECTION_META[initialSection];

  return (
    <PageFrame className="md:p-8">
      <PageHeader
        title={sectionMeta.title}
        description={sectionMeta.description}
        badge={
          <Badge variant="outline" className="text-[10px]">
            {initialSection}
          </Badge>
        }
      />

      <div className="grid gap-4">
        <AgentScopeCard
          scope={agentScope}
          hasProject={hasActiveProject}
          onScopeChange={(next) => setAgentScope(next)}
        />

        {initialSection === 'providers' && (
          <div className="grid gap-4">
            <Card size="sm">
              <CardHeader>
                <CardTitle>Agent Selection</CardTitle>
                <CardDescription>Choose provider and model used by generation features.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="grid gap-3 lg:grid-cols-3">
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
                        {AI_PROVIDERS.map((provider) => (
                          <SelectItem key={provider.id} value={provider.id}>
                            {provider.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="agents-model">Model</Label>
                    <Input
                      id="agents-model"
                      value={activeAgentConfig.ai?.model ?? ''}
                      onChange={(event) =>
                        updateActiveAgentConfig({
                          ...activeAgentConfig,
                          ai: {
                            ...normalizeAiConfig(activeAgentConfig.ai),
                            model: event.target.value || null,
                          },
                        })
                      }
                      placeholder="haiku / sonnet / gpt-5 / gemini-2.0-flash"
                    />
                  </div>
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
                    {testStatus === 'loading' ? 'Testing…' : 'Test Agent'}
                  </Button>
                  {testStatus === 'ok' && (
                    <span className="text-xs text-emerald-500">Agent working ✓</span>
                  )}
                  {testStatus === 'error' && (
                    <span className="text-xs text-destructive">Failed — check binary/model/path</span>
                  )}
                </div>
                {!hasActiveProject && (
                  <p className="text-muted-foreground text-xs">Open a project to run provider tests.</p>
                )}
              </CardContent>
            </Card>

            <Card size="sm">
              <CardHeader>
                <CardTitle>Modes</CardTitle>
                <CardDescription>Modes define explicit capability boundaries.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
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
            <Card size="sm">
              <CardHeader>
                <CardTitle>MCP Server Registry</CardTitle>
                <CardDescription>Registry for MCP tools used by this scope.</CardDescription>
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
                        <Button variant="ghost" size="xs" onClick={() => handleRemoveServer(server.id)}>
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
                      onValueChange={(value) =>
                        setNewServer({
                          ...newServer,
                          scope: value as typeof newServer.scope,
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
                <CardDescription>Export MCP registry and agent docs to client configs.</CardDescription>
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
                  <p className="text-muted-foreground text-xs">Open a project to export client config files.</p>
                )}
              </CardContent>
            </Card>
          </div>
        )}

        {initialSection === 'skills' && (
          <div className="grid gap-4">
            <Card size="sm">
              <CardHeader>
                <CardTitle>Skills + Context</CardTitle>
                <CardDescription>Define reusable skills, context paths, and operating rules.</CardDescription>
              </CardHeader>
              <CardContent className="grid gap-3 lg:grid-cols-3">
                <div className="space-y-2">
                  <Label htmlFor="agents-skills">Skills (one per line)</Label>
                  <Textarea
                    id="agents-skills"
                    rows={8}
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
                  <Label htmlFor="agents-context">Context Paths (one per line)</Label>
                  <Textarea
                    id="agents-context"
                    rows={8}
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
                  <Label htmlFor="agents-rules">Rules (one per line)</Label>
                  <Textarea
                    id="agents-rules"
                    rows={8}
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
          </div>
        )}

        {initialSection === 'prompts' && (
          <div className="grid gap-4">
            <Card size="sm">
              <CardHeader>
                <CardTitle>Prompt Snippets</CardTitle>
                <CardDescription>Reusable instruction fragments for agents in this scope.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-2">
                <Label htmlFor="agents-prompts">Prompts (one per line)</Label>
                <Textarea
                  id="agents-prompts"
                  rows={12}
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
