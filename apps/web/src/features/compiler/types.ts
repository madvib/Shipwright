// Re-export shared types from @ship/ui
export type {
  McpServerConfig,
  Skill,
  Rule,
  Permissions,
  ModeConfig,
  ProjectLibrary,
} from '@ship/ui'
export { DEFAULT_PERMISSIONS, DEFAULT_LIBRARY } from '@ship/ui'

// Compile output types (web-specific — from @ship/compiler WASM)
export interface CompileResult {
  provider: string
  context_content: string | null
  mcp_servers: Record<string, unknown> | null
  mcp_config_path: string | null
  skill_files: Record<string, string>
  rule_files: Record<string, string>
  claude_settings_patch: Record<string, unknown> | null
  codex_config_patch: string | null
  gemini_settings_patch: Record<string, unknown> | null
  gemini_policy_patch: string | null
  cursor_hooks_patch: Record<string, unknown> | null
  cursor_cli_permissions: Record<string, unknown> | null
}

export type CompileOutput = Record<string, CompileResult>

// Provider metadata
export interface Provider {
  id: string
  name: string
  description: string
  files: string[]
}

export const PROVIDERS: Provider[] = [
  {
    id: 'claude',
    name: 'Claude Code',
    description: "Anthropic's AI coding assistant",
    files: ['CLAUDE.md', '.mcp.json', '.claude/settings.json'],
  },
  {
    id: 'gemini',
    name: 'Gemini CLI',
    description: "Google's AI coding assistant",
    files: ['GEMINI.md', '.gemini/settings.json'],
  },
  {
    id: 'codex',
    name: 'Codex CLI',
    description: "OpenAI's AI coding assistant",
    files: ['AGENTS.md', '.codex/config.toml'],
  },
  {
    id: 'cursor',
    name: 'Cursor',
    description: 'AI-first code editor by Anysphere',
    files: ['AGENTS.md', '.cursor/mcp.json', '.cursor/rules/'],
  },
]
