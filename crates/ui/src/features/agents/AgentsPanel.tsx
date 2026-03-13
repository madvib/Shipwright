import { useEffect, useMemo, useRef, useState } from 'react';
import { Bot, Plus, Shield, ShieldAlert, FileSearch, Trash2, Upload, Download, Globe, Folder, Package, PenLine, ChevronDown, ChevronRight, ScrollText, LockIcon, Info, Zap, BookOpen, Terminal, Wrench, Save, ArrowLeft, ExternalLink } from 'lucide-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { commands, AgentDiscoveryCache, CatalogEntry, HookConfig, McpProbeReport, McpRegistryEntry, McpServerConfig, McpServerType, McpValidationIssue, McpValidationReport, Permissions, ProjectConfig, ProjectDiscovery, ProviderInfo, ProviderToolVocabularyEntry, SkillToolHint } from '@/bindings';
import { DEFAULT_STATUSES } from '@/lib/workspace-ui';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Alert, AlertDescription } from '@ship/ui';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Card, CardContent } from '@ship/ui';
import { Input } from '@ship/ui';
import { Label } from '@ship/ui';
import { PageFrame, PageHeader } from '@ship/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/ui';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@ship/ui';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/ui';
import MarkdownEditor from '@/components/editor';
import { AutocompleteInput } from '@ship/ui';
import { cn } from '@/lib/utils';
import { useAgentAssetInventory } from '@/features/agents/shared/useAgentAssetInventory';
import { useMcpRegistrySearch } from '@/features/agents/shared/useMcpRegistrySearch';
import { ExplorerDialog } from '@/features/agents/shared/ExplorerDialog';

interface AgentsPanelProps {
  activeProject?: ProjectDiscovery | null;
  projectConfig: ProjectConfig | null;
  globalAgentConfig: ProjectConfig | null;
  onSaveProject: (config: ProjectConfig) => void | Promise<void>;
  onSaveGlobalAgentConfig: (config: ProjectConfig) => void | Promise<void>;
  initialSection?: AgentSection;
  onBackToSettings?: () => void;
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

const SECTION_META: Record<AgentSection, { title: string; description: string }> = {
  providers: {
    title: 'Providers',
    description: 'Provider defaults and sync.',
  },
  mcp: {
    title: 'MCP Servers',
    description: 'Installed servers and connectivity.',
  },
  skills: {
    title: 'Skills',
    description: 'Skill files and source installs.',
  },
  rules: {
    title: 'Rules',
    description: 'Session instructions.',
  },
  hooks: {
    title: 'Hooks',
    description: 'Lifecycle commands.',
  },
  permissions: {
    title: 'Permissions',
    description: 'Tool, shell, and filesystem policy.',
  },
};

type HookEventOption = {
  value: string;
  label: string;
  providers: string[];
  matcherHint?: string;
  description?: string;
};

const HOOK_EVENTS: HookEventOption[] = [
  { value: 'SessionStart', label: 'Session Start', providers: ['claude', 'gemini'], description: 'Triggered when a new agent session begins.' },
  { value: 'UserPromptSubmit', label: 'User Prompt Submit', providers: ['claude'], description: 'Triggered exactly after the user sends a message, before model processing.' },
  { value: 'PreToolUse', label: 'Pre Tool Use', providers: ['claude', 'gemini'], matcherHint: 'Tool matcher (e.g. Bash, mcp__*).', description: 'Before a tool is executed. Can be used for custom authorization.' },
  { value: 'PermissionRequest', label: 'Permission Request', providers: ['claude'], description: 'Triggered when the agent asks for explicit permission for an action.' },
  { value: 'PostToolUse', label: 'Post Tool Use', providers: ['claude', 'gemini'], matcherHint: 'Tool matcher (e.g. Bash, mcp__*).', description: 'After a tool execution completes successfully.' },
  { value: 'PostToolUseFailure', label: 'Post Tool Failure', providers: ['claude'], description: 'Triggered if a tool execution returns an error.' },
  { value: 'Notification', label: 'Notification', providers: ['claude', 'gemini'], description: 'Triggered when the agent sends an out-of-band notification.' },
  { value: 'SubagentStart', label: 'Subagent Start', providers: ['claude'], description: 'Triggered when a subagent is spawned.' },
  { value: 'SubagentStop', label: 'Subagent Stop', providers: ['claude'], description: 'Triggered when a subagent terminates.' },
  { value: 'Stop', label: 'Stop', providers: ['claude', 'gemini'], description: 'Triggered when the main agent loop stops.' },
  { value: 'PreCompact', label: 'Pre Compact', providers: ['claude', 'gemini'], description: 'Triggered before context compaction/token management starts.' },
  { value: 'BeforeTool', label: 'Before Tool', providers: ['gemini'], matcherHint: 'Tool matcher (e.g. run_shell_command).', description: 'Gemini-specific hook before tool execution.' },
  { value: 'AfterTool', label: 'After Tool', providers: ['gemini'], matcherHint: 'Tool matcher (e.g. run_shell_command).', description: 'Gemini-specific hook after tool execution.' },
  { value: 'BeforeAgent', label: 'Before Agent', providers: ['gemini'], description: 'Gemini-specific hook before agent logic runs.' },
  { value: 'AfterAgent', label: 'After Agent', providers: ['gemini'], description: 'Gemini-specific hook after agent logic runs.' },
  { value: 'SessionEnd', label: 'Session End', providers: ['gemini'], description: 'Triggered when the Gemini session completes.' },
  { value: 'BeforeModel', label: 'Before Model', providers: ['gemini'], description: 'Gemini-specific hook before model call.' },
  { value: 'AfterModel', label: 'After Model', providers: ['gemini'], description: 'Gemini-specific hook after model response.' },
  { value: 'BeforeToolSelection', label: 'Before Tool Selection', providers: ['gemini'], description: 'Gemini-specific hook during tool selection phase.' },
];

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
const MCP_STDIO_ONLY_ALPHA = false;
const EMPTY_CATALOG: CatalogEntry[] = [];
const EMPTY_RULES: Array<{ file_name: string; content: string }> = [];

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
      commands: { allow: [], deny: ['*'] },
      network: { policy: 'none', allow_hosts: [] },
      agent: { require_confirmation: [] },
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
      commands: { allow: ['*'], deny: [] },
      network: { policy: 'unrestricted', allow_hosts: [] },
      agent: { require_confirmation: [] },
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
            placeholder="e.g. ship"
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
              <TooltipContent>Stable slug used in permissions and provider exports.</TooltipContent>
            </Tooltip>
          </div>
          <AutocompleteInput
            value={draft.id ?? ''}
            options={idOptions}
            placeholder={slugifyId(draft.name || 'server-id') || 'server-id'}
            onValueChange={(value) => setField('id', value)}
            className="h-8 text-xs font-mono"
            autoCapitalize="none"
            autoCorrect="off"
            spellCheck={false}
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
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
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
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
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
                  autoCapitalize="none"
                  autoCorrect="off"
                  spellCheck={false}
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
                  autoCapitalize="none"
                  autoCorrect="off"
                  spellCheck={false}
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

function getProviderSyncSummary(
  provider: ProviderRow,
  enabled: boolean,
  validationReport: McpValidationReport | null
): ProviderSyncSummary {
  const providerIssues = (validationReport?.issues ?? []).filter((issue) => issue.provider_id === provider.id);
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
      issues: blockingProviderIssues,
    };
  }

  if (blockingProviderIssues.length > 0) {
    return {
      status: 'needs-attention',
      detail: `Fix ${provider.name} config issues before syncing.`,
      issues: blockingProviderIssues,
    };
  }

  if (driftIssues.length > 0) {
    return {
      status: 'drift-detected',
      detail: `${provider.name} config shape diverges from Ship expectations.`,
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

function permissionsQueryKey(scope: ScopeKey) {
  return ['permissions', scope] as const;
}

interface PatternListEditorProps {
  patterns: string[];
  options: Array<{ value: string; label?: string; keywords?: string[] }>;
  addLabel: string;
  addValue?: string;
  noResultsText: string;
  onChange: (updater: (current: string[]) => string[]) => void;
}

function PatternListEditor({
  patterns,
  options,
  addLabel,
  addValue = '',
  noResultsText,
  onChange,
}: PatternListEditorProps) {
  return (
    <div className="space-y-2">
      {patterns.map((pattern, idx) => (
        <div key={idx} className="flex items-center gap-2">
          <AutocompleteInput
            value={pattern || ''}
            options={options}
            noResultsText={noResultsText}
            onValueChange={(value) =>
              onChange((current) => current.map((item, itemIndex) => (itemIndex === idx ? value : item)))
            }
            className="font-mono text-xs"
            autoCapitalize="none"
            autoCorrect="off"
            spellCheck={false}
          />
          <Button
            variant="ghost"
            size="xs"
            onClick={() => onChange((current) => current.filter((_, index) => index !== idx))}
          >
            <Trash2 className="size-3.5" />
          </Button>
        </div>
      ))}
      <Button
        variant="outline"
        size="xs"
        className="w-full border-dashed"
        onClick={() => onChange((current) => [...current, addValue])}
      >
        <Plus className="mr-1 size-3.5" /> {addLabel}
      </Button>
    </div>
  );
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
  const providerToolVocabulary = (providerToolVocabularyQuery.data ?? []) as ProviderToolVocabularyEntry[];

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
  const skillToolHints = (skillToolHintsQuery.data ?? []) as SkillToolHint[];
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
  }, [
    initialSection,
    agentScope,
    hasActiveProject,
    activeAgentConfig,
    mcpServersInventoryQuery.data,
  ]);

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
        const haystack = [
          entry.id,
          entry.name,
          entry.description,
          entry.author ?? '',
          ...(entry.tags ?? []),
        ]
          .join(' ')
          .toLowerCase();
        return haystack.includes(query);
      })
      .sort((left, right) => left.name.localeCompare(right.name));
  }, [mcpCatalogEntries, mcpCatalogInput]);
  const skillFolderRows = useMemo(
    () =>
      activeDocs.map((doc) => ({
        id: doc.id,
        fileName: 'SKILL.md',
        title: doc.title || doc.id,
      })),
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
    return Array.from(
      new Set(
        direct
          .map((value) => value.trim())
          .filter(Boolean)
      )
    );
  }, [skillToolHints]);
  const providerNativeToolIds = useMemo(() => {
    const configured = new Set((activeAgentConfig.providers ?? []).map((provider) => provider.trim()).filter(Boolean));
    const enabledRows = providerToolVocabulary.filter((entry) => entry.enabled);
    const configuredRows = providerToolVocabulary.filter((entry) => configured.has(entry.provider_id));
    const installedRows = providerToolVocabulary.filter((entry) => entry.installed);
    const sourceRows =
      configuredRows.length > 0
        ? configuredRows
        : enabledRows.length > 0
          ? enabledRows
          : installedRows.length > 0
            ? installedRows
            : providerToolVocabulary;

    return Array.from(new Set(sourceRows.flatMap((entry) => entry.tool_ids ?? [])))
      .map((value) => value.trim())
      .filter((value) => value.length > 0 && value !== '*' && !value.startsWith('mcp__'));
  }, [
    activeAgentConfig.providers,
    providerToolVocabulary,
  ]);
  const permissionToolSuggestions = useMemo(() => {
    const providerNativeTools = providerNativeToolIds;
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
    const builtInSet = new Set(providerNativeTools);
    const values = Array.from(new Set([
      ...baseline,
      ...providerNativeTools,
      ...serverPatterns,
      ...catalogPatterns,
      ...discoveredMcpToolPatterns,
      ...skillToolAllowedPatterns,
    ]));
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
        const type = value === '*'
          ? 'wildcard'
          : builtInSet.has(value)
            ? 'builtin'
            : value.startsWith('mcp__')
              ? 'mcp'
              : 'pattern';
        return {
          value,
          label: type === 'wildcard'
            ? 'Wildcard (all tools)'
            : type === 'builtin'
              ? 'Built-in provider tool'
              : type === 'mcp'
                ? 'MCP tool pattern'
                : 'Tool pattern',
          keywords: [type, 'tool'],
        };
      });
  }, [
    activeAgentConfig.mcp_servers,
    mcpCatalogEntries,
    discoveredMcpToolPatterns,
    skillToolAllowedPatterns,
    providerNativeToolIds,
  ]);
  const hookCommandSuggestions = useMemo(() => {
    const seeded = ['$SHIP_HOOKS_BIN', 'ship hooks run', 'node', 'bash'];
    const shellValues = (discoveryCache?.shell_commands ?? []).slice(0, 120);
    const values = [...seeded, ...mcpCommandOptions.map((option) => option.value), ...shellValues];
    return Array.from(new Set(values.filter(Boolean))).map((value) => ({ value }));
  }, [mcpCommandOptions, discoveryCache]);
  const hookMatcherSuggestions = useMemo(() => {
    const seeded = [
      'mcp__*',
      'mcp__*__read*',
      'mcp__*__write*',
    ];
    const values = [
      ...seeded,
      ...providerNativeToolIds,
      ...permissionToolSuggestions.map((option) => option.value),
    ];
    return Array.from(new Set(values.filter(Boolean))).map((value) => ({ value }));
  }, [permissionToolSuggestions, providerNativeToolIds]);
  const filesystemPathSuggestions = useMemo(
    () => {
      const seeded = [
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
      permissionsSaveQueueRef.current = queued.then(
        () => undefined,
        () => undefined
      );
      await queued;
      return {
        scope: input.scope,
        permissions: normalizePermissionsForEditor(input.permissions),
      };
    },
    onMutate: (input) => {
      queryClient.setQueryData(
        permissionsQueryKey(input.scope),
        normalizePermissionsForEditor(input.permissions),
      );
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
      return {
        ...current,
        [agentScope]: normalizePermissionsForEditor(permissions),
      };
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

  const updatePermissions = (
    updater: (current: Permissions) => Permissions
  ) => {
    setPermissionsDraftByScope((drafts) => {
      const fallback = normalizePermissionsForEditor(
        (queryClient.getQueryData(permissionsQueryKey(agentScope)) as Permissions | undefined)
        ?? permissions
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
      if (!trimmed) {
        issues.push(`Tools ${listName}: empty pattern.`);
        return;
      }
      if (trimmed.includes(' ')) {
        issues.push(`Tools ${listName}: "${trimmed}" contains spaces; use glob patterns/tool IDs.`);
      }
      if (trimmed.startsWith('mcp__') && !trimmed.slice('mcp__'.length).includes('__')) {
        issues.push(`Tools ${listName}: "${trimmed}" is missing "__tool" segment.`);
      }
    };

    for (const value of activePermissions.tools?.allow ?? []) checkToolPattern(value, 'allow');
    for (const value of activePermissions.tools?.deny ?? []) checkToolPattern(value, 'deny');

    for (const value of activePermissions.commands?.allow ?? []) {
      if (!value.trim()) issues.push('Commands allow: empty pattern.');
    }
    for (const value of activePermissions.commands?.deny ?? []) {
      if (!value.trim()) issues.push('Commands deny: empty pattern.');
    }
    for (const value of activePermissions.filesystem?.allow ?? []) {
      if (!value.trim()) issues.push('Filesystem allow: empty path pattern.');
    }
    for (const value of activePermissions.filesystem?.deny ?? []) {
      if (!value.trim()) issues.push('Filesystem deny: empty path pattern.');
    }
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
    const existingIds = new Set(
      servers
        .map((server) => (server.id ?? '').trim())
        .filter(Boolean)
    );
    const baseId = inferMcpServerId(definition);
    let id = baseId;
    let index = 2;
    while (existingIds.has(id)) {
      id = `${baseId}-${index}`;
      index += 1;
    }
    const nextServer: McpServerConfig = {
      ...definition,
      id,
      name: definition.name.trim() || id,
    };
    servers.push(nextServer);
    updateActiveAgentConfig({ ...activeAgentConfig, mcp_servers: servers });
    setMcpExplorerOpen(false);
    setMcpEditDraft({ idx: servers.length - 1, server: nextServer });
  };

  const handleInstallCatalogMcpEntry = (entry: CatalogEntry) => {
    appendMcpServerDefinition(mcpServerFromCatalog(entry));
  };

  const handleInstallRegistryEntry = (entry: McpRegistryEntry) => {
    const normalizedServer = mcpServerFromRegistry(entry);
    if (MCP_STDIO_ONLY_ALPHA && normalizedServer.server_type !== 'stdio') {

      return;
    }
    appendMcpServerDefinition(normalizedServer);
  };

  const handleInstallSkillFromSource = () => {
    const source = parsedSkillInstallSpec.source.trim();
    const skillId = parsedSkillInstallSpec.skillId;
    if (!source || !parsedSkillInstallSpec.canInstall) return;
    installSkillFromSourceMut.mutate({
      source,
      skillId,
    });
  };

  const handleExport = async (target: string) => {
    setExportStatus((prev) => ({ ...prev, [target]: 'loading' }));

    try {
      const res = await commands.exportAgentConfigCmd(target);
      if (res.status === 'error') throw new Error(res.error);
      setExportStatus((prev) => ({ ...prev, [target]: 'ok' }));
    } catch (err) {
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
        importedMcp > 0
          ? `${importedMcp} MCP server${importedMcp === 1 ? '' : 's'} imported`
          : 'No new MCP servers',
        importedSkills > 0
          ? `${importedSkills} skill${importedSkills === 1 ? '' : 's'} imported`
          : 'No new skills',
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
      refreshProviders();
      void mcpValidationQuery.refetch();
    } catch (err) {
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
      if (deny.has(pattern)) {
        deny.delete(pattern);
      } else {
        deny.add(pattern);
      }
      return {
        ...current,
        tools: {
          ...current.tools,
          allow: current.tools?.allow ?? ['*'],
          deny: Array.from(deny),
        },
      };
    });
  };

  const handleToggleServerToolBlock = (serverId: string) => {
    const wildcard = `mcp__${serverId}__*`;
    updatePermissions((current) => {
      const deny = new Set(current.tools?.deny ?? []);
      if (deny.has(wildcard)) {
        deny.delete(wildcard);
      } else {
        deny.add(wildcard);
      }
      return {
        ...current,
        tools: {
          ...current.tools,
          allow: current.tools?.allow ?? ['*'],
          deny: Array.from(deny),
        },
      };
    });
  };

  const handleSave = () => {
    if (agentScope === 'global') return onSaveGlobalAgentConfig(localGlobalAgent);
    return onSaveProject(localProject);
  };

  const sectionMeta = SECTION_META[initialSection];
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
  const headerActions = (
    <div className="flex flex-wrap items-center gap-2">
      {scopeToggle}
    </div>
  );
  const sectionTitle = onBackToSettings && initialSection === 'skills'
    ? (
      <div className="flex items-center gap-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              className="h-8 w-8 rounded-full"
              onClick={onBackToSettings}
              aria-label="Back to settings"
            >
              <ArrowLeft className="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Back to settings</TooltipContent>
        </Tooltip>
        <span>{sectionMeta.title}</span>
      </div>
    )
    : sectionMeta.title;
  const frameClass =
    initialSection === 'skills' && onBackToSettings ? 'p-2 md:p-4' : 'md:p-8';

  return (
    <PageFrame className={frameClass}>
      <PageHeader
        title={sectionTitle}
        description={sectionMeta.description}
        badge={<Badge variant="outline">Agents</Badge>}
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
                    <p className="text-[11px] text-muted-foreground">Ship manages provider configs and native exports.</p>
                  </div>
                </div>
                {providersPending && (
                  <Badge variant="outline" className="text-[10px] text-muted-foreground">
                    checking PATH...
                  </Badge>
                )}
              </div>
              {providersError && (
                <div className="flex items-center justify-between gap-2 border-b bg-rose-500/5 px-4 py-2.5">
                  <p className="text-[11px] text-rose-700 dark:text-rose-300">
                    Provider detection failed. Showing supported providers with unknown install status.
                  </p>
                  <Button
                    type="button"
                    variant="ghost"
                    size="xs"
                    className="h-6 px-2 text-[10px]"
                    onClick={() => refreshProviders()}
                  >
                    Retry
                  </Button>
                </div>
              )}
              <div className="divide-y divide-border/50">
                {providerRows.map((provider) => {
                  const syncSummary = getProviderSyncSummary(provider, true, mcpValidationReport);
                  const isExpanded = expandedProviderId === provider.id;
                  const providerHookEvents = HOOK_EVENTS.filter((event) => event.providers.includes(provider.id));
                  const issueCount = syncSummary.issues.length;
                  const nextIssue = syncSummary.issues.find((issue) => issue.level !== 'info') ?? syncSummary.issues[0] ?? null;
                  const importDisabledReason =
                    !hasActiveProject
                      ? 'Open or create a project to import provider config files.'
                      : agentScope !== 'project'
                        ? 'Switch to Project scope to import into this project.'
                        : provider.id === 'claude'
                          ? 'Imports from .mcp.json first, then ~/.claude.json if missing.'
                          : provider.id === 'gemini'
                            ? 'Imports from .gemini/settings.json first, then ~/.gemini/settings.json if missing.'
                            : 'Imports from .codex/config.toml in this project.';
                  const exportTargetReason = hasActiveProject
                    ? provider.id === 'claude'
                      ? 'Writes .mcp.json, CLAUDE.md, .claude/settings.json, and .claude/skills.'
                      : provider.id === 'gemini'
                        ? 'Writes .gemini/settings.json, GEMINI.md, .gemini/policies/ship-permissions.toml, and .gemini/skills.'
                        : 'Writes .codex/config.toml, AGENTS.md, and .agents/skills.'
                    : 'Open or create a project to export provider config files.';
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
                                Provider sync health for this scope.
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
                            {issueCount > 0 ? (
                              <Badge variant="outline" className="cursor-default border-amber-500/40 bg-amber-500/10 text-[10px] text-amber-800 dark:text-amber-300">
                                {issueCount} warning{issueCount === 1 ? '' : 's'}
                              </Badge>
                            ) : null}
                          </div>
                          {importSummary[provider.id] && (
                            <p className="text-[11px] text-muted-foreground">{importSummary[provider.id]}</p>
                          )}
                        </div>

                        <div className="flex shrink-0 items-center gap-1.5">
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
                                variant="outline"
                                size="xs"
                                disabled={!hasActiveProject || exportStatus[provider.id] === 'loading'}
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
                              {exportTargetReason}
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
                          {(!provider.installed || nextIssue) && (
                            <div className={cn(
                              'rounded-md border px-3 py-2 text-[11px]',
                              !provider.installed
                                ? 'border-amber-500/30 bg-amber-500/5 text-amber-800 dark:text-amber-200'
                                : 'border-border/70 bg-background/80 text-muted-foreground'
                            )}>
                              {!provider.installed ? (
                                <div className="flex items-center justify-between gap-2">
                                  <p>
                                    Install <code>{provider.binary}</code> and then rescan provider detection.
                                  </p>
                                  <Button
                                    type="button"
                                    variant="outline"
                                    size="xs"
                                    className="h-6 px-2 text-[10px]"
                                    onClick={() => refreshProviders()}
                                  >
                                    Rescan
                                  </Button>
                                </div>
                              ) : nextIssue ? (
                                <div className="space-y-1">
                                  <p className="font-medium text-foreground">Next step</p>
                                  <p>{nextIssue.message}</p>
                                  {nextIssue.hint ? <p>Hint: {nextIssue.hint}</p> : null}
                                </div>
                              ) : null}
                            </div>
                          )}

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
                                  <Tooltip key={`${provider.id}-${event.value}`}>
                                    <TooltipTrigger asChild>
                                      <Badge variant="secondary" className="text-[10px] cursor-default">
                                        {event.label}
                                      </Badge>
                                    </TooltipTrigger>
                                    <TooltipContent className="max-w-xs">{event.description}</TooltipContent>
                                  </Tooltip>
                                ))}
                              </div>
                            ) : (
                              <p className="text-[11px] text-muted-foreground">
                                No native hook export support for this provider yet.
                              </p>
                            )}
                          </div>

                          <div className="space-y-2 rounded-md border bg-background/80 p-2.5">
                            <div className="flex items-center justify-between gap-2">
                              <p className="text-[11px] font-medium">
                                Diagnostics
                              </p>
                              <Button
                                type="button"
                                variant="outline"
                                size="xs"
                                className="h-6 px-2 text-[10px]"
                                onClick={() => void mcpValidationQuery.refetch()}
                                disabled={mcpValidationQuery.isFetching}
                              >
                                {mcpValidationQuery.isFetching ? 'Checking…' : 'Validate'}
                              </Button>
                            </div>
                            {syncSummary.issues.length === 0 ? (
                              <p className="text-[11px] text-emerald-700 dark:text-emerald-300">
                                No diagnostics warnings for this provider.
                              </p>
                            ) : (
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
                                    <p className="font-medium">{issue.message}</p>
                                    {issue.hint && <p className="opacity-90">Hint: {issue.hint}</p>}
                                  </div>
                                ))}
                              </div>
                            )}
                          </div>
                        </div>
                      )}
                    </div>
                  );
                })}
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
                  <div className="flex items-center gap-2 text-sm font-semibold">
                    <h3 className="text-sm font-semibold">MCP Servers</h3>
                    {MCP_STDIO_ONLY_ALPHA && (
                      <Badge variant="secondary" className="text-[10px]">stdio-only alpha</Badge>
                    )}
                  </div>
                  <p className="text-[11px] text-muted-foreground">Manage your MCP server library and validate connectivity.</p>
                </div>
                <Badge variant="secondary" className="shrink-0 text-[10px]">
                  {(activeAgentConfig.mcp_servers ?? []).length} server{(activeAgentConfig.mcp_servers ?? []).length !== 1 ? 's' : ''}
                </Badge>
              </div>

              <div className="border-b bg-muted/20 px-4 py-3">
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    type="button"
                    size="sm"
                    variant="secondary"
                    className="h-8"
                    onClick={() => setMcpExplorerOpen(true)}
                  >
                    <Plus className="mr-1.5 size-3.5" />
                    Explore
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    className="h-8"
                    onClick={() => setMcpEditDraft({ idx: null, server: { ...EMPTY_MCP_SERVER } })}
                  >
                    <PenLine className="mr-1.5 size-3.5" />
                    Add Manually
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    className="h-8"
                    onClick={() => setMcpDiagnosticsOpen(true)}
                  >
                    <Wrench className="mr-1.5 size-3.5" />
                    Diagnostics
                    {mcpDiagnosticsIssueCount > 0 ? (
                      <Badge variant="secondary" className="ml-1.5 h-4 px-1.5 py-0 text-[10px]">
                        {mcpDiagnosticsIssueCount}
                      </Badge>
                    ) : null}
                  </Button>
                </div>
                <div className="mt-2 flex flex-wrap items-center gap-2 text-[11px]">
                  {hasNoReachableServers ? (
                    <Badge variant="outline" className="border-rose-500/40 bg-rose-500/10 text-rose-700 dark:text-rose-300">
                      {mcpProbeReport?.reachable_servers ?? 0}/{mcpProbeReport?.checked_servers ?? 0} reachable
                    </Badge>
                  ) : mcpProbeReport ? (
                    <Badge variant="outline" className="text-muted-foreground">
                      {mcpProbeReport.reachable_servers}/{mcpProbeReport.checked_servers} reachable
                    </Badge>
                  ) : null}
                  {mcpValidationReport ? (
                    <Badge variant="outline" className={mcpValidationReport.ok ? 'text-emerald-700 dark:text-emerald-300' : 'text-amber-700 dark:text-amber-300'}>
                      Preflight {mcpValidationReport.ok ? 'ready' : 'needs attention'}
                    </Badge>
                  ) : null}
                </div>
              </div>

              <ExplorerDialog
                open={mcpExplorerOpen}
                onOpenChange={setMcpExplorerOpen}
                title="MCP Explorer"
                icon={<Package className="size-4 text-violet-500" />}
              >
                <div className="space-y-3">
                    <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_180px]">
                      <Input
                        value={mcpCatalogInput}
                        onChange={(event) => {
                          const next = event.target.value;
                          setMcpCatalogInput(next);
                          if (next.trim().length > 0 && mcpExplorerFilter === 'recommended') {
                            setMcpExplorerFilter('all');
                          }
                        }}
                        placeholder="Search by name, ID, tag, author"
                        className="h-8 text-xs"
                        autoCapitalize="none"
                        autoCorrect="off"
                        spellCheck={false}
                      />
                      <Select
                        value={mcpExplorerFilter}
                        onValueChange={(value) => setMcpExplorerFilter(value as 'recommended' | 'catalog' | 'registry' | 'all')}
                      >
                        <SelectTrigger size="sm" className="h-8 text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="recommended">Recommended</SelectItem>
                          <SelectItem value="catalog">Catalog</SelectItem>
                          <SelectItem value="registry">Registry</SelectItem>
                          <SelectItem value="all">All</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    {(mcpExplorerFilter === 'recommended' || mcpExplorerFilter === 'all') && !hasMcpSearchQuery && (
                      <div className="space-y-2 rounded-md border bg-muted/15 p-2.5">
                        <div className="space-y-0.5">
                          <p className="text-[11px] font-semibold">Recommended</p>
                          <p className="text-[10px] text-muted-foreground">
                            Based on embedded catalog entries tagged official.
                          </p>
                        </div>
                        <div className="grid gap-2 sm:grid-cols-2">
                          {recommendedMcpCatalogEntries.map((entry) => {
                            const serverId = inferMcpServerId(mcpServerFromCatalog(entry)).toLowerCase();
                            const installed = installedMcpServerIdSet.has(serverId);
                            return (
                              <div key={`recommended-${entry.id}`} className="rounded border bg-background/70 p-2">
                                <div className="flex items-start justify-between gap-2">
                                  <div className="min-w-0">
                                    <p className="truncate text-xs font-medium">{entry.name}</p>
                                    <p className="truncate text-[10px] text-muted-foreground">{entry.id}</p>
                                  </div>
                                  <div className="flex items-center gap-1">
                                    {entry.source_url ? (
                                      <Tooltip>
                                        <TooltipTrigger asChild>
                                          <Button
                                            type="button"
                                            size="icon-sm"
                                            variant="ghost"
                                            className="h-6 w-6"
                                            onClick={() => {
                                              void openUrl(entry.source_url as string);
                                            }}
                                          >
                                            <ExternalLink className="size-3.5" />
                                          </Button>
                                        </TooltipTrigger>
                                        <TooltipContent>Open source docs</TooltipContent>
                                      </Tooltip>
                                    ) : null}
                                    <Button
                                      type="button"
                                      size="xs"
                                      variant="outline"
                                      disabled={installed}
                                      onClick={() => handleInstallCatalogMcpEntry(entry)}
                                    >
                                      {installed ? 'Installed' : 'Add'}
                                    </Button>
                                  </div>
                                </div>
                                <p className="mt-1 line-clamp-2 text-[10px] text-muted-foreground">{entry.description}</p>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    )}

                    {(mcpExplorerFilter === 'catalog' || mcpExplorerFilter === 'all') && (
                      <div className="space-y-2 rounded-md border bg-muted/15 p-2.5">
                        <div className="flex items-center justify-between gap-2">
                          <p className="text-[11px] font-semibold">Catalog</p>
                          <Badge variant="secondary" className="text-[10px]">
                            {filteredMcpCatalogEntries.length} results
                          </Badge>
                        </div>
                        <div className="max-h-[48vh] space-y-1 overflow-y-auto pr-1">
                          {filteredMcpCatalogEntries.map((entry) => {
                            const serverId = inferMcpServerId(mcpServerFromCatalog(entry)).toLowerCase();
                            const installed = installedMcpServerIdSet.has(serverId);
                            return (
                              <div key={`catalog-${entry.id}`} className="flex items-center justify-between gap-2 rounded border bg-background/70 px-2 py-1.5 text-xs">
                                <div className="min-w-0">
                                  <p className="truncate font-medium">{entry.name}</p>
                                  <p className="truncate text-[10px] text-muted-foreground">{entry.id}</p>
                                </div>
                                <div className="flex items-center gap-1">
                                  {entry.source_url ? (
                                    <Tooltip>
                                      <TooltipTrigger asChild>
                                        <Button
                                          type="button"
                                          size="icon-sm"
                                          variant="ghost"
                                          className="h-6 w-6"
                                          onClick={() => {
                                            void openUrl(entry.source_url as string);
                                          }}
                                        >
                                          <ExternalLink className="size-3.5" />
                                        </Button>
                                      </TooltipTrigger>
                                      <TooltipContent>Open source docs</TooltipContent>
                                    </Tooltip>
                                  ) : null}
                                  <Button
                                    type="button"
                                    size="xs"
                                    variant="outline"
                                    disabled={installed}
                                    onClick={() => handleInstallCatalogMcpEntry(entry)}
                                  >
                                    {installed ? 'Installed' : 'Add'}
                                  </Button>
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    )}

                    {(mcpExplorerFilter === 'registry' || mcpExplorerFilter === 'all') && (
                      <div className="space-y-2 rounded-md border bg-muted/15 p-2.5">
                        <div className="flex items-center justify-between gap-2">
                          <p className="text-[11px] font-semibold">Registry</p>
                          {mcpRegistryQuery.isFetching ? (
                            <Badge variant="outline" className="text-[10px]">Searching…</Badge>
                          ) : null}
                        </div>
                        {mcpCatalogInput.trim().length < 2 ? (
                          <p className="text-[11px] text-muted-foreground">Type at least 2 characters to search the registry.</p>
                        ) : mcpRegistryQuery.isError ? (
                          <p className="text-[11px] text-amber-700 dark:text-amber-300">
                            Registry lookup unavailable right now.
                          </p>
                        ) : (
                          <div className="max-h-[48vh] space-y-1 overflow-y-auto pr-1">
                            {mcpRegistryEntries.map((entry) => {
                              const requiredEnv = entry.required_env ?? [];
                              const requiredHeaders = entry.required_headers ?? [];
                              const serverId = inferMcpServerId(mcpServerFromRegistry(entry)).toLowerCase();
                              const installed = installedMcpServerIdSet.has(serverId);
                              return (
                                <div key={`registry-${entry.id}`} className="flex items-center justify-between gap-2 rounded border bg-background/70 px-2 py-1.5 text-xs">
                                  <div className="min-w-0">
                                    <p className="truncate font-medium">{entry.title}</p>
                                    <p className="truncate text-[10px] text-muted-foreground">
                                      {entry.server_name} • {entry.transport} • v{entry.version}
                                    </p>
                                    {(requiredEnv.length > 0 || requiredHeaders.length > 0) && (
                                      <p className="truncate text-[10px] text-amber-700 dark:text-amber-300">
                                        Requires: {requiredEnv.length > 0 ? `${requiredEnv.length} env` : ''}{requiredEnv.length > 0 && requiredHeaders.length > 0 ? ', ' : ''}{requiredHeaders.length > 0 ? `${requiredHeaders.length} headers` : ''}
                                      </p>
                                    )}
                                  </div>
                                  <Button
                                    type="button"
                                    size="xs"
                                    variant="outline"
                                    disabled={installed}
                                    onClick={() => handleInstallRegistryEntry(entry)}
                                  >
                                    {installed ? 'Installed' : 'Add'}
                                  </Button>
                                </div>
                              );
                            })}
                            {mcpRegistryEntries.length === 0 ? (
                              <p className="py-2 text-[11px] text-muted-foreground">No registry matches for this query.</p>
                            ) : null}
                          </div>
                        )}
                      </div>
                    )}
                </div>
              </ExplorerDialog>

              <ExplorerDialog
                open={mcpDiagnosticsOpen}
                onOpenChange={setMcpDiagnosticsOpen}
                title="MCP Diagnostics"
                icon={<Wrench className="size-4 text-violet-500" />}
              >
                <div className="flex h-full min-h-0 flex-col gap-3">
                  <div className="flex flex-wrap items-center gap-2 rounded-md border bg-background/70 p-2.5">
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="h-8"
                      onClick={() => void mcpValidationQuery.refetch()}
                      disabled={mcpValidationQuery.isFetching}
                    >
                      {mcpValidationQuery.isFetching ? 'Validating…' : 'Run Validate'}
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="h-8"
                      onClick={() => void mcpProbeQuery.refetch()}
                      disabled={(activeAgentConfig.mcp_servers ?? []).length === 0 || mcpProbeQuery.isFetching}
                    >
                      {mcpProbeQuery.isFetching ? 'Probing…' : 'Run Probe'}
                    </Button>
                    <span className="text-[11px] text-muted-foreground">
                      Validate checks config shape. Probe checks reachability and discovered tools.
                    </span>
                  </div>

                  <Tabs defaultValue="preflight" className="flex min-h-0 flex-1 flex-col gap-3">
                    <TabsList className="h-8 w-fit">
                      <TabsTrigger value="preflight" className="text-xs">Preflight</TabsTrigger>
                      <TabsTrigger value="runtime" className="text-xs">Runtime Probe</TabsTrigger>
                    </TabsList>

                    <TabsContent value="preflight" className="min-h-0 flex-1">
                      <div className="flex h-full min-h-0 flex-col gap-2 rounded-md border bg-background/70 p-3">
                        {mcpValidationQuery.isError ? (
                          <p className="text-[11px] text-destructive">{String(mcpValidationQuery.error)}</p>
                        ) : null}
                        {mcpValidationReport ? (
                          <>
                            <div className="flex flex-wrap items-center justify-between gap-2 text-[11px]">
                              <span className="font-semibold">
                                Preflight: {mcpValidationReport.ok ? 'ready' : 'needs attention'}
                              </span>
                              <span className="text-muted-foreground">
                                {mcpValidationReport.checked_servers} servers • {mcpValidationReport.checked_provider_configs} provider configs
                              </span>
                            </div>
                            {mcpValidationReport.issues.length === 0 ? (
                              <p className="text-[11px] text-emerald-700 dark:text-emerald-300">No issues found.</p>
                            ) : (
                              <div className="min-h-0 flex-1 space-y-1 overflow-y-auto pr-1">
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
                          </>
                        ) : (
                          <p className="text-[11px] text-muted-foreground">Run Validate to see preflight diagnostics.</p>
                        )}
                      </div>
                    </TabsContent>

                    <TabsContent value="runtime" className="min-h-0 flex-1">
                      <div className="flex h-full min-h-0 flex-col gap-2 rounded-md border bg-background/70 p-3">
                        {mcpProbeQuery.isError ? (
                          <p className="text-[11px] text-destructive">{String(mcpProbeQuery.error)}</p>
                        ) : null}
                        {mcpProbeReport ? (
                          <>
                            <div className="flex flex-wrap items-center justify-between gap-2 text-[11px]">
                              <span className="font-semibold">
                                Runtime probe: {mcpProbeReport.reachable_servers}/{mcpProbeReport.checked_servers} reachable • {mcpProbeReport.discovered_tools} tools
                              </span>
                              <span className="text-muted-foreground">{formatEpochSeconds(mcpProbeReport.generated_at)}</span>
                            </div>
                            {(mcpProbeReport.results ?? []).length === 0 ? (
                              <p className="text-[11px] text-muted-foreground">No probe results yet.</p>
                            ) : (
                              <div className="min-h-0 flex-1 space-y-1 overflow-y-auto pr-1">
                                {(mcpProbeReport.results ?? []).map((result) => {
                                  const probeTools = result.discovered_tools ?? [];
                                  const probeWarnings = result.warnings ?? [];
                                  return (
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
                                          {result.status.toUpperCase()}
                                        </Badge>
                                      </div>
                                      <p className="text-muted-foreground">
                                        {result.transport} • {probeTools.length} tools • {result.duration_ms}ms
                                      </p>
                                      {result.message ? <p>{result.message}</p> : null}
                                      {probeWarnings.slice(0, 1).map((warning) => (
                                        <p key={warning} className="text-amber-700 dark:text-amber-300">{warning}</p>
                                      ))}
                                    </div>
                                  );
                                })}
                              </div>
                            )}
                          </>
                        ) : (
                          <p className="text-[11px] text-muted-foreground">Run Probe to inspect runtime diagnostics.</p>
                        )}
                      </div>
                    </TabsContent>
                  </Tabs>
                </div>
              </ExplorerDialog>

              <div className="divide-y divide-border/50">
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

                {(activeAgentConfig.mcp_servers ?? []).length === 0 && mcpEditDraft === null && (
                  <div className="flex flex-col items-center gap-2 px-4 py-8 text-center">
                    <Package className="size-8 text-muted-foreground opacity-30" />
                    <p className="text-sm text-muted-foreground">No MCP servers configured.</p>
                  </div>
                )}

                {(activeAgentConfig.mcp_servers ?? []).map((server, idx) => {
                  const serverId = (server.id ?? server.name).trim();
                  const normalizedServerId = serverId || inferMcpServerId(server);
                  const transport = server.server_type ?? 'stdio';
                  const envCount = server.env ? Object.keys(server.env).length : 0;
                  const probeResult = mcpProbeByServerId.get(normalizedServerId);
                  const probeStatusLabel = probeResult?.status === 'needs-attention'
                    ? 'Issue'
                    : probeResult?.status === 'partial'
                      ? 'Partial'
                      : null;
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
                  const allToolsBlocked = isMcpServerDenied(activePermissions ?? undefined, normalizedServerId);
                  return (
                    <div key={`${normalizedServerId}-${idx}`} className={cn('transition-colors', mcpEditDraft?.idx === idx && 'bg-muted/30')}>
                      <div
                        className="flex cursor-pointer items-center gap-3 px-4 py-3 transition-colors hover:bg-muted/30"
                        role="button"
                        tabIndex={0}
                        onClick={() => setMcpEditDraft({ idx, server: { ...server } })}
                        onKeyDown={(event) => {
                          if (event.key === 'Enter' || event.key === ' ') {
                            event.preventDefault();
                            setMcpEditDraft({ idx, server: { ...server } });
                          }
                        }}
                      >
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
                            {MCP_STDIO_ONLY_ALPHA && transport !== 'stdio' && (
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Badge variant="secondary" className="cursor-default px-1.5 py-0 text-[9px]">
                                    discovery-only
                                  </Badge>
                                </TooltipTrigger>
                                <TooltipContent>
                                  HTTP/SSE endpoints are validated and probed, but active stdio execution is the current alpha path.
                                </TooltipContent>
                              </Tooltip>
                            )}
                            {probeResult && probeStatusLabel ? (
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Badge
                                    variant="outline"
                                    className={cn(
                                      'cursor-default px-1.5 py-0 text-[10px]',
                                      probeResult.status === 'partial'
                                          ? 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300'
                                        : 'border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-300'
                                    )}
                                  >
                                    {probeStatusLabel}
                                  </Badge>
                                </TooltipTrigger>
                                <TooltipContent className="max-w-xs">
                                  {probeResult.message || 'Open diagnostics for details.'}
                                </TooltipContent>
                              </Tooltip>
                            ) : null}
                            {discoveredTools.length > 0 && (
                              <Badge variant="secondary" className="cursor-default px-1.5 py-0 text-[9px]">
                                {discoveredTools.length} tools
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
                          {discoveredTools.length > 0 && (
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  variant={allToolsBlocked ? 'secondary' : 'outline'}
                                  size="xs"
                                  className="h-6 px-2 text-[10px]"
                                  onClick={(event) => {
                                    event.stopPropagation();
                                    handleToggleServerToolBlock(normalizedServerId);
                                  }}
                                  disabled={!activePermissions || savePermissionsMut.isPending}
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
                                className="h-6 w-6 p-0 text-destructive hover:bg-destructive/10 hover:text-destructive"
                                onClick={(event) => {
                                  event.stopPropagation();
                                  handleRemoveMcpServer(idx);
                                }}
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
                          <p className="text-[11px] font-medium">Discovered Tools</p>
                          <div className="mt-1.5 flex flex-wrap gap-1">
                            {discoveredTools.slice(0, 24).map((tool) => {
                              const denied = isMcpToolDenied(activePermissions ?? undefined, normalizedServerId, tool.name);
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
                                      disabled={!activePermissions || savePermissionsMut.isPending}
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
                      {mcpEditDraft?.idx === idx && (
                        <McpServerForm
                          draft={mcpEditDraft.server}
                          onChange={(s) => setMcpEditDraft({ idx, server: s })}
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

              </div>

            </Card>
          </div>
        )}

        {/* ════════════════════════════════════════════════════════════════
            SKILLS / RULES
        ════════════════════════════════════════════════════════════════ */}
        {(initialSection === 'skills' || initialSection === 'rules') && activeDocKind && (
          <div className="grid gap-4">
            <div className="grid gap-5 md:grid-cols-[320px_minmax(0,1fr)] xl:grid-cols-[360px_minmax(0,1fr)]">
              <div className="space-y-3">
                <div className="flex items-center gap-3 px-1">
                  <div className="flex size-7 items-center justify-center rounded-lg border border-cyan-500/20 bg-cyan-500/10">
                    {initialSection === 'skills' ? <BookOpen className="size-3.5 text-cyan-500" /> : <ScrollText className="size-3.5 text-cyan-500" />}
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold">{initialSection === 'skills' ? 'Skills' : 'Rules'}</h3>
                    <p className="text-[11px] text-muted-foreground">{initialSection === 'skills' ? `${agentScope} scope` : 'global scope'}</p>
                  </div>
                </div>

                <div className="grid gap-2 sm:grid-cols-2 md:grid-cols-1">
                  <Button variant="outline" size="sm" className="w-full justify-start" onClick={() => handleCreateDoc(activeDocKind)}>
                    <Plus className="size-3.5" />
                    New {initialSection === 'skills' ? 'Skill' : 'Rule'}
                  </Button>
                  {initialSection === 'skills' ? (
                    <Button
                      variant="outline"
                      size="sm"
                      className="w-full justify-start"
                      onClick={() => setSkillExplorerOpen(true)}
                    >
                      <Download className="size-3.5" />
                      Discover
                    </Button>
                  ) : null}
                </div>

                <div className="space-y-1">
                  {activeDocs.length === 0 ? (
                    <p className="py-4 text-center text-xs text-muted-foreground">
                      No {initialSection === 'skills' ? 'skills' : 'rules'} yet.
                    </p>
                  ) : (
                    initialSection === 'skills' ? (
                      <div className="max-h-[62vh] overflow-y-auto rounded-lg border bg-background/60">
                        {skillFolderRows.map((skill) => {
                          const selected = activeDoc?.id === skill.id;
                          return (
                            <div key={skill.id} className="border-b last:border-b-0">
                              <button
                                type="button"
                                className={cn(
                                  'flex w-full items-center gap-2 px-2.5 py-2 text-left text-xs font-medium',
                                  selected ? 'bg-primary/10 text-primary' : 'hover:bg-muted/40'
                                )}
                                onClick={() => selectActiveDoc(activeDocKind, skill.id)}
                              >
                                <Folder className="size-3.5 opacity-80" />
                                <span className="truncate">{skill.id}</span>
                              </button>
                              <button
                                type="button"
                                className={cn(
                                  'flex w-full items-center gap-2 px-2.5 py-1.5 pl-8 text-left text-xs',
                                  selected ? 'bg-primary/5 text-primary' : 'text-muted-foreground hover:bg-muted/30'
                                )}
                                onClick={() => selectActiveDoc(activeDocKind, skill.id)}
                              >
                                <ScrollText className="size-3.5 opacity-70" />
                                <span>{skill.fileName}</span>
                                <span className="truncate opacity-70">· {skill.title}</span>
                              </button>
                            </div>
                          );
                        })}
                      </div>
                    ) : (
                      activeDocs.map((doc) => {
                        const selected = activeDoc?.id === doc.id;
                        return (
                          <button
                            key={doc.id}
                            type="button"
                            className={cn(
                              'w-full rounded-md px-2.5 py-2 text-left transition-colors',
                              selected ? 'bg-primary/10 text-primary' : 'hover:bg-muted/50'
                            )}
                            onClick={() => selectActiveDoc(activeDocKind, doc.id)}
                          >
                            <p className="truncate text-sm font-medium">{doc.title || 'Untitled'}</p>
                          </button>
                        );
                      })
                    )
                  )}
                </div>
              </div>

              <div className="space-y-3">
                <div className="flex items-center gap-3 px-1">
                  <div className="flex size-7 items-center justify-center rounded-lg border border-indigo-500/20 bg-indigo-500/10">
                    <PenLine className="size-3.5 text-indigo-500" />
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-1.5 text-sm font-semibold">
                      <span className="opacity-50 font-normal">{initialSection === 'skills' ? 'Skills' : 'Rules'}</span>
                      <span className="opacity-30">/</span>
                      <span>{initialSection === 'skills' ? 'Skill Editor' : 'Rules Editor'}</span>
                      <Badge variant="outline" className="ml-2 h-4 px-1.5 py-0 text-[10px] font-normal normal-case tracking-tight opacity-70">
                        {agentScope} scope
                      </Badge>
                    </div>
                    <p className="text-[11px] text-muted-foreground">{initialSection === 'skills' ? 'Edit skill content' : 'Edit rule content'}</p>
                  </div>
                  <Button
                    variant="outline"
                    size="xs"
                    className="h-7"
                    onClick={() => void handleSave()}
                    disabled={agentScope === 'project' && !projectConfig}
                  >
                    <Save className="mr-1 size-3.5" />
                    Save {agentScope === 'project' ? 'Project' : 'Global'}
                  </Button>
                  {activeDoc ? (
                    <Button
                      variant="ghost"
                      size="xs"
                      className="text-muted-foreground opacity-70 hover:bg-muted hover:text-destructive"
                      onClick={() => handleDeleteDoc(activeDocKind, activeDoc.id)}
                    >
                      <Trash2 className="mr-1 size-3.5" />
                      Delete
                    </Button>
                  ) : null}
                </div>
                {!activeDoc ? (
                  <div className="flex h-[440px] flex-col items-center justify-center gap-2 text-center">
                    <ScrollText className="size-8 text-muted-foreground opacity-30" />
                    <p className="text-sm text-muted-foreground">Select or create a document to start editing.</p>
                  </div>
                ) : (
                  <div className="space-y-3">
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
                      rows={36}
                      className="min-h-[72vh]"
                      editorClassName={initialSection === 'rules' ? '!border-0 !rounded-none !bg-transparent' : undefined}
                      defaultMode="edit"
                      showFrontmatter={false}
                      showStats={false}
                      fillHeight={false}
                    />
                  </div>
                )}
              </div>
            </div>

            {initialSection === 'skills' ? (
              <ExplorerDialog
                open={skillExplorerOpen}
                onOpenChange={setSkillExplorerOpen}
                title="Discover Skills"
                icon={<BookOpen className="size-4 text-cyan-500" />}
              >
                <Tabs defaultValue="skills-sh" className="flex h-full min-h-0 flex-col gap-3">
                  <TabsList className="h-8 w-fit">
                    <TabsTrigger value="skills-sh" className="text-xs">skills.sh</TabsTrigger>
                    <TabsTrigger value="curated" className="text-xs">Curated Repo</TabsTrigger>
                  </TabsList>

                  <TabsContent value="skills-sh" className="min-h-0 flex-1">
                    <div className="flex h-full min-h-0 flex-col gap-3 rounded-md border bg-muted/15 p-3">
                      <div className="space-y-2">
                        <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto_auto]">
                          <Input
                            value={skillSourceInput}
                            onChange={(event) => setSkillSourceInput(event.target.value)}
                            placeholder="Paste full skills.sh command or skill ID"
                            className="h-9 text-xs font-mono"
                            autoCapitalize="none"
                            autoCorrect="off"
                            spellCheck={false}
                          />
                          <Button
                            type="button"
                            size="sm"
                            variant="secondary"
                            className="h-9"
                            onClick={() => handleInstallSkillFromSource()}
                            disabled={!canInstallFromSource || installSkillFromSourceMut.isPending}
                          >
                            {installSkillFromSourceMut.isPending ? 'Installing…' : (
                              <>
                                <Download className="mr-1 size-3.5" />
                                Install
                              </>
                            )}
                          </Button>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                variant="outline"
                                size="icon-sm"
                                aria-label="Open skills.sh externally"
                                onClick={() => {
                                  void openUrl('https://skills.sh');
                                }}
                              >
                                <ExternalLink className="size-3.5" />
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent>Open skills.sh (external)</TooltipContent>
                          </Tooltip>
                        </div>
                        <p className="text-[11px] text-muted-foreground">
                          {parsedSkillInstallSpec.parseHint ?? 'Use `npx skills add <skill-id>` or just `<skill-id>`.'}
                        </p>
                        {installSkillFromSourceMut.error ? (
                          <p className="text-[11px] text-destructive">{String(installSkillFromSourceMut.error)}</p>
                        ) : null}
                      </div>
                    </div>
                  </TabsContent>

                  <TabsContent value="curated" className="min-h-0 flex-1">
                    <div className="flex h-full min-h-0 flex-col items-center justify-center gap-2 rounded-md border border-dashed bg-muted/15 p-4 text-center">
                      <BookOpen className="size-5 text-muted-foreground/70" />
                      <p className="text-sm font-medium">Curated repository not connected yet.</p>
                      <p className="text-xs text-muted-foreground">
                        This tab will host Ship-curated skills as a first-class discovery source.
                      </p>
                    </div>
                  </TabsContent>
                </Tabs>
              </ExplorerDialog>
            ) : null}
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
          <div className="grid gap-4">
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
                          Presets apply a curated bundle of tool allow/deny rules. They overwrite your current permissions — customize further after applying.
                        </TooltipContent>
                      </Tooltip>
                    </div>
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
                            onClick={() => {
                              const next = normalizePermissionsForEditor(preset.apply());
                              setPermissionsDraftByScope((current) => ({ ...current, [agentScope]: next }));
                              setPermissionsDirtyByScope((current) => ({ ...current, [agentScope]: true }));
                            }}
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
                  </div>
                  <div className="flex items-center gap-2">
                    {discoveryCache && (
                      <Badge variant="outline" className="text-[10px]">
                        {discoveryCache.shell_commands.length} commands • {discoveryCache.filesystem_paths.length} paths
                      </Badge>
                    )}
                    {permissionsDirty ? (
                      <Badge className="bg-amber-500/20 text-amber-800 hover:bg-amber-500/20 dark:text-amber-300">
                        Unsaved policy changes
                      </Badge>
                    ) : null}
                    <Button
                      type="button"
                      variant="outline"
                      size="xs"
                      className="h-6 px-2 text-[10px]"
                      onClick={() => savePermissionsDraft()}
                      disabled={!activePermissions || !permissionsDirty || savePermissionsMut.isPending}
                    >
                      <Save className="mr-1 size-3" />
                      {savePermissionsMut.isPending ? 'Saving…' : 'Save Policy'}
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="xs"
                      className="h-6 px-2 text-[10px]"
                      onClick={() => refreshDiscoveryCacheMut.mutate()}
                      disabled={refreshDiscoveryCacheMut.isPending}
                    >
                      {refreshDiscoveryCacheMut.isPending ? 'Refreshing…' : 'Refresh detection'}
                    </Button>
                  </div>
                </div>
                <CardContent className="space-y-6 !pt-5">
                  {!activePermissions ? (
                    <p className="py-10 text-center text-sm text-muted-foreground">Loading permissions...</p>
                  ) : (
                      <Tabs value={permissionsTab} onValueChange={(value) => setPermissionsTab(value as 'tools' | 'commands' | 'filesystem')}>
                      <TabsList className="mb-4">
                        <TabsTrigger value="tools">MCP Tools</TabsTrigger>
                        <TabsTrigger value="commands">Shell Commands</TabsTrigger>
                        <TabsTrigger value="filesystem">Filesystem</TabsTrigger>
                      </TabsList>
                      {permissionValidationIssues.length > 0 && (
                        <div className="mb-3 rounded border border-amber-500/30 bg-amber-500/10 px-2.5 py-2 text-[11px] text-amber-700 dark:text-amber-300">
                          <p className="font-medium">Validation warnings: {permissionValidationIssues.length}</p>
                          <p className="mt-0.5">
                            {permissionValidationIssues[0]}
                            {permissionValidationIssues.length > 1 ? ' (fix highlighted lists before saving)' : ''}
                          </p>
                        </div>
                      )}

                      <TabsContent value="tools" className="space-y-6">
                        <p className="text-[11px] text-muted-foreground">
                          Built-in tools use plain IDs like <code>Edit</code>. MCP tools use <code>mcp__server__tool</code> patterns.
                        </p>
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
                            <PatternListEditor
                              patterns={toolAllowPatterns}
                              options={permissionToolSuggestions}
                              addLabel="Add Pattern"
                              addValue="mcp__"
                              noResultsText="Type a custom tool pattern."
                              onChange={(applyAllow) => {
                                updatePermissions((current) => ({
                                  ...current,
                                  tools: {
                                    ...current.tools,
                                    allow: applyAllow(current.tools?.allow || []),
                                    deny: current.tools?.deny || [],
                                  },
                                }));
                              }}
                            />
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
                                  Deny always overrides allow. Blocked tools will never execute even if they match an allow pattern. Built-in provider tools use plain IDs like <code>Edit</code> and <code>MultiEdit</code>.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <PatternListEditor
                              patterns={toolDenyPatterns}
                              options={permissionToolSuggestions}
                              addLabel="Add Pattern"
                              addValue="mcp__*__"
                              noResultsText="Type a custom restriction pattern."
                              onChange={(applyDeny) => {
                                updatePermissions((current) => ({
                                  ...current,
                                  tools: {
                                    ...current.tools,
                                    deny: applyDeny(current.tools?.deny || []),
                                    allow: current.tools?.allow || ['*'],
                                  },
                                }));
                              }}
                            />
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
                            <PatternListEditor
                              patterns={activePermissions.commands?.allow || []}
                              options={commandPatternSuggestions}
                              addLabel="Add Pattern"
                              noResultsText="Type a custom command pattern."
                              onChange={(applyAllow) => {
                                updatePermissions((current) => ({
                                  ...current,
                                  commands: {
                                    ...current.commands,
                                    allow: applyAllow(current.commands?.allow || []),
                                    deny: current.commands?.deny || [],
                                  },
                                }));
                              }}
                            />
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
                            <PatternListEditor
                              patterns={activePermissions.commands?.deny || []}
                              options={commandPatternSuggestions}
                              addLabel="Add Pattern"
                              noResultsText="Type a custom blocked command."
                              onChange={(applyDeny) => {
                                updatePermissions((current) => ({
                                  ...current,
                                  commands: {
                                    ...current.commands,
                                    deny: applyDeny(current.commands?.deny || []),
                                    allow: current.commands?.allow || [],
                                  },
                                }));
                              }}
                            />
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
                            <PatternListEditor
                              patterns={activePermissions.agent?.require_confirmation || []}
                              options={commandPatternSuggestions}
                              addLabel="Add Pattern"
                              noResultsText="Type a command requiring approval."
                              onChange={(applyRequireConfirmation) => {
                                updatePermissions((current) => ({
                                  ...current,
                                  agent: {
                                    ...current.agent,
                                    require_confirmation: applyRequireConfirmation(current.agent?.require_confirmation || []),
                                  },
                                }));
                              }}
                            />
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
                                  Glob patterns for paths the agent can read and write. Prefer scoped directories like <code>~/projects/**</code> and add exceptions explicitly.
                                </TooltipContent>
                              </Tooltip>
                            </div>
                            <PatternListEditor
                              patterns={activePermissions.filesystem?.allow || []}
                              options={filesystemPathSuggestions}
                              addLabel="Add Path"
                              noResultsText="Type a custom path pattern."
                              onChange={(applyAllow) => {
                                updatePermissions((current) => ({
                                  ...current,
                                  filesystem: {
                                    ...current.filesystem,
                                    allow: applyAllow(current.filesystem?.allow || []),
                                    deny: current.filesystem?.deny || [],
                                  },
                                }));
                              }}
                            />
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
                            <PatternListEditor
                              patterns={activePermissions.filesystem?.deny || []}
                              options={filesystemPathSuggestions}
                              addLabel="Add Pattern"
                              noResultsText="Type a custom blocked path."
                              onChange={(applyDeny) => {
                                updatePermissions((current) => ({
                                  ...current,
                                  filesystem: {
                                    ...current.filesystem,
                                    deny: applyDeny(current.filesystem?.deny || []),
                                    allow: current.filesystem?.allow || [],
                                  },
                                }));
                              }}
                            />
                          </div>
                        </div>
                      </TabsContent>

                    </Tabs>
                  )}
                </CardContent>
              </Card>
            </div>
          </div>
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
