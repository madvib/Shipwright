import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { McpSection } from './McpSection'
import type { McpServerConfig } from '@ship/ui'
import type { ToolToggleState } from '../types'

const meta: Meta<typeof McpSection> = {
  title: 'Agents/McpSection',
  component: McpSection,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof McpSection>

const nullFields = {
  url: null,
  timeout_secs: null,
  codex_enabled_tools: [],
  codex_disabled_tools: [],
  gemini_include_tools: [],
  gemini_exclude_tools: [],
}

const SERVERS: McpServerConfig[] = [
  { name: 'ship', command: 'ship', args: ['mcp', 'serve'], server_type: 'stdio', ...nullFields },
  { name: 'github', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'], server_type: 'stdio', ...nullFields },
  { name: 'filesystem', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '.'], server_type: 'stdio', ...nullFields },
]

const TOOL_STATES: Record<string, ToolToggleState> = {
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
}

/** Empty state -- no MCP servers attached. */
export const Empty: Story = {
  args: {
    servers: [],
    toolStates: {},
    onRemove: fn(),
    onSetToolPermission: fn(),
    onSetGroupPermission: fn(),
    onAdd: fn(),
  },
}

/** Three servers with tool toggle states for GitHub. */
export const WithServers: Story = {
  args: {
    servers: SERVERS,
    toolStates: TOOL_STATES,
    onRemove: fn(),
    onSetToolPermission: fn(),
    onSetGroupPermission: fn(),
    onAdd: fn(),
  },
}

/** Single server -- filesystem only. */
export const SingleServer: Story = {
  args: {
    servers: [SERVERS[2]],
    toolStates: {},
    onRemove: fn(),
    onSetToolPermission: fn(),
    onSetGroupPermission: fn(),
    onAdd: fn(),
  },
}
