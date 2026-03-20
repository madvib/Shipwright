// ── Generated types (source of truth: Rust crate `compiler`) ─────────────────
// Re-exported from the specta-generated file. Run `cargo xtask gen-types` to
// regenerate after changing Rust types.

export type {
  AgentLayerConfig,
  AgentLimits,
  AgentProfile,
  AiConfig,
  CatalogEntry,
  CatalogKind,
  CommandPermissions,
  CompileOutput,
  FsPermissions,
  GitConfig,
  HookConfig,
  HookTrigger,
  McpRefs,
  McpServerConfig,
  McpServerType,
  ModeConfig,
  NamespaceConfig,
  NetworkPermissions,
  NetworkPolicy,
  PermissionConfig,
  Permissions,
  PluginEntry,
  PluginRefs,
  PluginsManifest,
  ProfileMeta,
  ProfilePermissions,
  ProfileRules,
  ProjectConfig,
  ProjectLibrary,
  Rule,
  Skill,
  SkillRefs,
  SkillSource,
  StatusConfig,
  ToolPermissions,
} from './generated'

// ── MCP Registry types (not from Rust — external schema) ─────────────────────

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

// ── Runtime defaults ─────────────────────────────────────────────────────────

import type { Permissions, ProjectLibrary } from './generated'

export const DEFAULT_PERMISSIONS: Permissions = {
  tools: { allow: [], deny: [] },
  filesystem: { allow: ['**/*'], deny: [] },
  commands: { allow: [], deny: [] },
  network: { policy: 'none', allow_hosts: [] },
  agent: { require_confirmation: [] },
}

export const DEFAULT_LIBRARY: ProjectLibrary = {
  modes: [],
  active_agent: null,
  mcp_servers: [],
  skills: [],
  rules: [],
  permissions: DEFAULT_PERMISSIONS,
  hooks: [],
  plugins: { install: [], scope: 'project' },
  agent_profiles: [],
  claude_team_agents: [],
  env: {},
  available_models: [],
}
