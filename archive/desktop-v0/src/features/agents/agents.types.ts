import type { CatalogEntry, McpServerConfig, McpServerType, McpValidationIssue, McpValidationReport, ProjectConfig, ProjectDiscovery, ProviderInfo } from '@/bindings';

// ── Shared types ─────────────────────────────────────────────────────────────

export interface AgentsPanelProps {
  activeProject?: ProjectDiscovery | null;
  projectConfig: ProjectConfig | null;
  globalAgentConfig: ProjectConfig | null;
  onSaveProject: (config: ProjectConfig) => void | Promise<void>;
  onSaveGlobalAgentConfig: (config: ProjectConfig) => void | Promise<void>;
  initialSection?: AgentSection;
  onBackToSettings?: () => void;
}

export type ScopeKey = 'global' | 'project';
export type MarkdownDocKind = 'skills' | 'rules';

export type AgentDoc = {
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

export const PROVIDER_LOGO: Record<string, { src: string; invertDark?: boolean }> = {
  claude: { src: '/provider-logos/claude.svg' },
  gemini: { src: '/provider-logos/googlegemini.svg' },
  codex: { src: '/provider-logos/OpenAI-black-monoblossom.svg', invertDark: true },
};

export const EMPTY_AGENT_LAYER = {
  skills: [],
  prompts: [],
  context: [],
  rules: [],
};

export const SECTION_META: Record<AgentSection, { title: string; description: string }> = {
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

export type HookEventOption = {
  value: string;
  label: string;
  providers: string[];
  matcherHint?: string;
  description?: string;
};

export const HOOK_EVENTS: HookEventOption[] = [
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

export const EMPTY_MCP_SERVER: McpServerConfig = {
  name: '',
  command: '',
  args: [],
  url: null,
  timeout_secs: null,
};

export type McpEditDraft = {
  idx: number | null;
  server: McpServerConfig;
};

export type ProviderRow = ProviderInfo & {
  checking: boolean;
};

export type ProviderSyncStatus = 'ready' | 'needs-attention' | 'drift-detected';

export type ProviderSyncSummary = {
  status: ProviderSyncStatus;
  detail: string;
  issues: McpValidationIssue[];
};

export type McpValidation = {
  level: 'info' | 'warning';
  message: string;
};

export const PROVIDER_DRIFT_CODES = new Set([
  'provider-config-root',
  'provider-config-mcp-key',
]);

export const PROVIDER_STATUS_COPY: Record<ProviderSyncStatus, string> = {
  ready: 'Ready',
  'needs-attention': 'Needs attention',
  'drift-detected': 'Drift detected',
};

export const SUPPORTED_PROVIDER_BASE: Array<{ id: string; name: string; binary: string }> = [
  { id: 'claude', name: 'Claude Code', binary: 'claude' },
  { id: 'gemini', name: 'Gemini CLI', binary: 'gemini' },
  { id: 'codex', name: 'Codex CLI', binary: 'codex' },
];
export const SUPPORTED_PROVIDER_IDS = new Set(SUPPORTED_PROVIDER_BASE.map((provider) => provider.id));
export const MCP_STDIO_ONLY_ALPHA = false;
export const EMPTY_CATALOG: CatalogEntry[] = [];
export const EMPTY_RULES: Array<{ file_name: string; content: string }> = [];

// ── Pure helper functions used across agents feature ─────────────────────────

export function slugifyId(value: string): string {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 64);
}

export function splitShellArgs(raw: string): string[] {
  const input = raw.trim();
  if (!input) return [];
  const matches = input.match(/(?:[^\s"']+|"[^"]*"|'[^']*')+/g) ?? [];
  return matches
    .map((segment) => segment.replace(/^['"]|['"]$/g, ''))
    .filter(Boolean);
}

export function getMcpTemplateValidation(server: McpServerConfig): McpValidation[] {
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

export function inferMcpServerId(server: McpServerConfig): string {
  const explicit = (server.id ?? '').trim();
  if (explicit) return slugifyId(explicit);
  const fromName = slugifyId(server.name || '');
  if (fromName) return fromName;
  if (server.command) return slugifyId(server.command);
  return `mcp-${Date.now()}`;
}

// ── Provider sync helpers ─────────────────────────────────────────────────────

export function getProviderSyncSummary(
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

export function providerStatusBadgeClass(status: ProviderSyncStatus): string {
  if (status === 'ready') {
    return 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300';
  }
  if (status === 'drift-detected') {
    return 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300';
  }
  return 'border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-300';
}
