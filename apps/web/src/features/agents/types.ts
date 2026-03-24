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
    // Agent-level fields compiled per-provider
    model?: string | null
    env?: Record<string, string> | null
    availableModels?: string[] | null
    agentLimits?: { max_turns?: number; max_cost_per_session?: number } | null
    // UI-only persisted state — not in compiler schema
    providerSettings?: Record<string, Record<string, unknown>>
    toolPermissions?: Record<string, ToolToggleState>
    /** "project" | "library" — set on pull, used for filtering. */
    source?: string
  }

// UI-only types — not in the compiler schema.
export type ToolPermission = 'allow' | 'ask' | 'deny'

export interface ToolToggleState {
  [toolName: string]: ToolPermission
}

