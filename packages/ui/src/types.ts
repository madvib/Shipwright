// ── Shared agent config types ────────────────────────────────────────────────
// These mirror the Rust types in crates/core/compiler and crates/core/runtime.

export interface McpServerConfig {
  name: string
  command: string
  args?: string[]
  env?: Record<string, string>
  server_type?: 'stdio' | 'sse' | 'http'
  scope?: 'global' | 'project'
  disabled?: boolean
  url?: string | null
  timeout_secs?: number | null
}

export interface Skill {
  id: string
  name: string
  content: string
  description?: string | null
  source?: string
  author?: string | null
  version?: string | null
}

export interface Rule {
  file_name: string
  content: string
}

export interface Permissions {
  tools: { allow: string[]; deny: string[] }
  filesystem: { allow: string[]; deny: string[] }
  commands: { allow: string[]; deny: string[] }
  network: {
    policy: 'none' | 'localhost' | 'allow-list' | 'unrestricted'
    allow_hosts: string[]
  }
  agent: { require_confirmation: string[] }
}

export interface ModeConfig {
  name: string
  description?: string | null
  mcp_servers?: McpServerConfig[]
  skills?: Skill[]
  rules?: Rule[]
  permissions?: Permissions
  active_tools?: string[]
}

export interface ProjectLibrary {
  modes: ModeConfig[]
  active_mode?: string | null
  mcp_servers: McpServerConfig[]
  skills: Skill[]
  rules: Rule[]
  permissions?: Permissions | null
}

// ── MCP Registry types ───────────────────────────────────────────────────────

export interface McpRegistryServer {
  id: string
  name: string
  description?: string
  homepage?: string
  repository?: string
  license?: string
  tags?: string[]
  package?: {
    registry?: string
    name?: string
    version?: string
    command?: string
    args?: string[]
  }
  vendor?: {
    name?: string
    url?: string
  }
}

// ── Defaults ─────────────────────────────────────────────────────────────────

export const DEFAULT_PERMISSIONS: Permissions = {
  tools: { allow: [], deny: [] },
  filesystem: { allow: ['**/*'], deny: [] },
  commands: { allow: [], deny: [] },
  network: { policy: 'none', allow_hosts: [] },
  agent: { require_confirmation: [] },
}

export const DEFAULT_LIBRARY: ProjectLibrary = {
  modes: [],
  active_mode: null,
  mcp_servers: [],
  skills: [],
  rules: [],
  permissions: DEFAULT_PERMISSIONS,
}
