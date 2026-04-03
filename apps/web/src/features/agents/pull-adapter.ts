// Converts PullAgent (from @ship/ui generated types) to ResolvedAgentProfile
// (the UI-resolved type used throughout the agent editor).

import type { PullAgent, Skill, McpServerConfig, Rule, HookConfig } from '@ship/ui'
import type { ResolvedAgentProfile } from './types'

export function pullAgentToResolved(pull: PullAgent): ResolvedAgentProfile {
  return {
    profile: {
      id: pull.profile.id,
      name: pull.profile.name,
      description: pull.profile.description,
      providers: pull.profile.providers,
      version: pull.profile.version,
    },
    skills: pull.skills.map(pullSkillToSkill),
    mcpServers: pull.mcpServers.map(pullMcpToConfig),
    rules: pull.rules.map(pullRuleToRule),
    hooks: (pull.hooks ?? []) as HookConfig[],
    model: pull.model ?? null,
    env: (pull.env as Record<string, string> | null) ?? null,
    availableModels: pull.available_models ?? null,
    agentLimits: (pull.agent_limits as { max_turns?: number; max_cost_per_session?: number } | null) ?? null,
    providerSettings: (pull.provider_settings ?? {}) as Record<string, Record<string, unknown>>,
    toolPermissions: {},
    source: pull.source ?? 'project',
  }
}

function pullSkillToSkill(s: { id: string; name: string; description?: string | null; content: string; source: string; artifacts?: string[]; tags?: string[]; authors?: string[]; files?: string[]; reference_docs?: Partial<{ [key in string]: string }> }): Skill {
  return {
    id: s.id,
    name: s.name,
    description: s.description ?? '',
    content: s.content,
    source: s.source as Skill['source'],
    artifacts: s.artifacts ?? [],
    vars: {},
  }
}

function pullMcpToConfig(s: { name: string; command: string; url?: string | null }): McpServerConfig {
  return {
    name: s.name,
    command: s.command,
    url: s.url ?? null,
    timeout_secs: null,
    codex_enabled_tools: [],
    codex_disabled_tools: [],
    gemini_include_tools: [],
    gemini_exclude_tools: [],
  }
}

function pullRuleToRule(r: { file_name: string; content: string }): Rule {
  return { file_name: r.file_name, content: r.content }
}
