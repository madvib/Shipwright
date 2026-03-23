// UI-only types for the agent editor.
// Compiler types come from @ship/ui — import them directly where needed.

import type {
  AgentProfile,
  Skill,
  McpServerConfig,
  Rule,
  HookConfig,
} from '@ship/ui'

// The compiled AgentProfile stores refs (skill IDs, server names).
// The UI needs resolved objects for rendering. This type replaces the
// ref fields with their resolved counterparts — everything else is inherited.
export type ResolvedAgentProfile =
  Omit<AgentProfile, 'skills' | 'mcp' | 'rules'> & {
    skills: Skill[]
    mcpServers: McpServerConfig[]
    rules: Rule[]
    hooks: HookConfig[]
  }

// UI-only types — not in the compiler schema.
export type ToolPermission = 'allow' | 'ask' | 'deny'

export interface ToolToggleState {
  [toolName: string]: ToolPermission
}

export interface McpToolConfig {
  name: string
  description: string
  group: 'read' | 'write' | 'admin'
}

// Demo data for MCP tool panels (tech debt — should come from runtime tool discovery)
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
