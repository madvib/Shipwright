// Re-export shared types from @ship/ui (generated from Rust via specta)
export type {
  McpServerConfig,
  Skill,
  Rule,
  Permissions,
  ModeConfig,
  ProjectLibrary,
} from '@ship/ui'
export { DEFAULT_PERMISSIONS, DEFAULT_LIBRARY } from '@ship/ui'

// Compile output types — generated from Rust `CompileOutput`.
// The TS codebase calls the per-provider result "CompileResult".
import type { CompileOutput as _CompileOutput } from '@ship/ui'
export type CompileResult = _CompileOutput

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
  {
    id: 'opencode',
    name: 'OpenCode',
    description: 'Open-source AI coding assistant',
    files: ['opencode.json'],
  },
]
