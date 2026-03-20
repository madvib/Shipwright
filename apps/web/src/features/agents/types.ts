// ── Agent profile types ──────────────────────────────────────────────────────
// Local types for the agent detail page. These extend @ship/ui types
// with agent-specific metadata not yet in the shared package.

import type { McpServerConfig, Skill, Rule, Permissions } from '@ship/ui'

export type ToolPermission = 'allow' | 'ask' | 'deny'

export interface ToolToggleState {
  [toolName: string]: ToolPermission
}

export interface McpToolConfig {
  name: string
  description: string
  group: 'read' | 'write' | 'admin'
}

export interface HookConfig {
  trigger: string
  command: string
  providers: string[]
}

export interface AgentSettings {
  model: string
  defaultMode: string
  extendedThinking: boolean
  autoMemory: boolean
}

export interface AgentProfile {
  id: string
  name: string
  description: string
  providers: string[]
  version: string
  skills: Skill[]
  mcpServers: McpServerConfig[]
  subagents: SubagentRef[]
  permissions: Permissions
  permissionPreset: string
  settings: AgentSettings
  hooks: HookConfig[]
  rules: Rule[]
  mcpToolStates: Record<string, ToolToggleState>
}

export interface SubagentRef {
  id: string
  name: string
  description: string
}

// ── Default data ─────────────────────────────────────────────────────────────

export const DEFAULT_SETTINGS: AgentSettings = {
  model: 'claude-sonnet-4-6',
  defaultMode: 'default',
  extendedThinking: true,
  autoMemory: false,
}

export const DEMO_AGENT: AgentProfile = {
  id: 'web-lane',
  name: 'web-lane',
  description: 'Web lane specialist for apps/web/. Active context for web feature work.',
  providers: ['claude', 'gemini'],
  version: 'v0.1.0',
  skills: [
    { id: 'ship-coordination', name: 'ship-coordination', content: '', source: 'custom' },
    { id: 'code-review', name: 'code-review', content: '', source: 'community' },
    { id: 'debug-expert', name: 'debug-expert', content: '', source: 'community' },
    { id: 'frontend-design', name: 'frontend-design', content: '', source: 'custom' },
    { id: 'vercel-react', name: 'vercel-react', content: '', source: 'community' },
  ],
  mcpServers: [
    { name: 'ship', command: 'ship', args: ['mcp', 'serve'], server_type: 'stdio', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
    { name: 'github', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'], server_type: 'stdio', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
    { name: 'filesystem', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '.'], server_type: 'stdio', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
  ],
  subagents: [
    { id: 'react-designer', name: 'react-designer', description: 'UI components, shadcn, Tailwind' },
    { id: 'react-architect', name: 'react-architect', description: 'Data flow, TanStack, state' },
  ],
  permissions: {
    tools: { allow: ['Read', 'Grep', 'Glob', 'Bash(git *)'], deny: ['Bash(rm -rf *)'] },
    filesystem: { allow: ['apps/web/**'], deny: ['.env', 'credentials.*'] },
    commands: { allow: ['git status', 'pnpm *'], deny: ['git push --force'] },
    network: { policy: 'allow-list', allow_hosts: ['localhost', 'api.github.com'] },
    agent: { require_confirmation: [] },
  },
  permissionPreset: 'ship-guarded',
  settings: { ...DEFAULT_SETTINGS },
  hooks: [
    { trigger: 'PreToolUse', command: './scripts/check-no-compat-surface.sh', providers: ['claude', 'gemini'] },
    { trigger: 'Stop', command: 'ship mcp sync-permissions', providers: ['claude'] },
    { trigger: 'Notification', command: "notify-send 'Agent' '$MESSAGE'", providers: ['claude'] },
  ],
  rules: [
    { file_name: '010-no-compat.md', content: 'No backward compatibility without consumers. Make hard breaks...' },
    { file_name: '020-test-policy.md', content: 'Add or update tests for every bug fix and behavior change...' },
  ],
  mcpToolStates: {
    github: {
      get_file_contents: 'allow',
      search_code: 'allow',
      search_repositories: 'allow',
      list_issues: 'allow',
      get_issue: 'allow',
      get_pull_request: 'allow',
      list_pull_requests: 'allow',
      get_pull_request_diff: 'allow',
      create_issue: 'ask',
      create_pull_request: 'deny',
      merge_pull_request: 'deny',
      push_files: 'deny',
      create_or_update_file: 'deny',
      create_repository: 'deny',
      fork_repository: 'deny',
      create_branch: 'deny',
      update_issue: 'deny',
      add_issue_comment: 'deny',
    },
  },
}

export const GITHUB_TOOLS: McpToolConfig[] = [
  { name: 'get_file_contents', description: 'Read file content from a repository', group: 'read' },
  { name: 'search_code', description: 'Search for code across repositories', group: 'read' },
  { name: 'search_repositories', description: 'Search for GitHub repositories', group: 'read' },
  { name: 'list_issues', description: 'List issues in a repository', group: 'read' },
  { name: 'get_issue', description: 'Get details of a specific issue', group: 'read' },
  { name: 'get_pull_request', description: 'Get details of a pull request', group: 'read' },
  { name: 'list_pull_requests', description: 'List pull requests in a repository', group: 'read' },
  { name: 'get_pull_request_diff', description: 'Get the diff of a pull request', group: 'read' },
  { name: 'create_issue', description: 'Create a new issue in a repository', group: 'write' },
  { name: 'create_pull_request', description: 'Create a new pull request', group: 'write' },
  { name: 'merge_pull_request', description: 'Merge a pull request', group: 'write' },
  { name: 'push_files', description: 'Push files to a repository branch', group: 'write' },
  { name: 'create_or_update_file', description: 'Create or update a file in a repository', group: 'write' },
  { name: 'create_repository', description: 'Create a new repository', group: 'admin' },
  { name: 'fork_repository', description: 'Fork a repository to your account', group: 'admin' },
  { name: 'create_branch', description: 'Create a new branch in a repository', group: 'admin' },
  { name: 'update_issue', description: 'Update an existing issue', group: 'admin' },
  { name: 'add_issue_comment', description: 'Add a comment to an issue', group: 'admin' },
]

// Tool configs for demo servers
export const MCP_TOOL_REGISTRY: Record<string, McpToolConfig[]> = {
  github: GITHUB_TOOLS,
  ship: [
    { name: 'open_project', description: 'Set active project for MCP calls', group: 'read' },
    { name: 'list_workspaces', description: 'List all workspaces', group: 'read' },
    { name: 'list_jobs', description: 'List coordination jobs', group: 'read' },
    { name: 'start_session', description: 'Start a workspace session', group: 'write' },
    { name: 'end_session', description: 'End the active session', group: 'write' },
    { name: 'log_progress', description: 'Record progress note', group: 'write' },
    { name: 'create_job', description: 'Create a coordination job', group: 'write' },
    { name: 'create_adr', description: 'Create architecture decision record', group: 'admin' },
  ],
  filesystem: [
    { name: 'read_file', description: 'Read a file from the filesystem', group: 'read' },
    { name: 'list_directory', description: 'List directory contents', group: 'read' },
    { name: 'search_files', description: 'Search for files matching a pattern', group: 'read' },
    { name: 'get_file_info', description: 'Get metadata about a file', group: 'read' },
    { name: 'write_file', description: 'Write content to a file', group: 'write' },
    { name: 'create_directory', description: 'Create a new directory', group: 'write' },
    { name: 'move_file', description: 'Move or rename a file', group: 'admin' },
    { name: 'delete_file', description: 'Delete a file', group: 'admin' },
  ],
}
