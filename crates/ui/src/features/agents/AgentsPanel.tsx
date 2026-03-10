import React, { useEffect, useMemo, useRef, useState } from 'react';
import { Bot, Plus, Shield, ShieldAlert, FileSearch, Trash2, Upload, Download, Globe, Folder, Package, PenLine, ChevronDown, ChevronRight, Check, ScrollText, LockIcon, Info, Zap, BookOpen, Terminal, Link, Wrench } from 'lucide-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { commands, AgentDiscoveryCache, CatalogEntry, HookConfig, McpProbeReport, McpRegistryEntry, McpServerConfig, McpServerType, McpValidationIssue, McpValidationReport, ModeConfig, Permissions, ProjectConfig, ProviderInfo, SkillToolHint } from '@/bindings';
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
import { FileTree, FileTreeFile, FileTreeFolder } from '@ship/ui';
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
  description?: string | null;
  source?: string | null;
  author?: string | null;
  version?: string | null;
};

export type AgentSection = 'providers' | 'mcp' | 'skills' | 'rules' | 'hooks' | 'permissions';

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
  hooks: {
    title: 'Hooks',
    description: 'Lifecycle intercepts for context injection, policy enforcement, and session automation.',
  },
  permissions: {
    title: 'Permissions',
    description: 'Security policy: control what tools and paths your agent can access.',
  },
};

type HookEventOption = {
  value: string;
  label: string;
  providers: string[];
  matcherHint?: string;
};

const HOOK_EVENTS: HookEventOption[] = [
  { value: 'SessionStart', label: 'Session Start', providers: ['claude', 'gemini'] },
  { value: 'UserPromptSubmit', label: 'User Prompt Submit', providers: ['claude'] },
  { value: 'PreToolUse', label: 'Pre Tool Use', providers: ['claude', 'gemini'], matcherHint: 'Tool matcher (e.g. Bash, mcp__*).' },
  { value: 'PermissionRequest', label: 'Permission Request', providers: ['claude'] },
  { value: 'PostToolUse', label: 'Post Tool Use', providers: ['claude', 'gemini'], matcherHint: 'Tool matcher (e.g. Bash, mcp__*).' },
  { value: 'PostToolUseFailure', label: 'Post Tool Failure', providers: ['claude'] },
  { value: 'Notification', label: 'Notification', providers: ['claude', 'gemini'] },
  { value: 'SubagentStart', label: 'Subagent Start', providers: ['claude'] },
  { value: 'SubagentStop', label: 'Subagent Stop', providers: ['claude'] },
  { value: 'Stop', label: 'Stop', providers: ['claude', 'gemini'] },
  { value: 'PreCompact', label: 'Pre Compact', providers: ['claude', 'gemini'] },
  { value: 'BeforeTool', label: 'Before Tool', providers: ['gemini'], matcherHint: 'Tool matcher (e.g. run_shell_command).' },
  { value: 'AfterTool', label: 'After Tool', providers: ['gemini'], matcherHint: 'Tool matcher (e.g. run_shell_command).' },
  { value: 'BeforeAgent', label: 'Before Agent', providers: ['gemini'] },
  { value: 'AfterAgent', label: 'After Agent', providers: ['gemini'] },
  { value: 'SessionEnd', label: 'Session End', providers: ['gemini'] },
  { value: 'BeforeModel', label: 'Before Model', providers: ['gemini'] },
  { value: 'AfterModel', label: 'After Model', providers: ['gemini'] },
  { value: 'BeforeToolSelection', label: 'Before Tool Selection', providers: ['gemini'] },
];

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

type ProviderRow = ProviderInfo & {
  checking: boolean;
};

type ProviderSyncStatus = 'ready' | 'needs-attention' | 'drift-detected';

type ProviderSyncSummary = {
  status: ProviderSyncStatus;
  detail: string;
  issues: McpValidationIssue[];
};

type McpValidation = {
  level: 'info' | 'warning';
  message: string;
};

const PROVIDER_DRIFT_CODES = new Set([
  'provider-config-root',
  'provider-config-mcp-key',
]);

const PROVIDER_STATUS_COPY: Record<ProviderSyncStatus, string> = {
  ready: 'Ready',
  'needs-attention': 'Needs attention',
  'drift-detected': 'Drift detected',
};

const SUPPORTED_PROVIDER_BASE: Array<{ id: string; name: string; binary: string }> = [
  { id: 'claude', name: 'Claude Code', binary: 'claude' },
  { id: 'gemini', name: 'Gemini CLI', binary: 'gemini' },
  { id: 'codex', name: 'Codex CLI', binary: 'codex' },
];
const SUPPORTED_PROVIDER_IDS = new Set(SUPPORTED_PROVIDER_BASE.map((provider) => provider.id));
const MCP_STDIO_ONLY_ALPHA = true;

type ModePresetDefinition = {
  id: string;
  name: string;
  description: string;
  target_agents: string[];
  allow_tools: string[];
  deny_tools: string[];
  skill_hints: string[];
  mcp_hints: string[];
  default_skill_ids: string[];
  default_mcp_server_ids: string[];
  default_prompt_id: string | null;
};

const MODE_PRESETS: ModePresetDefinition[] = [
  {
    id: 'frontend-react',
    name: 'Frontend React',
    description: 'UI implementation template for React + TypeScript projects with design-system and accessibility focus.',
    target_agents: ['claude', 'codex'],
    allow_tools: ['Read', 'Glob', 'Grep', 'Edit', 'Write', 'MultiEdit', 'Bash', 'mcp__*__read*', 'mcp__*__list*', 'mcp__*__search*', 'mcp__*__write*'],
    deny_tools: ['mcp__*__delete*'],
    skill_hints: ['react', 'frontend', 'ui', 'accessibility', 'typescript', 'design system'],
    mcp_hints: ['playwright', 'storybook', 'browser', 'figma', 'design'],
    default_skill_ids: ['ship-workflow', 'task-policy', 'start-session'],
    default_mcp_server_ids: ['ship'],
    default_prompt_id: 'ship-workflow',
  },
  {
    id: 'rust-expert',
    name: 'Rust Expert',
    description: 'Systems programming template for Rust services, safety-first refactors, and performance tuning.',
    target_agents: ['claude', 'codex'],
    allow_tools: ['Read', 'Glob', 'Grep', 'Edit', 'Write', 'MultiEdit', 'Bash', 'mcp__*__read*', 'mcp__*__list*', 'mcp__*__search*'],
    deny_tools: ['mcp__*__delete*'],
    skill_hints: ['rust', 'cargo', 'clippy', 'tokio', 'systems', 'performance'],
    mcp_hints: ['postgres', 'redis', 'docker', 'kubernetes', 'sentry'],
    default_skill_ids: ['ship-workflow', 'task-policy', 'start-session'],
    default_mcp_server_ids: ['ship'],
    default_prompt_id: 'ship-workflow',
  },
  {
    id: 'documentation-expert',
    name: 'Documentation Expert',
    description: 'Docs template for ADRs, API docs, release notes, and repo knowledge curation.',
    target_agents: ['claude', 'gemini', 'codex'],
    allow_tools: ['Read', 'Glob', 'Grep', 'Edit', 'Write', 'MultiEdit', 'mcp__*__read*', 'mcp__*__list*', 'mcp__*__search*'],
    deny_tools: ['Bash', 'mcp__*__write*', 'mcp__*__delete*', 'mcp__*__exec*'],
    skill_hints: ['docs', 'documentation', 'adr', 'api', 'release notes', 'writer'],
    mcp_hints: ['github', 'linear', 'notion', 'docs'],
    default_skill_ids: ['ship-workflow', 'task-policy', 'create-document'],
    default_mcp_server_ids: ['ship'],
    default_prompt_id: 'create-document',
  },
];

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
    name: 'Ship Guarded',
    description: 'Ship-first baseline — read + Ship MCP by default, risky mutations require explicit opt-in.',
    icon: Shield,
    colorClass: 'text-emerald-500',
    apply: () => ({
      tools: {
        allow: ['Read', 'Glob', 'Grep', 'mcp__ship__*', 'mcp__*__read*', 'mcp__*__list*', 'mcp__*__search*'],
        deny: ['Bash', 'Write', 'Edit', 'MultiEdit', 'mcp__*__write*', 'mcp__*__delete*', 'mcp__*__exec*'],
      },
      filesystem: { allow: ['**/*'], deny: ['/etc/**', '/sys/**', '/proc/**', '~/.ssh/**', '~/.gnupg/**'] },
      commands: {
        allow: ['git status', 'git diff', 'git log', 'ls', 'cat', 'rg', 'find', 'pwd'],
        deny: ['rm -rf', 'git push --force', 'npm publish', 'cargo publish'],
      },
      network: { policy: 'none', allow_hosts: [] },
      agent: {
        max_cost_per_session: 5.0,
        max_turns: 50,
        require_confirmation: ['Bash', 'Write', 'Edit', 'MultiEdit', 'mcp__*__write*', 'mcp__*__delete*', 'mcp__*__exec*'],
      },
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
  idOptions,
  commandOptions,
  envKeyOptions,
  isNew,
}: {
  draft: McpServerConfig;
  onChange: (server: McpServerConfig) => void;
  onSave: () => void;
  onCancel: () => void;
  idOptions: Array<{ value: string; label?: string; keywords?: string[] }>;
  commandOptions: Array<{ value: string; label?: string; keywords?: string[] }>;
  envKeyOptions: Array<{ value: string; label?: string; keywords?: string[] }>;
  isNew?: boolean;
}) {
  const transport = draft.server_type ?? 'stdio';
  const argsStr = (draft.args ?? []).join(' ');
  const validations = getMcpTemplateValidation(draft);
  const setField = <K extends keyof McpServerConfig>(key: K, value: McpServerConfig[K]) =>
    onChange({ ...draft, [key]: value });

  return (
    <div className="border-t bg-muted/20 px-4 py-4 space-y-3">
      <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
        {isNew ? 'New MCP Server' : 'Edit Server'}
      </p>

      <div className="grid gap-3 sm:grid-cols-[1fr_1fr_auto]">
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">Name</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>Display name shown in UI and provider config exports.</TooltipContent>
            </Tooltip>
          </div>
          <Input
            value={draft.name}
            onChange={(e) => setField('name', e.target.value)}
            placeholder="e.g. shipwright"
            className="h-8 text-xs"
            autoFocus={isNew}
          />
        </div>
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">Server ID</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>Stable slug used in permissions, modes, and exports.</TooltipContent>
            </Tooltip>
          </div>
          <AutocompleteInput
            value={draft.id ?? ''}
            options={idOptions}
            placeholder={slugifyId(draft.name || 'server-id') || 'server-id'}
            onValueChange={(value) => setField('id', value)}
            className="h-8 text-xs font-mono"
          />
        </div>
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">Transport</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>
                {MCP_STDIO_ONLY_ALPHA
                  ? 'Alpha currently supports stdio MCP servers only.'
                  : 'How Ship connects to this MCP server: local process, SSE, or HTTP.'}
              </TooltipContent>
            </Tooltip>
          </div>
          <Select value={transport} onValueChange={(v) => setField('server_type', v as McpServerType)}>
            <SelectTrigger size="sm" className="w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="stdio">stdio</SelectItem>
              <SelectItem value="sse" disabled={MCP_STDIO_ONLY_ALPHA}>SSE</SelectItem>
              <SelectItem value="http" disabled={MCP_STDIO_ONLY_ALPHA}>HTTP</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {transport === 'stdio' ? (
        <div className="grid gap-3 sm:grid-cols-[1fr_1fr]">
          <div className="space-y-1.5">
            <div className="flex items-center gap-1.5">
              <Label className="text-xs">Command</Label>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="size-3 cursor-default text-muted-foreground" />
                </TooltipTrigger>
                <TooltipContent>Executable launched for stdio servers (resolved from PATH if not absolute).</TooltipContent>
              </Tooltip>
            </div>
            <AutocompleteInput
              value={draft.command}
              options={commandOptions}
              onValueChange={(value) => setField('command', value)}
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
              onChange={(e) => setField('args', splitShellArgs(e.target.value))}
              placeholder="--port 3000"
              className="h-8 text-xs font-mono"
            />
          </div>
        </div>
      ) : (
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5">
            <Label className="text-xs">URL</Label>
            <Tooltip>
              <TooltipTrigger asChild>
                <Info className="size-3 cursor-default text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>Endpoint for HTTP/SSE transport, including protocol and port.</TooltipContent>
            </Tooltip>
          </div>
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
                <AutocompleteInput
                  value={key}
                  options={envKeyOptions}
                  onValueChange={(value) => {
                    const entries = Object.entries(draft.env ?? {});
                    entries[envIdx] = [value, val ?? ''];
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

      {validations.length > 0 && (
        <div className="space-y-1.5 rounded-md border bg-background/50 px-2.5 py-2">
          {validations.map((check, idx) => (
            <p
              key={`${check.message}-${idx}`}
              className={cn(
                "text-[11px]",
                check.level === 'warning' ? 'text-amber-600' : 'text-muted-foreground'
              )}
            >
              {check.level === 'warning' ? 'Warning' : 'Hint'}: {check.message}
            </p>
          ))}
        </div>
      )}

      <div className="flex items-center gap-2 pt-1">
        <Button
          size="sm"
          onClick={onSave}
          disabled={
            !draft.name.trim()
            || (transport === 'stdio' && !draft.command.trim())
            || (transport !== 'stdio' && !draft.url?.trim())
            || (MCP_STDIO_ONLY_ALPHA && transport !== 'stdio')
          }
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

function formatEpochSeconds(value: string): string {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return value;
  return new Date(numeric * 1000).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
  });
}

function slugifyId(value: string): string {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 64);
}

function inferSkillIdFromSource(source: string): string {
  return parseSkillSourceInstallSpec(source).inferredSkillId;
}

type SkillSourceInstallSpec = {
  source: string;
  gitRef: string | null;
  repoPath: string | null;
  inferredSkillId: string;
};

function parseSkillSourceInstallSpec(rawSource: string): SkillSourceInstallSpec {
  const trimmed = rawSource.trim().replace(/\/+$/g, '');
  if (!trimmed) {
    return { source: '', gitRef: null, repoPath: null, inferredSkillId: '' };
  }

  const githubTree = trimmed.match(
    /^https?:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?\/(tree|blob)\/([^/]+)\/(.+)$/i
  );
  if (githubTree) {
    const owner = githubTree[1];
    const repo = githubTree[2];
    const ref = githubTree[4];
    const rawPath = githubTree[5].replace(/\/+$/g, '');
    const normalizedPath = rawPath.replace(/\/SKILL\.md$/i, '');
    const pathSegments = normalizedPath.split('/').filter(Boolean);
    const inferred = slugifyId(pathSegments[pathSegments.length - 1] ?? repo);
    return {
      source: `${owner}/${repo}`,
      gitRef: ref || null,
      repoPath: normalizedPath || '.',
      inferredSkillId: inferred,
    };
  }

  const base = trimmed.replace(/\.git$/i, '');
  const segments = base.split('/').filter(Boolean);
  const candidate = segments[segments.length - 1] ?? '';
  return {
    source: trimmed,
    gitRef: null,
    repoPath: null,
    inferredSkillId: slugifyId(candidate),
  };
}

function inferMcpServerId(server: McpServerConfig): string {
  const explicit = (server.id ?? '').trim();
  if (explicit) return slugifyId(explicit);
  const fromName = slugifyId(server.name || '');
  if (fromName) return fromName;
  if (server.command) return slugifyId(server.command);
  return `mcp-${Date.now()}`;
}

function splitShellArgs(raw: string): string[] {
  const input = raw.trim();
  if (!input) return [];
  const matches = input.match(/(?:[^\s"']+|"[^"]*"|'[^']*')+/g) ?? [];
  return matches
    .map((segment) => segment.replace(/^['"]|['"]$/g, ''))
    .filter(Boolean);
}

function getMcpTemplateValidation(server: McpServerConfig): McpValidation[] {
  const checks: McpValidation[] = [];
  const transport = server.server_type ?? 'stdio';
  if (MCP_STDIO_ONLY_ALPHA && transport !== 'stdio') {
    checks.push({
      level: 'warning',
      message: 'HTTP/SSE transports are deferred for alpha. Use stdio transport.',
    });
  }
  if (transport === 'stdio') {
    if (!server.command.trim()) {
      checks.push({ level: 'warning', message: 'Command is required for stdio transport.' });
    }
    if (/\s/.test(server.command.trim()) && (server.args ?? []).length === 0) {
      checks.push({
        level: 'info',
        message: 'Command includes spaces. Prefer command + args split for provider portability.',
      });
    }
    if (/[;&|]{1,2}/.test(server.command)) {
      checks.push({
        level: 'warning',
        message: 'Command contains shell operators. Split into command + args to avoid provider parser issues.',
      });
    }
  } else if (server.url?.trim()) {
    try {
      // eslint-disable-next-line no-new
      new URL(server.url.trim());
    } catch {
      checks.push({ level: 'warning', message: 'URL appears invalid. Use a fully qualified URL like https://host/path.' });
    }
  } else {
    checks.push({ level: 'warning', message: 'URL is required for HTTP/SSE transport.' });
  }

  const unresolved = (server.args ?? []).filter((arg) => /^\{.+\}$/.test(arg));
  if (unresolved.length > 0) {
    checks.push({
      level: 'info',
      message: `Replace placeholder args before saving: ${unresolved.join(', ')}`,
    });
  }
  const jsonLikeArgs = (server.args ?? []).filter((arg) => {
    const trimmed = arg.trim();
    return trimmed.startsWith('{') || trimmed.startsWith('[');
  });
  jsonLikeArgs.forEach((arg) => {
    try {
      JSON.parse(arg);
    } catch {
      checks.push({
        level: 'warning',
        message: `Argument looks like JSON but is invalid: ${arg.slice(0, 40)}${arg.length > 40 ? '…' : ''}`,
      });
    }
  });

  const badEnv = Object.keys(server.env ?? {}).filter((key) => !/^[A-Z_][A-Z0-9_]*$/.test(key));
  if (badEnv.length > 0) {
    checks.push({
      level: 'warning',
      message: `Env keys should be uppercase snake_case: ${badEnv.join(', ')}`,
    });
  }
  const emptySecretKeys = Object.entries(server.env ?? {})
    .filter(([key, value]) => /(TOKEN|KEY|SECRET|PASSWORD)/.test(key) && !String(value ?? '').trim())
    .map(([key]) => key);
  if (emptySecretKeys.length > 0) {
    checks.push({
      level: 'info',
      message: `Add values for secret env keys before export: ${emptySecretKeys.join(', ')}`,
    });
  }
  return checks;
}

function mcpToolPattern(serverId: string, toolName: string): string {
  return `mcp__${serverId}__${toolName}`;
}

function isMcpToolDenied(permissions: Permissions | undefined, serverId: string, toolName: string): boolean {
  if (!permissions) return false;
  const deny = permissions.tools?.deny ?? [];
  const exact = mcpToolPattern(serverId, toolName);
  return deny.includes(exact) || deny.includes(`mcp__${serverId}__*`) || deny.includes('mcp__*__*');
}

function isMcpServerDenied(permissions: Permissions | undefined, serverId: string): boolean {
  if (!permissions) return false;
  return (permissions.tools?.deny ?? []).includes(`mcp__${serverId}__*`);
}

function mcpServerFromCatalog(entry: CatalogEntry): McpServerConfig {
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

function mcpServerFromRegistry(entry: McpRegistryEntry): McpServerConfig {
  const env = Object.fromEntries((entry.required_env ?? []).map((key) => [key, '']));
  const transport = (entry.transport ?? 'stdio').toLowerCase();
  const serverType: McpServerType =
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

function sourceLabel(source?: string | null): string {
  if (!source) return 'custom';
  return source;
}

function hasYamlFrontmatter(content: string): boolean {
  const trimmed = content.trimStart();
  if (!trimmed.startsWith('---\n')) return false;
  return trimmed.slice(4).includes('\n---');
}

function includesAnyHint(value: string, hints: string[]): boolean {
  const normalized = value.trim().toLowerCase();
  if (!normalized) return false;
  return hints.some((hint) => normalized.includes(hint.toLowerCase()));
}

function getProviderSyncSummary(
  provider: ProviderRow,
  enabled: boolean,
  validationReport: McpValidationReport | null
): ProviderSyncSummary {
  const providerIssues = (validationReport?.issues ?? []).filter((issue) => issue.provider_id === provider.id);
  const sharedBlockingIssues = enabled
    ? (validationReport?.issues ?? []).filter((issue) => !issue.provider_id && issue.level !== 'info')
    : [];
  const blockingProviderIssues = providerIssues.filter(
    (issue) => issue.level === 'error' || (issue.level === 'warning' && !PROVIDER_DRIFT_CODES.has(issue.code))
  );
  const driftIssues = providerIssues.filter(
    (issue) => issue.level !== 'error' && PROVIDER_DRIFT_CODES.has(issue.code)
  );

  if (enabled && !provider.installed) {
    return {
      status: 'needs-attention',
      detail: `${provider.binary} is not installed or not on PATH.`,
      issues: [...blockingProviderIssues, ...sharedBlockingIssues],
    };
  }

  if (blockingProviderIssues.length > 0 || sharedBlockingIssues.length > 0) {
    return {
      status: 'needs-attention',
      detail: 'Fix the reported config or MCP issues before syncing this provider.',
      issues: [...blockingProviderIssues, ...sharedBlockingIssues],
    };
  }

  if (driftIssues.length > 0) {
    return {
      status: 'drift-detected',
      detail: 'Provider config shape diverges from Ship-managed expectations.',
      issues: driftIssues,
    };
  }

  if (!enabled) {
    return {
      status: 'ready',
      detail: 'Provider sync is currently off in this scope.',
      issues: [],
    };
  }

  return {
    status: 'ready',
    detail: 'Ready to export from Ship config.',
    issues: [],
  };
}

function providerStatusBadgeClass(status: ProviderSyncStatus): string {
  if (status === 'ready') {
    return 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300';
  }
  if (status === 'drift-detected') {
    return 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300';
  }
  return 'border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-300';
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
  const [expandedProviderId, setExpandedProviderId] = useState<string>('claude');
  const [expandedModeId, setExpandedModeId] = useState<string | null>(null);
  const [editingMode, setEditingMode] = useState<ModeConfig | null>(null);
  const [exportStatus, setExportStatus] = useState<Record<string, 'idle' | 'loading' | 'ok' | 'error'>>({});
  const [importStatus, setImportStatus] = useState<Record<string, 'idle' | 'loading' | 'ok' | 'error'>>({});
  const [importSummary, setImportSummary] = useState<Record<string, string>>({});
  const [agentError, setAgentError] = useState<string | null>(null);
  const [mcpEditDraft, setMcpEditDraft] = useState<McpEditDraft | null>(null);
  const [skillStudioMode, setSkillStudioMode] = useState<boolean>(true);
  const [skillTreeExpanded, setSkillTreeExpanded] = useState<Set<string>>(() => new Set());
  const [mcpCatalogInput, setMcpCatalogInput] = useState('');
  const [skillCatalogInput, setSkillCatalogInput] = useState('');
  const [skillSourceInput, setSkillSourceInput] = useState('');
  const [skillSourceIdInput, setSkillSourceIdInput] = useState('');
  const hasAutoScopedToProjectRef = useRef<boolean>(!!projectConfig);
  const discoveryRefreshedRef = useRef<Record<ScopeKey, boolean>>({
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

  // Providers Query
  const providersQuery = useQuery({
    queryKey: ['providers'],
    queryFn: async () => {
      const res = await commands.listProvidersCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'providers',
  });
  const providers = providersQuery.data ?? [];

  // Catalog Query
  const { data: catalog = [] } = useQuery({
    queryKey: ['catalog'],
    queryFn: async () => commands.listCatalogCmd(),
    enabled:
      initialSection === 'mcp' ||
      initialSection === 'skills' ||
      initialSection === 'permissions',
  });

  // Skills Query
  const { data: skills = [] } = useQuery({
    queryKey: ['skills', agentScope],
    queryFn: async () => {
      const res = await commands.listSkillsCmd(skillScope);
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
      : rules.map((r) => ({ id: r.file_name, title: r.file_name, content: r.content, updated: '' }));
  const mcpCatalogEntries = useMemo(
    () => catalog.filter((entry) => entry.kind === 'mcp-server'),
    [catalog]
  );
  const skillCatalogEntries = useMemo(
    () => catalog.filter((entry) => entry.kind === 'skill'),
    [catalog]
  );

  const activeSelectedDocId = activeDocKind ? selectedDocIds[agentScope][activeDocKind] : null;
  const activeDoc = activeDocs.find((doc) => doc.id === activeSelectedDocId) ?? activeDocs[0] ?? null;
  const skillScopeRoot =
    agentScope === 'project'
      ? '.ship/skills'
      : '~/.ship/skills';

  const selectActiveDoc = (kind: MarkdownDocKind, docId: string) => {
    setSelectedDocIds((current) => ({
      ...current,
      [agentScope]: { ...current[agentScope], [kind]: docId },
    }));
  };

  const handleSkillTreeSelect = (path: string) => {
    const normalizedPrefix = `${skillScopeRoot}/`;
    if (!path.startsWith(normalizedPrefix)) return;
    const relative = path.slice(normalizedPrefix.length);
    const [skillId] = relative.split('/');
    if (!skillId) return;
    selectActiveDoc('skills', skillId);
    setSkillTreeExpanded((current) => {
      const next = new Set(current);
      next.add(skillScopeRoot);
      next.add(`${skillScopeRoot}/${skillId}`);
      return next;
    });
  };

  // Mutations
  const createSkillMut = useMutation({
    mutationFn: async (vars: { id: string; name: string; content: string }) => {
      const res = await commands.createSkillCmd(vars.id, vars.name, vars.content, skillScope);
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
      const res = await commands.updateSkillCmd(vars.id, vars.name ?? null, vars.content ?? null, skillScope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills', agentScope] });
    },
  });

  const deleteSkillMut = useMutation({
    mutationFn: async (id: string) => {
      const res = await commands.deleteSkillCmd(id, skillScope);
      if (res.status === 'error') throw new Error(res.error);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills', agentScope] });
      setSelectedDocIds((curr) => ({ ...curr, [agentScope]: { ...curr[agentScope], skills: null } }));
    },
  });

  const installSkillFromSourceMut = useMutation({
    mutationFn: async (vars: { source: string; skillId: string; gitRef?: string | null; repoPath?: string | null }) => {
      const res = await commands.installSkillFromSourceCmd(
        vars.source,
        vars.skillId,
        vars.gitRef ?? null,
        vars.repoPath ?? null,
        skillScope,
        false
      );
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    onSuccess: (installedSkill) => {
      queryClient.invalidateQueries({ queryKey: ['skills', agentScope] });
      setSelectedDocIds((curr) => ({
        ...curr,
        [agentScope]: { ...curr[agentScope], skills: installedSkill.id },
      }));
      setSkillSourceInput('');
      setSkillSourceIdInput('');
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
  const mcpValidationReport = mcpValidationQuery.data ?? null;
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
  const skillToolHints = (skillToolHintsQuery.data ?? []) as SkillToolHint[];
  const mcpRegistryQuery = useQuery({
    queryKey: ['mcp-registry', mcpCatalogInput.trim().toLowerCase()],
    queryFn: async () => {
      const query = mcpCatalogInput.trim();
      const res = await commands.searchMcpRegistryCmd(query.length > 0 ? query : null, 20);
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'mcp',
    staleTime: 120000,
  });
  const mcpRegistryEntries = mcpRegistryQuery.data ?? [];

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
    if (initialSection !== 'skills') return;
    setSkillTreeExpanded((current) => {
      const next = new Set(current);
      next.add(skillScopeRoot);
      if (activeDoc?.id) {
        next.add(`${skillScopeRoot}/${activeDoc.id}`);
      }
      return next;
    });
  }, [initialSection, skillScopeRoot, activeDoc?.id]);
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
  }, [
    initialSection,
    agentScope,
    discoveryCache,
    refreshDiscoveryCacheMut,
  ]);
  useEffect(() => {
    if (!mcpProbeReport) return;
    queryClient.invalidateQueries({ queryKey: ['agent-discovery', agentScope] });
  }, [mcpProbeReport, queryClient, agentScope]);

  const hasActiveProject = !!projectConfig;
  const activeAgentConfig = useMemo(
    () => (agentScope === 'project' ? localProject : localGlobalAgent),
    [agentScope, localGlobalAgent, localProject]
  );
  const providerRows = useMemo<ProviderRow[]>(() => {
    const enabled = new Set(activeAgentConfig.providers ?? []);
    const detectedById = new Map(providers.map((provider) => [provider.id, provider]));
    const checking = providersQuery.isPending;
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
  }, [providers, activeAgentConfig.providers, providersQuery.isPending]);
  const catalogMcpOptions = useMemo(
    () =>
      mcpCatalogEntries.map((entry) => ({
        value: entry.id,
        label: entry.name,
        keywords: [entry.description, ...(entry.tags ?? [])],
      })),
    [mcpCatalogEntries]
  );
  const registryMcpOptions = useMemo(
    () =>
      mcpRegistryEntries.map((entry) => ({
        value: entry.server_name,
        label: `${entry.title} (Registry)`,
        keywords: [
          entry.description,
          entry.version,
          entry.transport,
          entry.id,
        ],
      })),
    [mcpRegistryEntries]
  );
  const allMcpOptions = useMemo(
    () => [...catalogMcpOptions, ...registryMcpOptions],
    [catalogMcpOptions, registryMcpOptions]
  );
  const catalogSkillOptions = useMemo(
    () =>
      skillCatalogEntries.map((entry) => ({
        value: entry.id,
        label: entry.name,
        keywords: [entry.description, ...(entry.tags ?? [])],
      })),
    [skillCatalogEntries]
  );
  const catalogSkillSourceOptions = useMemo(
    () =>
      skillCatalogEntries
        .map((entry) => entry.source_url?.trim() ?? '')
        .filter(Boolean)
        .map((value) => ({ value })),
    [skillCatalogEntries]
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
    const fromCatalog = mcpCatalogEntries.flatMap((entry) => [
      entry.command ?? '',
      entry.install_command ?? '',
    ]);
    const fromRegistry = mcpRegistryEntries.flatMap((entry) => [
      entry.command ?? '',
    ]);
    const fromExisting = (activeAgentConfig.mcp_servers ?? []).map((server) => server.command ?? '');
    const values = [...seeded, ...fromCatalog, ...fromRegistry, ...fromExisting]
      .map((value) => value.trim())
      .filter(Boolean);
    return Array.from(new Set(values)).map((value) => ({ value }));
  }, [mcpCatalogEntries, mcpRegistryEntries, activeAgentConfig.mcp_servers]);
  const mcpEnvKeyOptions = useMemo(() => {
    const seeded = ['GITHUB_TOKEN', 'BRAVE_API_KEY', 'SLACK_BOT_TOKEN', 'OPENAI_API_KEY', 'ANTHROPIC_API_KEY'];
    const fromRegistry = mcpRegistryEntries.flatMap((entry) => entry.required_env ?? []);
    const fromExisting = (activeAgentConfig.mcp_servers ?? []).flatMap((server) => Object.keys(server.env ?? {}));
    const values = [...seeded, ...fromRegistry, ...fromExisting].filter(Boolean);
    return Array.from(new Set(values)).map((value) => ({ value }));
  }, [mcpRegistryEntries, activeAgentConfig.mcp_servers]);
  const mcpTemplateEntry = useMemo(() => {
    const query = mcpCatalogInput.trim().toLowerCase();
    if (!query) return null;
    return (
      mcpCatalogEntries.find((entry) => entry.id.toLowerCase() === query) ??
      mcpCatalogEntries.find((entry) => entry.name.toLowerCase() === query) ??
      null
    );
  }, [mcpCatalogInput, mcpCatalogEntries]);
  const mcpRegistryTemplateEntry = useMemo(() => {
    const query = mcpCatalogInput.trim().toLowerCase();
    if (!query) return null;
    return (
      mcpRegistryEntries.find((entry) => entry.server_name.toLowerCase() === query) ??
      mcpRegistryEntries.find((entry) => entry.title.toLowerCase() === query) ??
      mcpRegistryEntries.find((entry) => entry.id.toLowerCase() === query) ??
      null
    );
  }, [mcpCatalogInput, mcpRegistryEntries]);
  const mcpProbeByServerId = useMemo(
    () => new Map((mcpProbeReport?.results ?? []).map((result) => [result.server_id, result])),
    [mcpProbeReport]
  );
  const cachedMcpToolsByServerId = useMemo(() => {
    const map = new Map<string, Array<{ name: string; description?: string | null }>>();
    Object.entries(discoveryCache?.mcp_tools ?? {}).forEach(([serverId, tools]) => {
      map.set(serverId, tools);
    });
    return map;
  }, [discoveryCache]);
  const discoveredMcpToolPatterns = useMemo(() => {
    const fromProbe = (mcpProbeReport?.results ?? []).flatMap((result) =>
      result.discovered_tools.map((tool) => mcpToolPattern(result.server_id, tool.name))
    );
    const fromCache = Array.from(cachedMcpToolsByServerId.entries()).flatMap(([serverId, tools]) =>
      tools.map((tool) => mcpToolPattern(serverId, tool.name))
    );
    return Array.from(new Set([...fromProbe, ...fromCache]));
  }, [mcpProbeReport, cachedMcpToolsByServerId]);
  const skillTemplateEntry = useMemo(() => {
    const query = skillCatalogInput.trim().toLowerCase();
    if (!query) return null;
    return (
      skillCatalogEntries.find((entry) => entry.id.toLowerCase() === query) ??
      skillCatalogEntries.find((entry) => entry.name.toLowerCase() === query) ??
      null
    );
  }, [skillCatalogInput, skillCatalogEntries]);
  const inferredSkillSourceId = useMemo(
    () => inferSkillIdFromSource(skillSourceInput),
    [skillSourceInput]
  );
  const modePresetMatches = useMemo(() => {
    return MODE_PRESETS.reduce<Record<string, { skillIds: string[]; mcpServerIds: string[] }>>((acc, preset) => {
      const matchingSkills = skills
        .filter((skill) => includesAnyHint(`${skill.id} ${skill.name ?? ''}`, preset.skill_hints))
        .map((skill) => skill.id);
      const matchingMcpServers = (activeAgentConfig.mcp_servers ?? [])
        .filter((server) => includesAnyHint(`${server.id ?? ''} ${server.name ?? ''}`, preset.mcp_hints))
        .map((server) => server.id ?? server.name)
        .filter(Boolean);
      acc[preset.id] = {
        skillIds: Array.from(new Set(matchingSkills)),
        mcpServerIds: Array.from(new Set(matchingMcpServers)),
      };
      return acc;
    }, {});
  }, [skills, activeAgentConfig.mcp_servers]);
  const skillToolAllowedPatterns = useMemo(() => {
    const direct = skillToolHints.flatMap((hint) => hint.allowed_tools ?? []);
    return Array.from(
      new Set(
        direct
          .map((value) => value.trim())
          .filter(Boolean)
      )
    );
  }, [skillToolHints]);
  const permissionToolSuggestions = useMemo(() => {
    const serverPatterns = (activeAgentConfig.mcp_servers ?? [])
      .map((server) => (server.id ?? '').trim())
      .filter(Boolean)
      .flatMap((id) => [
        `mcp__${id}__*`,
        `mcp__${id}__read*`,
        `mcp__${id}__write*`,
      ]);
    const catalogPatterns = mcpCatalogEntries
      .map((entry) => entry.id.startsWith('mcp-') ? entry.id.slice(4) : entry.id)
      .flatMap((id) => [`mcp__${id}__*`, `mcp__${id}__read*`, `mcp__${id}__write*`]);
    const baseline = ['*', 'mcp__*__*', 'mcp__*__read*', 'mcp__*__write*', 'mcp__*__delete*'];
    return Array.from(new Set([
      ...baseline,
      ...serverPatterns,
      ...catalogPatterns,
      ...discoveredMcpToolPatterns,
      ...skillToolAllowedPatterns,
    ]))
      .map((value) => ({ value }));
  }, [
    activeAgentConfig.mcp_servers,
    mcpCatalogEntries,
    discoveredMcpToolPatterns,
    skillToolAllowedPatterns,
  ]);
  const hookCommandSuggestions = useMemo(() => {
    const seeded = ['$SHIP_HOOKS_BIN', 'ship hooks run', 'node', 'bash'];
    const shellValues = (discoveryCache?.shell_commands ?? []).slice(0, 120);
    const values = [...seeded, ...mcpCommandOptions.map((option) => option.value), ...shellValues];
    return Array.from(new Set(values.filter(Boolean))).map((value) => ({ value }));
  }, [mcpCommandOptions, discoveryCache]);
  const hookMatcherSuggestions = useMemo(() => {
    const seeded = [
      'Bash',
      'Edit',
      'Write',
      'Read',
      'Glob',
      'Grep',
      'mcp__*',
      'mcp__*__read*',
      'mcp__*__write*',
      'run_shell_command',
    ];
    const values = [...seeded, ...permissionToolSuggestions.map((option) => option.value)];
    return Array.from(new Set(values.filter(Boolean))).map((value) => ({ value }));
  }, [permissionToolSuggestions]);
  const filesystemPathSuggestions = useMemo(
    () => {
      const seeded = [
        '**/*',
        '.ship/**',
        'src/**',
        'docs/**',
        'tests/**',
        '~/.ssh/**',
        '~/.gnupg/**',
        '/etc/**',
        '/proc/**',
        '/sys/**',
      ];
      const discovered = discoveryCache?.filesystem_paths ?? [];
      return Array.from(new Set([...seeded, ...discovered])).map((value) => ({ value }));
    },
    [discoveryCache]
  );
  const commandPatternSuggestions = useMemo(() => {
    const seeded = ['ship', 'ship *', 'gh', 'gh *', 'git', 'git *', 'cargo', 'cargo *', 'npm', 'npm *', 'pnpm', 'pnpm *', 'python', 'python *', 'bash', 'bash *'];
    const discovered = (discoveryCache?.shell_commands ?? []).flatMap((command) => [command, `${command} *`]);
    return Array.from(new Set([...seeded, ...discovered]))
      .map((value) => value.trim())
      .filter(Boolean)
      .map((value) => ({ value }));
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

  // Permissions Query
  const { data: permissions } = useQuery({
    queryKey: ['permissions'],
    queryFn: async () => {
      const res = await commands.getPermissionsCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: initialSection === 'permissions' || initialSection === 'mcp',
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

  const buildUniqueSkillId = (baseId: string): string => {
    const normalized = slugifyId(baseId) || `skill-${Date.now()}`;
    const existing = new Set(skills.map((skill) => skill.id));
    if (!existing.has(normalized)) return normalized;
    let index = 2;
    let candidate = `${normalized}-${index}`;
    while (existing.has(candidate)) {
      index += 1;
      candidate = `${normalized}-${index}`;
    }
    return candidate;
  };

  const handleApplyMcpTemplate = () => {
    if (mcpTemplateEntry) {
      setMcpEditDraft({
        idx: null,
        server: mcpServerFromCatalog(mcpTemplateEntry),
      });
      return;
    }
    if (!mcpRegistryTemplateEntry) return;
    if (MCP_STDIO_ONLY_ALPHA && (mcpRegistryTemplateEntry.transport ?? '').toLowerCase() !== 'stdio') {
      setAgentError(
        `Registry entry '${mcpRegistryTemplateEntry.title}' is ${mcpRegistryTemplateEntry.transport.toUpperCase()} transport. HTTP/SSE is deferred for alpha.`
      );
      return;
    }
    setMcpEditDraft({
      idx: null,
      server: mcpServerFromRegistry(mcpRegistryTemplateEntry),
    });
  };

  const handleInstallRegistryEntry = (entry: McpRegistryEntry) => {
    const normalizedServer = mcpServerFromRegistry(entry);
    if (MCP_STDIO_ONLY_ALPHA && normalizedServer.server_type !== 'stdio') {
      setAgentError(
        `Registry entry '${entry.title}' uses ${entry.transport.toUpperCase()} transport. HTTP/SSE is deferred for alpha; choose a stdio MCP server.`
      );
      return;
    }
    const servers = [...(activeAgentConfig.mcp_servers ?? [])];
    const existingIds = new Set(
      servers
        .map((server) => (server.id ?? '').trim())
        .filter(Boolean)
    );
    const baseId = inferMcpServerId(normalizedServer);
    let id = baseId;
    let index = 2;
    while (existingIds.has(id)) {
      id = `${baseId}-${index}`;
      index += 1;
    }
    const nextServer: McpServerConfig = {
      ...normalizedServer,
      id,
      name: normalizedServer.name.trim() || id,
    };
    servers.push(nextServer);
    updateActiveAgentConfig({ ...activeAgentConfig, mcp_servers: servers });
    setMcpEditDraft({ idx: servers.length - 1, server: nextServer });
  };

  const handleApplySkillTemplate = () => {
    if (!skillTemplateEntry) return;
    const newId = buildUniqueSkillId(skillTemplateEntry.id);
    createSkillMut.mutate({
      id: newId,
      name: skillTemplateEntry.name,
      content: skillTemplateEntry.skill_template ?? `# ${skillTemplateEntry.name}\n\n`,
    });
  };

  const handleInstallSkillFromSource = () => {
    const parsed = parseSkillSourceInstallSpec(skillSourceInput);
    const source = parsed.source.trim();
    const rawId = skillSourceIdInput.trim() || parsed.inferredSkillId;
    const skillId = slugifyId(rawId);
    if (!source || !skillId) return;
    installSkillFromSourceMut.mutate({
      source,
      skillId,
      gitRef: parsed.gitRef,
      repoPath: parsed.repoPath,
    });
  };

  const handleSetShipGenerationProvider = (providerId: string) => {
    const updated = {
      ...activeAgentConfig,
      ai: {
        ...normalizeAiConfig(activeAgentConfig.ai),
        provider: providerId,
        model:
          (activeAgentConfig.ai?.provider ?? 'claude') === providerId
            ? activeAgentConfig.ai?.model ?? null
            : null,
      },
    };
    updateActiveAgentConfig(updated);
    if (agentScope === 'project') void onSaveProject(updated);
    else void onSaveGlobalAgentConfig(updated);
  };

  const handleAddModePreset = (preset: ModePresetDefinition) => {
    const existing = activeAgentConfig.modes ?? [];
    if (existing.some((mode) => mode.id === preset.id)) {
      handleSetActiveMode(preset.id);
      return;
    }
    const matching = modePresetMatches[preset.id] ?? { skillIds: [], mcpServerIds: [] };
    const availableSkillIds = new Set(skills.map((skill) => skill.id));
    const availableMcpServerIds = new Set(
      (activeAgentConfig.mcp_servers ?? [])
        .map((server) => (server.id ?? '').trim())
        .filter(Boolean)
    );
    const presetSkillIds = preset.default_skill_ids.filter((id) => availableSkillIds.has(id));
    const presetMcpServerIds = preset.default_mcp_server_ids.filter((id) => availableMcpServerIds.has(id));
    const resolvedSkillIds = Array.from(new Set([...presetSkillIds, ...matching.skillIds]));
    const resolvedMcpServerIds = Array.from(new Set([...presetMcpServerIds, ...matching.mcpServerIds]));
    const promptId = preset.default_prompt_id && resolvedSkillIds.includes(preset.default_prompt_id)
      ? preset.default_prompt_id
      : resolvedSkillIds[0] ?? null;
    const modeFromPreset: ModeConfig = {
      id: preset.id,
      name: preset.name,
      description: preset.description,
      active_tools: preset.allow_tools.filter((tool) => !tool.startsWith('mcp__')),
      mcp_servers: resolvedMcpServerIds,
      skills: resolvedSkillIds,
      rules: [],
      prompt_id: promptId,
      hooks: [],
      permissions: {
        allow: preset.allow_tools,
        deny: preset.deny_tools,
      },
      target_agents: preset.target_agents,
    };
    const updated = {
      ...activeAgentConfig,
      modes: [...existing, modeFromPreset],
      active_mode: modeFromPreset.id,
    };
    updateActiveAgentConfig(updated);
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

  const handleImport = async (target: string) => {
    setImportStatus((prev) => ({ ...prev, [target]: 'loading' }));
    setAgentError(null);
    try {
      const res = await commands.importAgentConfigCmd(target, true);
      if (res.status === 'error') throw new Error(res.error);
      const importedMcp = res.data.imported_mcp_servers;
      const importedPermissions = res.data.imported_permissions;
      const summaryParts = [
        importedMcp > 0
          ? `${importedMcp} MCP server${importedMcp === 1 ? '' : 's'} imported`
          : 'No new MCP servers',
      ];
      if (importedPermissions) {
        summaryParts.push('permissions imported');
      }
      setImportSummary((prev) => ({ ...prev, [target]: summaryParts.join(' • ') }));
      setImportStatus((prev) => ({ ...prev, [target]: 'ok' }));
      if (hasActiveProject) {
        const refreshed = await commands.getProjectConfig();
        if (refreshed.status === 'ok') {
          setLocalProject(normalizeProjectConfig(refreshed.data));
        }
      }
      void providersQuery.refetch();
      void mcpValidationQuery.refetch();
    } catch (err) {
      setImportStatus((prev) => ({ ...prev, [target]: 'error' }));
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
    if (!permissions) return;
    const pattern = mcpToolPattern(serverId, toolName);
    const deny = new Set(permissions.tools?.deny ?? []);
    if (deny.has(pattern)) {
      deny.delete(pattern);
    } else {
      deny.add(pattern);
    }
    savePermissionsMut.mutate({
      ...permissions,
      tools: {
        ...permissions.tools,
        allow: permissions.tools?.allow ?? ['*'],
        deny: Array.from(deny),
      },
    });
  };

  const handleToggleServerToolBlock = (serverId: string) => {
    if (!permissions) return;
    const wildcard = `mcp__${serverId}__*`;
    const deny = new Set(permissions.tools?.deny ?? []);
    if (deny.has(wildcard)) {
      deny.delete(wildcard);
    } else {
      deny.add(wildcard);
    }
    savePermissionsMut.mutate({
      ...permissions,
      tools: {
        ...permissions.tools,
        allow: permissions.tools?.allow ?? ['*'],
        deny: Array.from(deny),
      },
    });
  };

  const handleSave = () => {
    if (agentScope === 'global') return onSaveGlobalAgentConfig(localGlobalAgent);
    return onSaveProject(localProject);
  };

  const sectionMeta = SECTION_META[initialSection];

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
        <TooltipContent>
          Edit defaults shared across all projects on this machine.
        </TooltipContent>
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

  return (
    <PageFrame className="md:p-8">
      <PageHeader
        title={sectionMeta.title}
        description={sectionMeta.description}
        badge={<Badge variant="outline">Agents</Badge>}
        actions={scopeToggle}
      />

      <div className="grid gap-4">
        {!hasActiveProject && (
          <Alert className="border-amber-500/30 bg-amber-500/5">
            <AlertDescription className="text-xs text-amber-800 dark:text-amber-200">
              No project is currently selected, so you are editing global defaults. Open or create a project to configure workspace-specific overrides.
            </AlertDescription>
          </Alert>
        )}

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
                    <p className="text-[11px] text-muted-foreground">Ship manages your provider config. Pick one provider to power in-app AI features.</p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        type="button"
                        size="xs"
                        variant="outline"
                        className="h-6 px-2 text-[10px]"
                        onClick={() => void mcpValidationQuery.refetch()}
                        disabled={mcpValidationQuery.isFetching}
                      >
                        {mcpValidationQuery.isFetching ? 'Checking…' : 'Run checks'}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      Refresh provider sync health from current MCP and provider config state.
                    </TooltipContent>
                  </Tooltip>
                  {providersQuery.isPending && (
                    <Badge variant="outline" className="text-[10px] text-muted-foreground">
                      checking PATH...
                    </Badge>
                  )}
                </div>
              </div>
              {providersQuery.isError && (
                <div className="flex items-center justify-between gap-2 border-b bg-rose-500/5 px-4 py-2.5">
                  <p className="text-[11px] text-rose-700 dark:text-rose-300">
                    Provider detection failed. Showing supported providers with unknown install status.
                  </p>
                  <Button
                    type="button"
                    variant="ghost"
                    size="xs"
                    className="h-6 px-2 text-[10px]"
                    onClick={() => void providersQuery.refetch()}
                  >
                    Retry
                  </Button>
                </div>
              )}
              <div className="divide-y divide-border/50">
                {providerRows.map((provider) => {
                  const isEnabled = (activeAgentConfig.providers ?? []).includes(provider.id);
                  const isShipAiProvider = (activeAgentConfig.ai?.provider ?? 'claude') === provider.id;
                  const syncSummary = getProviderSyncSummary(provider, isEnabled, mcpValidationReport);
                  const isExpanded = expandedProviderId === provider.id;
                  const providerHookEvents = HOOK_EVENTS.filter((event) => event.providers.includes(provider.id));
                  const providerIssuePaths = Array.from(
                    new Set(
                      syncSummary.issues
                        .map((issue) => issue.source_path?.trim())
                        .filter((path): path is string => !!path)
                    )
                  );
                  const modelOptions = [
                    ...provider.models.map((model) => ({
                      value: model.id,
                      label: model.recommended ? `${model.name} (recommended)` : model.name,
                    })),
                  ];
                  const currentModel = activeAgentConfig.ai?.model?.trim();
                  if (
                    isShipAiProvider &&
                    currentModel &&
                    !modelOptions.some((option) => option.value === currentModel)
                  ) {
                    modelOptions.unshift({ value: currentModel, label: `${currentModel} (current)` });
                  }
                  const exportDisabledReason =
                    !hasActiveProject
                      ? 'Open or create a project to export provider config files.'
                      : !isEnabled
                        ? 'Enable provider sync in this scope before exporting.'
                        : 'Push Ship unified config to the provider native config file.';
                  const importDisabledReason =
                    !hasActiveProject
                      ? 'Open or create a project to import provider config files.'
                      : agentScope !== 'project'
                        ? 'Switch to Project scope to import into this project.'
                        : 'Import provider-native config into Ship (project file first, then global fallback).';
                  const logo = PROVIDER_LOGO[provider.id];
                  return (
                    <div key={provider.id} className={cn('transition-colors', isExpanded && 'bg-muted/30')}>
                      <div className="flex items-start gap-3 px-4 py-3">
                        <div className="flex size-8 shrink-0 items-center justify-center rounded-lg border bg-card">
                          {logo
                            ? <img src={logo.src} alt={provider.name} className={cn('size-5 object-contain', logo.invertDark && 'dark:invert')} />
                            : <Bot className="size-4 text-muted-foreground" />
                          }
                        </div>

                        <div className="min-w-0 flex-1 space-y-1">
                          <div className="flex flex-wrap items-center gap-1.5">
                            <p className="text-sm font-medium">{provider.name}</p>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Badge variant="outline" className={cn('cursor-default text-[10px]', providerStatusBadgeClass(syncSummary.status))}>
                                  {PROVIDER_STATUS_COPY[syncSummary.status]}
                                </Badge>
                              </TooltipTrigger>
                              <TooltipContent className="max-w-xs">
                                Provider sync health for this scope: `Ready` means safe to sync, `Needs attention` means blocking issues were detected, `Drift detected` means provider config shape diverges from Ship expectations.
                              </TooltipContent>
                            </Tooltip>
                            {provider.checking ? (
                              <Badge variant="outline" className="cursor-default text-[10px] text-muted-foreground">
                                checking...
                              </Badge>
                            ) : provider.installed ? (
                              <Badge variant="outline" className="cursor-default text-[10px] text-muted-foreground">
                                {provider.version ?? 'installed'}
                              </Badge>
                            ) : (
                              <Badge variant="outline" className="cursor-default text-[10px] text-muted-foreground">
                                not found
                              </Badge>
                            )}
                          </div>
                          <p className="font-mono text-[11px] text-muted-foreground">{provider.binary}</p>
                          <p className="text-[11px] text-muted-foreground">{syncSummary.detail}</p>
                          {importSummary[provider.id] && (
                            <p className="text-[11px] text-muted-foreground">{importSummary[provider.id]}</p>
                          )}
                        </div>

                        <div className="flex shrink-0 items-center gap-1">
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                type="button"
                                variant="outline"
                                size="xs"
                                disabled={!hasActiveProject || agentScope !== 'project' || importStatus[provider.id] === 'loading'}
                                onClick={() => void handleImport(provider.id)}
                                className="h-6 px-2 text-[10px]"
                              >
                                <Download className="mr-1 size-3" />
                                {importStatus[provider.id] === 'loading' ? 'Importing…'
                                  : importStatus[provider.id] === 'ok' ? 'Imported ✓'
                                  : 'Import'}
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                              {importDisabledReason}
                            </TooltipContent>
                          </Tooltip>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                type="button"
                                variant={isShipAiProvider ? 'secondary' : 'ghost'}
                                size="xs"
                                className={cn(
                                  'h-6 w-6 p-0',
                                  isShipAiProvider && 'bg-primary/10 text-primary'
                                )}
                                onClick={() => handleSetShipGenerationProvider(provider.id)}
                              >
                                <Zap className="size-3.5" />
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                              {isShipAiProvider
                                ? 'Currently powering in-app AI features in Ship'
                                : 'Use this provider to power AI features in Ship'}
                            </TooltipContent>
                          </Tooltip>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                type="button"
                                variant={isEnabled ? 'secondary' : 'outline'}
                                size="xs"
                                className="h-6 px-2 text-[10px]"
                                onClick={() => handleToggleProvider(provider.id, isEnabled)}
                              >
                                {isEnabled ? 'Sync On' : 'Sync Off'}
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                              {isEnabled
                                ? 'Disable provider export for this scope'
                                : 'Enable provider export for this scope'}
                            </TooltipContent>
                          </Tooltip>
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
                              {exportDisabledReason}
                            </TooltipContent>
                          </Tooltip>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                type="button"
                                variant="ghost"
                                size="xs"
                                className="h-6 w-6 p-0"
                                onClick={() => setExpandedProviderId(isExpanded ? '' : provider.id)}
                              >
                                {isExpanded
                                  ? <ChevronDown className="size-3.5" />
                                  : <ChevronRight className="size-3.5" />
                                }
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                              {isExpanded ? 'Hide advanced settings' : 'Show advanced settings'}
                            </TooltipContent>
                          </Tooltip>
                        </div>
                      </div>

                      {isExpanded && (
                        <div className="space-y-3 border-t bg-muted/20 px-4 py-4">
                          <div className="space-y-1.5">
                            <div className="flex items-center gap-1.5">
                              <Label className="text-xs">Config Paths</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Project/global paths come from Ship's provider registry. Detected paths come from runtime preflight checks.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <div className="flex flex-wrap gap-1.5">
                              {provider.project_config && (
                                <Badge variant="outline" className="font-mono text-[10px]">
                                  {`<project>/${provider.project_config}`}
                                </Badge>
                              )}
                              {provider.global_config && (
                                <Badge variant="outline" className="font-mono text-[10px]">
                                  {`~/${provider.global_config}`}
                                </Badge>
                              )}
                              {providerIssuePaths.map((path) => (
                                <Badge key={path} variant="outline" className="font-mono text-[10px]">
                                  {path}
                                </Badge>
                              ))}
                            </div>
                          </div>

                          <div className="space-y-1.5">
                            <div className="flex items-center gap-1.5">
                              <Label className="text-xs">Import / Export Resolution</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Import checks project config first, then falls back to global. Export always writes provider-native files based on provider conventions.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <p className="text-[11px] text-muted-foreground">
                              Import order: <span className="font-mono">{`<project>/${provider.project_config}`}</span> then <span className="font-mono">{`~/${provider.global_config}`}</span>
                            </p>
                            <p className="text-[11px] text-muted-foreground">
                              Export MCP target: <span className="font-mono">{`<project>/${provider.project_config}`}</span>
                            </p>
                          </div>

                          <div className="grid gap-3 sm:grid-cols-[1fr_1fr]">
                            <div className="space-y-1.5">
                              <div className="flex items-center gap-1.5">
                                <Label className="text-xs">In-App AI Model</Label>
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Info className="size-3 cursor-default text-muted-foreground" />
                                  </TooltipTrigger>
                                  <TooltipContent>
                                    {isShipAiProvider
                                      ? 'Model used for Ship AI features with this provider.'
                                      : 'Set this provider as Ship AI first to edit its model.'}
                                  </TooltipContent>
                                </Tooltip>
                              </div>
                              <AutocompleteInput
                                value={isShipAiProvider ? (activeAgentConfig.ai?.model ?? '') : ''}
                                options={modelOptions}
                                placeholder={isShipAiProvider ? 'Select or type a model ID' : 'Select this provider for Ship AI'}
                                noResultsText={isShipAiProvider ? 'No detected model IDs yet. You can still type one.' : 'Switch provider first.'}
                                disabled={!isShipAiProvider}
                                onValueChange={(value) => {
                                  if (!isShipAiProvider) return;
                                  updateActiveAgentConfig({
                                    ...activeAgentConfig,
                                    ai: { ...normalizeAiConfig(activeAgentConfig.ai), model: value || null },
                                  });
                                }}
                              />
                            </div>

                            <div className="space-y-1.5">
                              <div className="flex items-center gap-1.5">
                                <Label className="text-xs">CLI Path Override</Label>
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Info className="size-3 cursor-default text-muted-foreground" />
                                  </TooltipTrigger>
                                  <TooltipContent>
                                    {isShipAiProvider
                                      ? 'Optional absolute path to this provider binary. Blank uses PATH.'
                                      : 'Set this provider as Ship AI first to set CLI override.'}
                                  </TooltipContent>
                                </Tooltip>
                              </div>
                              <Input
                                value={isShipAiProvider ? (activeAgentConfig.ai?.cli_path ?? '') : ''}
                                disabled={!isShipAiProvider}
                                onChange={(event) => {
                                  if (!isShipAiProvider) return;
                                  updateActiveAgentConfig({
                                    ...activeAgentConfig,
                                    ai: { ...normalizeAiConfig(activeAgentConfig.ai), cli_path: event.target.value || null },
                                  });
                                }}
                                placeholder={isShipAiProvider ? 'Optional absolute path' : 'Select this provider for Ship AI'}
                                className="h-8 text-xs"
                              />
                            </div>
                          </div>

                          <div className="space-y-1.5">
                            <div className="flex items-center gap-1.5">
                              <Label className="text-xs">Hook Surface</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent>
                                  Hook support by provider. Ship stores hooks centrally and only exports native events where supported.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            {providerHookEvents.length > 0 ? (
                              <div className="flex flex-wrap gap-1.5">
                                {providerHookEvents.map((event) => (
                                  <Badge key={`${provider.id}-${event.value}`} variant="secondary" className="text-[10px]">
                                    {event.label}
                                  </Badge>
                                ))}
                              </div>
                            ) : (
                              <p className="text-[11px] text-muted-foreground">
                                No native hook export support for this provider yet.
                              </p>
                            )}
                          </div>

                          {syncSummary.issues.length > 0 && (
                            <div className="space-y-2 rounded-md border bg-background/80 p-2.5">
                              <p className="text-[11px] font-medium">
                                Diagnostics ({syncSummary.issues.length})
                              </p>
                              <div className="max-h-40 space-y-1 overflow-auto pr-1">
                                {syncSummary.issues.map((issue, idx) => (
                                  <div
                                    key={`${provider.id}-${issue.code}-${issue.server_id ?? issue.provider_id ?? idx}`}
                                    className={cn(
                                      'rounded border px-2 py-1.5 text-[11px]',
                                      issue.level === 'error'
                                        ? 'border-rose-500/30 bg-rose-500/5 text-rose-700 dark:text-rose-300'
                                        : issue.level === 'warning'
                                          ? 'border-amber-500/30 bg-amber-500/5 text-amber-700 dark:text-amber-300'
                                          : 'border-border/60 bg-muted/30 text-muted-foreground'
                                    )}
                                  >
                                    <p className="font-medium">
                                      {issue.level.toUpperCase()} • {issue.code}
                                    </p>
                                    <p>{issue.message}</p>
                                    {issue.hint && <p className="opacity-90">Hint: {issue.hint}</p>}
                                  </div>
                                ))}
                              </div>
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </Card>

            {/* ── Templates ── */}
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center justify-between border-b px-4 py-3">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="text-sm font-semibold">Agent Templates</h3>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent className="max-w-xs">
                        Templates are Ship opinionated starting points. Under the hood they map to mode config with skill, MCP, and tool-policy defaults.
                      </TooltipContent>
                    </Tooltip>
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    Start from a template, then tailor it per workspace/service.
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
                    <p className="text-sm text-muted-foreground">No templates defined yet.</p>
                    <p className="text-[11px] text-muted-foreground/60">Add a template to establish a focused starting environment for this workspace.</p>
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
                              <TooltipContent>Use this template for agent sessions in this scope</TooltipContent>
                            </Tooltip>
                          )}
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                variant="ghost"
                                size="xs"
                                className="h-6 w-6 p-0 text-destructive hover:bg-destructive/10 hover:text-destructive"
                                onClick={(e) => { e.stopPropagation(); handleRemoveMode(mode.id); }}
                              >
                                <Trash2 className="size-3.5" />
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent>Delete mode</TooltipContent>
                          </Tooltip>
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
                              <Label className="text-xs">Template Name</Label>
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
                                placeholder="What this template is for"
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

              <div className="space-y-3 border-t px-4 py-3">
                <div className="space-y-2">
                  <div className="flex items-center gap-1.5">
                    <Label className="text-xs">Start From Ship Templates</Label>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent>
                        Templates include target agents, tool policy defaults, and auto-linking to matching skills/MCP in this workspace.
                      </TooltipContent>
                    </Tooltip>
                  </div>
                  <div className="grid gap-2 sm:grid-cols-3">
                    {MODE_PRESETS.map((preset) => {
                      const exists = (activeAgentConfig.modes ?? []).some((mode) => mode.id === preset.id);
                      const matching = modePresetMatches[preset.id] ?? { skillIds: [], mcpServerIds: [] };
                      return (
                        <Button
                          key={preset.id}
                          type="button"
                          variant={exists ? 'secondary' : 'outline'}
                          size="sm"
                          disabled={exists}
                          className="h-auto flex-col items-start px-3 py-2 text-left"
                          onClick={() => handleAddModePreset(preset)}
                        >
                          <span className="text-xs font-semibold">{preset.name}</span>
                          <span className="mt-0.5 line-clamp-2 text-[10px] font-normal text-muted-foreground">
                            {preset.description}
                          </span>
                          <span className="mt-1 text-[10px] font-normal text-muted-foreground/80">
                            Agents: {preset.target_agents.join(', ')}
                          </span>
                          <span className="text-[10px] font-normal text-muted-foreground/80">
                            Auto-links: {matching.skillIds.length} skill{matching.skillIds.length === 1 ? '' : 's'}, {matching.mcpServerIds.length} MCP
                          </span>
                        </Button>
                      );
                    })}
                  </div>
                </div>

                <div className="space-y-1.5">
                  <Label className="text-xs">Create Custom Template</Label>
                  <div className="flex items-center gap-2">
                    <Input
                      value={newMode.name}
                      onChange={(e) => setNewMode({
                        ...newMode,
                        name: e.target.value,
                        id: e.target.value.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, ''),
                      })}
                      placeholder="New template name…"
                      className="h-8 text-xs"
                      onKeyDown={(e) => e.key === 'Enter' && newMode.name.trim() && handleAddMode()}
                    />
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3.5 shrink-0 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent>
                        Template ID is inferred automatically from this name.
                      </TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          size="sm"
                          onClick={handleAddMode}
                          disabled={!newMode.name.trim()}
                          className="shrink-0"
                        >
                          <Plus className="mr-1 size-3.5" />Add
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>Create a custom template</TooltipContent>
                    </Tooltip>
                  </div>
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
            {MCP_STDIO_ONLY_ALPHA && (
              <Alert className="border-amber-500/30 bg-amber-500/5">
                <AlertDescription className="text-xs text-amber-800 dark:text-amber-200">
                  Alpha note: MCP server execution is currently stdio-only. HTTP/SSE entries can be discovered but not configured for active use yet.
                </AlertDescription>
              </Alert>
            )}
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

              <div className="border-b bg-muted/20 px-4 py-3">
                <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto_auto_auto]">
                  <div className="flex items-center gap-1.5">
                    <AutocompleteInput
                      value={mcpCatalogInput}
                      options={allMcpOptions}
                      placeholder="Search MCP library templates..."
                      onValueChange={setMcpCatalogInput}
                      className="h-8 text-xs"
                    />
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3.5 shrink-0 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent>
                        Search local templates plus official MCP registry results.
                      </TooltipContent>
                    </Tooltip>
                  </div>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="h-8"
                        onClick={handleApplyMcpTemplate}
                        disabled={!mcpTemplateEntry && !mcpRegistryTemplateEntry}
                      >
                        <Plus className="mr-1.5 size-3.5" />
                        {mcpTemplateEntry ? 'Use Template' : 'Use Registry'}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      Add the selected MCP definition and auto-fill recommended defaults.
                    </TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        type="button"
                        size="sm"
                        variant="secondary"
                        className="h-8"
                        onClick={() => void mcpValidationQuery.refetch()}
                        disabled={mcpValidationQuery.isFetching}
                      >
                        {mcpValidationQuery.isFetching ? 'Validating…' : 'Validate MCP'}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      Run preflight checks on server definitions and provider config files.
                    </TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="h-8"
                        onClick={() => void mcpProbeQuery.refetch()}
                        disabled={(activeAgentConfig.mcp_servers ?? []).length === 0 || mcpProbeQuery.isFetching}
                      >
                        <Wrench className="mr-1.5 size-3.5" />
                        {mcpProbeQuery.isFetching ? 'Probing…' : 'Probe Tools'}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      Start each configured server and discover available MCP tools where supported.
                    </TooltipContent>
                  </Tooltip>
                </div>
                {(mcpTemplateEntry || mcpRegistryTemplateEntry) && (
                  <div className="mt-1.5 flex items-center gap-2 text-[11px] text-muted-foreground">
                    {mcpTemplateEntry && (
                      <span>{mcpTemplateEntry.name}: {mcpTemplateEntry.description}</span>
                    )}
                    {!mcpTemplateEntry && mcpRegistryTemplateEntry && (
                      <span>
                        {mcpRegistryTemplateEntry.title} ({mcpRegistryTemplateEntry.transport}, v{mcpRegistryTemplateEntry.version}): {mcpRegistryTemplateEntry.description}
                      </span>
                    )}
                    {(mcpTemplateEntry?.source_url ?? mcpRegistryTemplateEntry?.source_url ?? mcpRegistryTemplateEntry?.website_url) && (
                      <a
                        href={mcpTemplateEntry?.source_url ?? mcpRegistryTemplateEntry?.source_url ?? mcpRegistryTemplateEntry?.website_url ?? '#'}
                        target="_blank"
                        rel="noreferrer"
                        className="inline-flex items-center gap-1 text-primary hover:underline"
                      >
                        <Link className="size-3" />
                        source
                      </a>
                    )}
                  </div>
                )}
                {initialSection === 'mcp' && (
                  <div className="mt-2 space-y-1">
                    <div className="flex items-center justify-between text-[11px] text-muted-foreground">
                      <span>Official Registry matches</span>
                      {mcpRegistryQuery.isFetching && <span>searching…</span>}
                    </div>
                    {mcpRegistryQuery.isError ? (
                      <p className="text-[11px] text-amber-700 dark:text-amber-300">
                        Registry discovery unavailable right now. You can still configure servers manually.
                      </p>
                    ) : mcpRegistryEntries.length === 0 ? (
                      <p className="text-[11px] text-muted-foreground">No registry matches.</p>
                    ) : (
                      <div className="space-y-1">
                        {mcpRegistryEntries.slice(0, 5).map((entry) => (
                          <div key={entry.server_name} className="flex items-center justify-between gap-2 rounded border bg-background/70 px-2 py-1.5 text-[11px]">
                            <div className="min-w-0">
                              <p className="truncate font-medium">{entry.title}</p>
                              <p className="truncate text-muted-foreground">
                                {entry.server_name} • {entry.transport} • v{entry.version}
                              </p>
                              {(entry.required_env.length > 0 || entry.required_headers.length > 0) && (
                                <p className="truncate text-amber-700 dark:text-amber-300">
                                  Requires: {entry.required_env.length > 0 ? `${entry.required_env.length} env` : ''}{entry.required_env.length > 0 && entry.required_headers.length > 0 ? ', ' : ''}{entry.required_headers.length > 0 ? `${entry.required_headers.length} headers` : ''}
                                </p>
                              )}
                            </div>
                            <Button
                              type="button"
                              size="xs"
                              variant="outline"
                              className="h-6 px-2 text-[10px]"
                              onClick={() => handleInstallRegistryEntry(entry)}
                            >
                              Install
                            </Button>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}
                {mcpValidationQuery.isError && (
                  <p className="mt-1.5 text-[11px] text-destructive">
                    {String(mcpValidationQuery.error)}
                  </p>
                )}
                {mcpValidationReport && (
                  <div className="mt-2 space-y-2 rounded-md border bg-background/70 p-2.5">
                    <div className="flex items-center justify-between gap-2 text-[11px]">
                      <span className="font-medium">
                        Preflight: {mcpValidationReport.ok ? 'ready' : 'needs attention'}
                      </span>
                      <span className="text-muted-foreground">
                        {mcpValidationReport.checked_servers} servers, {mcpValidationReport.checked_provider_configs} provider config files
                      </span>
                    </div>
                    {mcpValidationReport.issues.length === 0 ? (
                      <p className="text-[11px] text-emerald-600 dark:text-emerald-400">
                        No issues found.
                      </p>
                    ) : (
                      <div className="max-h-40 space-y-1 overflow-auto pr-1">
                        {mcpValidationReport.issues.map((issue, idx) => (
                          <div
                            key={`${issue.code}-${issue.server_id ?? issue.provider_id ?? idx}`}
                            className={cn(
                              'rounded border px-2 py-1.5 text-[11px]',
                              issue.level === 'error'
                                ? 'border-rose-500/30 bg-rose-500/5 text-rose-700 dark:text-rose-300'
                                : issue.level === 'warning'
                                  ? 'border-amber-500/30 bg-amber-500/5 text-amber-700 dark:text-amber-300'
                                  : 'border-border/60 bg-muted/30 text-muted-foreground'
                            )}
                          >
                            <p className="font-medium">
                              {issue.level.toUpperCase()} {issue.server_id ? `• ${issue.server_id}` : issue.provider_id ? `• ${issue.provider_id}` : ''}
                            </p>
                            <p>{issue.message}</p>
                            {issue.hint && <p className="opacity-90">Hint: {issue.hint}</p>}
                            {issue.source_path && <p className="font-mono opacity-80">{issue.source_path}</p>}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}
                {mcpProbeQuery.isError && (
                  <p className="mt-1.5 text-[11px] text-destructive">
                    {String(mcpProbeQuery.error)}
                  </p>
                )}
                {mcpProbeReport && (
                  <div className="mt-2 space-y-2 rounded-md border bg-background/70 p-2.5">
                    <div className="flex flex-wrap items-center justify-between gap-2 text-[11px]">
                      <span className="font-medium">
                        Runtime probe: {mcpProbeReport.reachable_servers}/{mcpProbeReport.checked_servers} reachable • {mcpProbeReport.discovered_tools} tool{mcpProbeReport.discovered_tools === 1 ? '' : 's'}
                      </span>
                      <span className="text-muted-foreground">
                        {formatEpochSeconds(mcpProbeReport.generated_at)}
                      </span>
                    </div>
                    {mcpProbeReport.results.length === 0 ? (
                      <p className="text-[11px] text-muted-foreground">No probe results yet.</p>
                    ) : (
                      <div className="max-h-40 space-y-1 overflow-auto pr-1">
                        {mcpProbeReport.results.map((result) => (
                          <div key={`probe-${result.server_id}`} className="rounded border bg-muted/30 px-2 py-1.5 text-[11px]">
                            <div className="flex items-center justify-between gap-2">
                              <p className="font-medium">{result.server_name || result.server_id}</p>
                              <Badge
                                variant="outline"
                                className={cn(
                                  'cursor-default text-[10px]',
                                  result.status === 'ready'
                                    ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                                    : result.status === 'partial'
                                      ? 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300'
                                      : result.status === 'disabled'
                                        ? 'text-muted-foreground'
                                        : 'border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-300'
                                )}
                              >
                                {result.status}
                              </Badge>
                            </div>
                            <p className="text-muted-foreground">
                              {result.transport} • {result.discovered_tools.length} tool{result.discovered_tools.length === 1 ? '' : 's'} • {result.duration_ms}ms
                            </p>
                            {result.message && <p>{result.message}</p>}
                            {result.warnings.slice(0, 1).map((warning) => (
                              <p key={warning} className="text-amber-700 dark:text-amber-300">{warning}</p>
                            ))}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}
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
                  const serverId = (server.id ?? server.name).trim();
                  const normalizedServerId = serverId || inferMcpServerId(server);
                  const isEditing = mcpEditDraft?.idx === idx;
                  const transport = server.server_type ?? 'stdio';
                  const envCount = server.env ? Object.keys(server.env).length : 0;
                  const probeResult = mcpProbeByServerId.get(normalizedServerId);
                  const discoveredTools = (() => {
                    const probeTools = probeResult?.discovered_tools ?? [];
                    const cachedTools = cachedMcpToolsByServerId.get(normalizedServerId) ?? [];
                    const byName = new Map<string, { name: string; description?: string | null }>();
                    for (const tool of [...cachedTools, ...probeTools]) {
                      if (!tool?.name) continue;
                      byName.set(tool.name, {
                        name: tool.name,
                        description: tool.description ?? byName.get(tool.name)?.description ?? null,
                      });
                    }
                    return Array.from(byName.values());
                  })();
                  const allToolsBlocked = isMcpServerDenied(permissions, normalizedServerId);
                  return (
                    <div key={`${normalizedServerId}-${idx}`} className={cn('transition-colors', isEditing && 'bg-muted/30')}>
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
                            {probeResult && (
                              <Badge
                                variant="outline"
                                className={cn(
                                  'cursor-default px-1.5 py-0 text-[9px]',
                                  probeResult.status === 'ready'
                                    ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                                    : probeResult.status === 'partial'
                                      ? 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300'
                                      : probeResult.status === 'disabled'
                                        ? 'text-muted-foreground'
                                        : 'border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-300'
                                )}
                              >
                                {probeResult.status}
                              </Badge>
                            )}
                            {discoveredTools.length > 0 && (
                              <Badge variant="secondary" className="cursor-default px-1.5 py-0 text-[9px]">
                                {discoveredTools.length} tools
                              </Badge>
                            )}
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
                          {probeResult?.message && (
                            <p className="truncate text-[10px] text-muted-foreground">
                              {probeResult.message}
                            </p>
                          )}
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
                          {discoveredTools.length > 0 && (
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  variant={allToolsBlocked ? 'secondary' : 'outline'}
                                  size="xs"
                                  className="h-6 px-2 text-[10px]"
                                  onClick={() => handleToggleServerToolBlock(normalizedServerId)}
                                  disabled={!permissions || savePermissionsMut.isPending}
                                >
                                  {allToolsBlocked ? 'All Blocked' : 'Block All'}
                                </Button>
                              </TooltipTrigger>
                              <TooltipContent>
                                {allToolsBlocked
                                  ? `Allow all discovered tools for ${normalizedServerId}`
                                  : `Block all discovered tools for ${normalizedServerId}`}
                              </TooltipContent>
                            </Tooltip>
                          )}
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

                      {discoveredTools.length > 0 && (
                        <div className="border-t bg-background/40 px-4 py-2">
                          <div className="flex items-center justify-between gap-2">
                            <p className="text-[11px] font-medium">Discovered Tools</p>
                            <p className="text-[10px] text-muted-foreground">
                              click to block or allow
                            </p>
                          </div>
                          <div className="mt-1.5 flex flex-wrap gap-1">
                            {discoveredTools.slice(0, 24).map((tool) => {
                              const denied = isMcpToolDenied(permissions, normalizedServerId, tool.name);
                              const pattern = mcpToolPattern(normalizedServerId, tool.name);
                              return (
                                <Tooltip key={`${normalizedServerId}-${tool.name}`}>
                                  <TooltipTrigger asChild>
                                    <button
                                      type="button"
                                      className={cn(
                                        'rounded border px-1.5 py-0.5 text-[10px] font-mono transition-colors',
                                        denied
                                          ? 'border-rose-500/40 bg-rose-500/10 text-rose-700 dark:text-rose-300'
                                          : 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                                      )}
                                      onClick={() => handleToggleDiscoveredToolPolicy(normalizedServerId, tool.name)}
                                      disabled={!permissions || savePermissionsMut.isPending}
                                    >
                                      {tool.name}
                                    </button>
                                  </TooltipTrigger>
                                  <TooltipContent className="max-w-xs">
                                    <p>{denied ? 'Blocked by deny list' : 'Allowed (not denied)'}</p>
                                    <p className="font-mono text-[10px]">{pattern}</p>
                                    {tool.description && (
                                      <p className="mt-1 text-[10px] text-muted-foreground">{tool.description}</p>
                                    )}
                                  </TooltipContent>
                                </Tooltip>
                              );
                            })}
                            {discoveredTools.length > 24 && (
                              <Badge variant="outline" className="text-[10px]">
                                +{discoveredTools.length - 24} more
                              </Badge>
                            )}
                          </div>
                        </div>
                      )}

                      {isEditing && mcpEditDraft && (
                        <McpServerForm
                          draft={mcpEditDraft.server}
                          onChange={(s) => setMcpEditDraft({ ...mcpEditDraft, server: s })}
                          onSave={handleSaveMcpServer}
                          onCancel={() => setMcpEditDraft(null)}
                          idOptions={mcpIdOptions}
                          commandOptions={mcpCommandOptions}
                          envKeyOptions={mcpEnvKeyOptions}
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
                    idOptions={mcpIdOptions}
                    commandOptions={mcpCommandOptions}
                    envKeyOptions={mcpEnvKeyOptions}
                    isNew
                  />
                )}
              </div>

              {mcpEditDraft === null && (
                <div className="border-t px-4 py-3">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full border-dashed"
                        onClick={() => setMcpEditDraft({ idx: null, server: { ...EMPTY_MCP_SERVER } })}
                      >
                        <Plus className="mr-1.5 size-3.5" />
                        Add MCP Server
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      Configure a custom MCP server connection.
                    </TooltipContent>
                  </Tooltip>
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
                    Studio mode now renders each skill as an auditable folder with package metadata while you edit in real time.
                  </p>
                  <p className="text-muted-foreground">
                    Switch between folder-audit studio and compact list view as needed.
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
                <CardContent className="space-y-3 !pt-5">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button variant="outline" size="sm" className="w-full" onClick={() => handleCreateDoc(activeDocKind)}>
                        <Plus className="size-3.5" />
                        New {initialSection === 'skills' ? 'Skill' : 'Rule'}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      Create a new {initialSection === 'skills' ? 'skill package' : 'rule document'} in {agentScope} scope.
                    </TooltipContent>
                  </Tooltip>

                  {initialSection === 'skills' && (
                    <div>
                      <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto]">
                        <div className="flex items-center gap-1.5">
                          <AutocompleteInput
                            value={skillCatalogInput}
                            options={catalogSkillOptions}
                            placeholder="Install skill from library..."
                            onValueChange={setSkillCatalogInput}
                            className="h-8 text-xs"
                          />
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Info className="size-3.5 shrink-0 cursor-default text-muted-foreground" />
                            </TooltipTrigger>
                            <TooltipContent>
                              Search curated skills by name, ID, or keywords.
                            </TooltipContent>
                          </Tooltip>
                        </div>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              type="button"
                              size="sm"
                              variant="outline"
                              className="h-8"
                              onClick={handleApplySkillTemplate}
                              disabled={!skillTemplateEntry}
                            >
                              <Plus className="mr-1.5 size-3.5" />
                              Install
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>
                            Install the selected skill package from the catalog.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      {skillTemplateEntry && (
                        <div className="mt-1.5 flex items-center gap-2 text-[11px] text-muted-foreground">
                          <span>{skillTemplateEntry.name}: {skillTemplateEntry.description}</span>
                          {skillTemplateEntry.source_url && (
                            <a
                              href={skillTemplateEntry.source_url}
                              target="_blank"
                              rel="noreferrer"
                              className="inline-flex items-center gap-1 text-primary hover:underline"
                            >
                              <Link className="size-3" />
                              source
                            </a>
                          )}
                        </div>
                      )}
                      <div className="mt-2 grid gap-2 sm:grid-cols-[minmax(0,1fr)_130px_auto]">
                        <AutocompleteInput
                          value={skillSourceInput}
                          options={catalogSkillSourceOptions}
                          placeholder="Install skill from URL or repo path..."
                          onValueChange={setSkillSourceInput}
                          className="h-8 text-xs"
                        />
                        <Input
                          value={skillSourceIdInput}
                          onChange={(event) => setSkillSourceIdInput(event.target.value)}
                          placeholder={inferredSkillSourceId || 'skill-id'}
                          className="h-8 text-xs font-mono"
                        />
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              type="button"
                              size="sm"
                              variant="outline"
                              className="h-8"
                              onClick={handleInstallSkillFromSource}
                              disabled={!skillSourceInput.trim() || !(skillSourceIdInput.trim() || inferredSkillSourceId) || installSkillFromSourceMut.isPending}
                            >
                              <Upload className="mr-1.5 size-3.5" />
                              {installSkillFromSourceMut.isPending ? 'Installing…' : 'Install URL'}
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>
                            Install a skill from GitHub repo/tree URL, GitHub SSH URL, or local repo path.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      {installSkillFromSourceMut.isError && (
                        <p className="mt-1 text-[11px] text-destructive">
                          {String(installSkillFromSourceMut.error)}
                        </p>
                      )}
                    </div>
                  )}

                  {initialSection === 'skills' && (
                    <div className="flex items-center justify-between rounded-md border bg-muted/40 px-2.5 py-2">
                      <div className="flex items-center gap-2">
                        <Folder className="size-3.5 text-cyan-500" />
                        <div>
                          <p className="text-[11px] font-medium leading-tight">Studio folder audit</p>
                          <p className="text-[10px] text-muted-foreground">{skillScopeRoot}</p>
                        </div>
                      </div>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            type="button"
                            variant="ghost"
                            size="xs"
                            onClick={() => setSkillStudioMode((current) => !current)}
                          >
                            {skillStudioMode ? 'Use list' : 'Use studio'}
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          Switch between folder-audit tree and compact list view.
                        </TooltipContent>
                      </Tooltip>
                    </div>
                  )}

                  <div className="max-h-[500px] space-y-2 overflow-auto pr-1">
                    {activeDocs.length === 0 && (
                      <p className="py-4 text-center text-xs text-muted-foreground">
                        No {initialSection === 'skills' ? 'skills' : 'rules'} yet.
                      </p>
                    )}

                    {initialSection === 'skills' && skillStudioMode
                      ? (
                        <FileTree
                          className="border-border/60 bg-card/40 text-xs"
                          expanded={skillTreeExpanded}
                          onExpandedChange={(expanded) => setSkillTreeExpanded(new Set(expanded))}
                          selectedPath={activeDoc ? `${skillScopeRoot}/${activeDoc.id}/SKILL.md` : undefined}
                          onSelect={handleSkillTreeSelect}
                        >
                          <FileTreeFolder
                            path={skillScopeRoot}
                            name={agentScope === 'project' ? 'project-skills' : 'user-skills'}
                          >
                            {activeDocs.map((doc) => {
                              const skillPath = `${skillScopeRoot}/${doc.id}`;
                              const frontmatterPresent = hasYamlFrontmatter(doc.content);
                              const argumentPlaceholder = doc.content.includes('$ARGUMENTS');
                              return (
                                <FileTreeFolder key={doc.id} path={skillPath} name={doc.id}>
                                  <FileTreeFile path={`${skillPath}/SKILL.md`} name="SKILL.md" />
                                  <FileTreeFolder path={`${skillPath}/audit`} name="audit">
                                    <FileTreeFile
                                      path={`${skillPath}/audit/source`}
                                      name={`source:${sourceLabel(doc.source)}`}
                                      icon={<Package className="size-3 text-muted-foreground" />}
                                    />
                                    {doc.version ? (
                                      <FileTreeFile
                                        path={`${skillPath}/audit/version`}
                                        name={`version:${doc.version}`}
                                        icon={<Check className="size-3 text-muted-foreground" />}
                                      />
                                    ) : null}
                                    {doc.author ? (
                                      <FileTreeFile
                                        path={`${skillPath}/audit/author`}
                                        name={`author:${doc.author}`}
                                        icon={<Check className="size-3 text-muted-foreground" />}
                                      />
                                    ) : null}
                                    <FileTreeFile
                                      path={`${skillPath}/audit/frontmatter`}
                                      name={frontmatterPresent ? "frontmatter:ok" : "frontmatter:missing"}
                                      icon={<Check className="size-3 text-muted-foreground" />}
                                    />
                                    <FileTreeFile
                                      path={`${skillPath}/audit/arguments`}
                                      name={argumentPlaceholder ? "args:enabled" : "args:none"}
                                      icon={<Check className="size-3 text-muted-foreground" />}
                                    />
                                  </FileTreeFolder>
                                </FileTreeFolder>
                              );
                            })}
                          </FileTreeFolder>
                        </FileTree>
                      )
                      : activeDocs.map((doc) => {
                          const selected = activeDoc?.id === doc.id;
                          return (
                            <button
                              key={doc.id}
                              type="button"
                              className={`w-full rounded-md border px-2.5 py-2 text-left transition-colors ${selected ? 'border-primary/40 bg-primary/10' : 'hover:bg-muted/50'}`}
                              onClick={() => selectActiveDoc(activeDocKind, doc.id)}
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
                        ? 'Edit skill content — studio folder audit updates in real time as you type.'
                        : 'Edit rule content — global instructions applied to every session.'}
                    </p>
                  </div>
                  {activeDoc && (
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="xs"
                          className="text-destructive hover:bg-destructive/10 hover:text-destructive"
                          onClick={() => handleDeleteDoc(activeDocKind, activeDoc.id)}
                        >
                          <Trash2 className="mr-1 size-3.5" />
                          Delete
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        Permanently delete this {initialSection === 'skills' ? 'skill' : 'rule'} document.
                      </TooltipContent>
                    </Tooltip>
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
            HOOKS
        ════════════════════════════════════════════════════════════════ */}
        {initialSection === 'hooks' && (
          <div className="grid gap-4 lg:grid-cols-[1fr_320px]">
            <Card size="sm" className="overflow-hidden">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-amber-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-amber-500/20 bg-amber-500/10">
                  <Terminal className="size-3.5 text-amber-500" />
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="text-sm font-semibold">Lifecycle Hooks</h3>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="size-3 cursor-default text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent>
                        Hooks export natively to Claude and Gemini. Codex stores hook config in Ship but has no native hook runtime yet.
                      </TooltipContent>
                    </Tooltip>
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    Run command interceptors at key agent lifecycle moments for context, guardrails, and telemetry.
                  </p>
                </div>
                <Badge variant="secondary" className="shrink-0 text-[10px]">
                  {(activeAgentConfig.hooks ?? []).length} hook{(activeAgentConfig.hooks ?? []).length !== 1 ? 's' : ''}
                </Badge>
              </div>

              <CardContent className="space-y-3 !pt-5">
                {(activeAgentConfig.hooks ?? []).length === 0 && (
                  <div className="rounded-lg border border-dashed p-6 text-center">
                    <p className="text-sm text-muted-foreground">No hooks configured yet.</p>
                    <p className="mt-1 text-[11px] text-muted-foreground/70">
                      Add one to inject context, enforce shell policy, or stream events to ops.
                    </p>
                  </div>
                )}

                {(activeAgentConfig.hooks ?? []).map((hook, idx) => {
                  const triggerValue = String(hook.trigger || defaultHookTrigger);
                  const triggerMeta = HOOK_EVENTS.find((event) => event.value === triggerValue);
                  return (
                    <div key={`${hook.id}-${idx}`} className="space-y-3 rounded-lg border p-3">
                      <div className="grid gap-2 sm:grid-cols-[1fr_180px_auto]">
                        <div className="space-y-1">
                          <div className="flex items-center gap-1.5">
                            <Label className="text-[11px]">Hook ID</Label>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Info className="size-3 cursor-default text-muted-foreground" />
                              </TooltipTrigger>
                              <TooltipContent>Stable ID for this hook in project config and exports.</TooltipContent>
                            </Tooltip>
                          </div>
                          <Input
                            value={hook.id ?? ''}
                            onChange={(e) => handleUpdateHook(idx, { id: e.target.value })}
                            placeholder="hook-id"
                            className="h-8 text-xs font-mono"
                          />
                        </div>
                        <div className="space-y-1">
                          <div className="flex items-center gap-1.5">
                            <Label className="text-[11px]">Event</Label>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Info className="size-3 cursor-default text-muted-foreground" />
                              </TooltipTrigger>
                              <TooltipContent>Lifecycle moment that triggers this command.</TooltipContent>
                            </Tooltip>
                          </div>
                          <Select
                            value={triggerValue}
                            onValueChange={(value) =>
                              handleUpdateHook(idx, { trigger: value as HookConfig['trigger'] })
                            }
                          >
                            <SelectTrigger size="sm">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {activeHookEvents.map((event) => (
                                <SelectItem key={event.value} value={event.value}>
                                  {event.label}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                        <div className="space-y-1">
                          <div className="flex items-center justify-end">
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  type="button"
                                  variant="ghost"
                                  size="xs"
                                  className="h-8 w-8 p-0 text-destructive hover:bg-destructive/10 hover:text-destructive"
                                  onClick={() => handleRemoveHook(idx)}
                                >
                                  <Trash2 className="size-3.5" />
                                </Button>
                              </TooltipTrigger>
                              <TooltipContent>Delete hook</TooltipContent>
                            </Tooltip>
                          </div>
                        </div>
                      </div>

                      <div className="space-y-1">
                        <div className="flex items-center gap-1.5">
                          <Label className="text-[11px]">Command</Label>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Info className="size-3 cursor-default text-muted-foreground" />
                            </TooltipTrigger>
                            <TooltipContent>Command executed when this hook fires.</TooltipContent>
                          </Tooltip>
                        </div>
                        <AutocompleteInput
                          value={hook.command ?? ''}
                          options={hookCommandSuggestions}
                          onValueChange={(value) => handleUpdateHook(idx, { command: value })}
                          placeholder="$SHIP_HOOKS_BIN"
                          className="h-8 text-xs font-mono"
                        />
                      </div>

                      <div className="grid gap-2 sm:grid-cols-[1fr_140px_1fr]">
                        <div className="space-y-1">
                          <div className="flex items-center gap-1.5">
                            <Label className="text-[11px]">Description</Label>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Info className="size-3 cursor-default text-muted-foreground" />
                              </TooltipTrigger>
                              <TooltipContent>Optional note for audit logs and UI context.</TooltipContent>
                            </Tooltip>
                          </div>
                          <Input
                            value={hook.description ?? ''}
                            onChange={(e) => handleUpdateHook(idx, { description: e.target.value || null })}
                            placeholder="Description (optional)"
                            className="h-8 text-xs"
                          />
                        </div>
                        <div className="space-y-1">
                          <div className="flex items-center gap-1.5">
                            <Label className="text-[11px]">Timeout</Label>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Info className="size-3 cursor-default text-muted-foreground" />
                              </TooltipTrigger>
                              <TooltipContent>Max runtime in milliseconds before the hook is aborted.</TooltipContent>
                            </Tooltip>
                          </div>
                          <Input
                            type="number"
                            min={0}
                            value={hook.timeout_ms ?? ''}
                            onChange={(e) =>
                              {
                                const raw = e.target.value.trim();
                                const parsed = Number(raw);
                                handleUpdateHook(idx, {
                                  timeout_ms: raw && Number.isFinite(parsed) ? parsed : null,
                                });
                              }
                            }
                            placeholder="Timeout ms"
                            className="h-8 text-xs font-mono"
                          />
                        </div>
                        <div className="space-y-1">
                          <div className="flex items-center gap-1.5">
                            <Label className="text-[11px]">Matcher</Label>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Info className="size-3 cursor-default text-muted-foreground" />
                              </TooltipTrigger>
                              <TooltipContent>Optional tool/event filter. Leave blank to run on all matches.</TooltipContent>
                            </Tooltip>
                          </div>
                          <AutocompleteInput
                            value={hook.matcher ?? ''}
                            options={hookMatcherSuggestions}
                            onValueChange={(value) => handleUpdateHook(idx, { matcher: value || null })}
                            placeholder={triggerMeta?.matcherHint ?? 'Matcher (optional)'}
                            className="h-8 text-xs font-mono"
                          />
                        </div>
                      </div>
                    </div>
                  );
                })}

                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="outline" size="sm" className="w-full border-dashed" onClick={handleAddHook}>
                      <Plus className="mr-1.5 size-3.5" />
                      Add Hook
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    Add a lifecycle hook command for context injection, policy, or logging.
                  </TooltipContent>
                </Tooltip>
              </CardContent>
            </Card>

            <Card size="sm" className="h-fit overflow-hidden bg-muted/10">
              <div className="flex items-center gap-3 border-b bg-gradient-to-r from-slate-500/10 via-card/80 to-card/50 px-4 py-3">
                <div className="flex size-7 items-center justify-center rounded-lg border border-slate-500/20 bg-slate-500/10">
                  <Info className="size-3.5 text-slate-500" />
                </div>
                <h3 className="text-sm font-semibold">Provider Support</h3>
              </div>
              <CardContent className="space-y-3 text-xs leading-relaxed !pt-5">
                <div className="rounded-md border bg-card p-3">
                  <p className="font-semibold">Native hooks enabled</p>
                  <p className="mt-1 text-muted-foreground">
                    {providersWithNativeHooks.length > 0
                      ? providersWithNativeHooks.join(', ')
                      : 'No connected providers with native hook support.'}
                  </p>
                </div>

                <div className="rounded-md border bg-card p-3">
                  <p className="font-semibold">Assessment</p>
                  <p className="mt-1 text-muted-foreground">
                    Codex currently has no native hooks surface in config. Ship keeps hook state provider-agnostic, exports to Claude and Gemini, and skips Codex hook export.
                  </p>
                </div>

                {providersWithoutNativeHooks.length > 0 && (
                  <div className="rounded-md border bg-card p-3">
                    <p className="font-semibold">No native hooks</p>
                    <p className="mt-1 text-muted-foreground">{providersWithoutNativeHooks.join(', ')}</p>
                  </div>
                )}
              </CardContent>
            </Card>
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
                  <div className="flex-1">
                    <h3 className="text-sm font-semibold">Capabilities</h3>
                    <p className="text-[11px] text-muted-foreground">Fine-grained policy for tools, filesystem access, and session limits.</p>
                  </div>
                  <div className="flex items-center gap-2">
                    {discoveryCache && (
                      <Badge variant="outline" className="text-[10px]">
                        {discoveryCache.shell_commands.length} cmds • {discoveryCache.filesystem_paths.length} paths
                      </Badge>
                    )}
                    <Button
                      type="button"
                      variant="outline"
                      size="xs"
                      className="h-6 px-2 text-[10px]"
                      onClick={() => refreshDiscoveryCacheMut.mutate()}
                      disabled={refreshDiscoveryCacheMut.isPending}
                    >
                      {refreshDiscoveryCacheMut.isPending ? 'Refreshing…' : 'Refresh hints'}
                    </Button>
                  </div>
                </div>
                <CardContent className="space-y-6 !pt-5">
                  {!permissions ? (
                    <p className="py-10 text-center text-sm text-muted-foreground">Loading permissions...</p>
                  ) : (
                    <Tabs defaultValue="tools">
                      <TabsList className="mb-4">
                        <TabsTrigger value="tools">Tools</TabsTrigger>
                        <TabsTrigger value="commands">Commands</TabsTrigger>
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
                                    <AutocompleteInput
                                      value={p || ''}
                                      options={permissionToolSuggestions}
                                      noResultsText="Type a custom tool pattern."
                                      onValueChange={(value) => {
                                        const next = [...(permissions.tools?.allow || [])];
                                        next[idx] = value;
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
                                    <AutocompleteInput
                                      value={p || ''}
                                      options={permissionToolSuggestions}
                                      noResultsText="Type a custom restriction pattern."
                                      onValueChange={(value) => {
                                        const next = [...(permissions.tools?.deny || [])];
                                        next[idx] = value;
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

                      <TabsContent value="commands" className="space-y-6">
                        <div className="grid gap-6 md:grid-cols-3">
                          <div className="space-y-3">
                            <div className="flex items-center gap-2">
                              <Terminal className="size-4 text-emerald-500" />
                              <Label>Allow Commands</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Command prefixes or patterns that are explicitly allowed.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <p className="text-xs text-muted-foreground">e.g. <code className="font-mono">git</code> or <code className="font-mono">ship *</code></p>
                            <div className="space-y-2">
                              {(permissions.commands?.allow || []).map((p, idx) => (
                                <div key={idx} className="flex items-center gap-2">
                                  <AutocompleteInput
                                    value={p || ''}
                                    options={commandPatternSuggestions}
                                    noResultsText="Type a custom command pattern."
                                    onValueChange={(value) => {
                                      const next = [...(permissions.commands?.allow || [])];
                                      next[idx] = value;
                                      savePermissionsMut.mutate({
                                        ...permissions,
                                        commands: {
                                          ...permissions.commands,
                                          allow: next,
                                          deny: permissions.commands?.deny || [],
                                        },
                                      });
                                    }}
                                    className="font-mono text-xs"
                                  />
                                  <Button
                                    variant="ghost"
                                    size="xs"
                                    onClick={() => {
                                      const next = (permissions.commands?.allow || []).filter((_, i) => i !== idx);
                                      savePermissionsMut.mutate({
                                        ...permissions,
                                        commands: {
                                          ...permissions.commands,
                                          allow: next,
                                          deny: permissions.commands?.deny || [],
                                        },
                                      });
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
                                    commands: {
                                      ...permissions.commands,
                                      allow: [...(permissions.commands?.allow || []), ''],
                                      deny: permissions.commands?.deny || [],
                                    },
                                  });
                                }}
                              >
                                <Plus className="mr-1 size-3.5" /> Add Allow Pattern
                              </Button>
                            </div>
                          </div>

                          <div className="space-y-3">
                            <div className="flex items-center gap-2">
                              <ShieldAlert className="size-4 text-destructive" />
                              <Label>Block Commands</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  These command patterns are never executed.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <p className="text-xs text-muted-foreground">e.g. <code className="font-mono">rm -rf *</code> or <code className="font-mono">git push</code></p>
                            <div className="space-y-2">
                              {(permissions.commands?.deny || []).map((p, idx) => (
                                <div key={idx} className="flex items-center gap-2">
                                  <AutocompleteInput
                                    value={p || ''}
                                    options={commandPatternSuggestions}
                                    noResultsText="Type a custom blocked command."
                                    onValueChange={(value) => {
                                      const next = [...(permissions.commands?.deny || [])];
                                      next[idx] = value;
                                      savePermissionsMut.mutate({
                                        ...permissions,
                                        commands: {
                                          ...permissions.commands,
                                          deny: next,
                                          allow: permissions.commands?.allow || [],
                                        },
                                      });
                                    }}
                                    className="font-mono text-xs"
                                  />
                                  <Button
                                    variant="ghost"
                                    size="xs"
                                    onClick={() => {
                                      const next = (permissions.commands?.deny || []).filter((_, i) => i !== idx);
                                      savePermissionsMut.mutate({
                                        ...permissions,
                                        commands: {
                                          ...permissions.commands,
                                          deny: next,
                                          allow: permissions.commands?.allow || [],
                                        },
                                      });
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
                                    commands: {
                                      ...permissions.commands,
                                      deny: [...(permissions.commands?.deny || []), ''],
                                      allow: permissions.commands?.allow || [],
                                    },
                                  });
                                }}
                              >
                                <Plus className="mr-1 size-3.5" /> Add Block Pattern
                              </Button>
                            </div>
                          </div>

                          <div className="space-y-3">
                            <div className="flex items-center gap-2">
                              <Info className="size-4 text-amber-500" />
                              <Label>Require Approval</Label>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Info className="size-3 cursor-default text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  Matching commands prompt for confirmation even when allowed.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <p className="text-xs text-muted-foreground">e.g. <code className="font-mono">git push</code> or <code className="font-mono">ship release *</code></p>
                            <div className="space-y-2">
                              {(permissions.agent?.require_confirmation || []).map((p, idx) => (
                                <div key={idx} className="flex items-center gap-2">
                                  <AutocompleteInput
                                    value={p || ''}
                                    options={commandPatternSuggestions}
                                    noResultsText="Type a command requiring approval."
                                    onValueChange={(value) => {
                                      const next = [...(permissions.agent?.require_confirmation || [])];
                                      next[idx] = value;
                                      savePermissionsMut.mutate({
                                        ...permissions,
                                        agent: {
                                          ...permissions.agent,
                                          require_confirmation: next,
                                        },
                                      });
                                    }}
                                    className="font-mono text-xs"
                                  />
                                  <Button
                                    variant="ghost"
                                    size="xs"
                                    onClick={() => {
                                      const next = (permissions.agent?.require_confirmation || []).filter((_, i) => i !== idx);
                                      savePermissionsMut.mutate({
                                        ...permissions,
                                        agent: {
                                          ...permissions.agent,
                                          require_confirmation: next,
                                        },
                                      });
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
                                    agent: {
                                      ...permissions.agent,
                                      require_confirmation: [...(permissions.agent?.require_confirmation || []), ''],
                                    },
                                  });
                                }}
                              >
                                <Plus className="mr-1 size-3.5" /> Add Approval Pattern
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
                                  <AutocompleteInput
                                    value={p || ''}
                                    options={filesystemPathSuggestions}
                                    noResultsText="Type a custom path pattern."
                                    onValueChange={(value) => {
                                      const next = [...(permissions.filesystem?.allow || [])];
                                      next[idx] = value;
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
                                  <AutocompleteInput
                                    value={p || ''}
                                    options={filesystemPathSuggestions}
                                    noResultsText="Type a custom blocked path."
                                    onValueChange={(value) => {
                                      const next = [...(permissions.filesystem?.deny || [])];
                                      next[idx] = value;
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
    </PageFrame>
  );
}
