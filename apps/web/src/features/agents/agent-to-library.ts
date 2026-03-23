// ── Agent-to-Library Merge ───────────────────────────────────────────────────
// Pure function that merges a web-app AgentProfile into a ProjectLibrary for
// compilation. When the preview panel compiles, it uses this to produce
// agent-aware output instead of raw library output.

import type { ProjectLibrary, ModeConfig, HookConfig as RustHookConfig } from '@ship/ui'
import type { ResolvedAgentProfile } from './types'

/**
 * Merge an agent profile into a base library for compilation.
 *
 * Creates a ModeConfig from the agent (referencing its skills, MCP servers,
 * rules, and providers), adds the agent's assets to the library's top-level
 * arrays, and sets `active_agent` so the compiler resolves that agent's view.
 *
 * Returns a new library object -- never mutates the input.
 */
export function agentToLibrary(
  agent: ResolvedAgentProfile,
  baseLibrary: ProjectLibrary,
): ProjectLibrary {
  const modeId = `agent-${agent.profile.id}`

  // Build a mode that references the agent's assets by ID/name
  const mode: ModeConfig = {
    id: modeId,
    name: agent.profile.name,
    description: agent.profile.description || undefined,
    target_agents: ['claude', 'gemini', 'codex', 'cursor'],
    mcp_servers: agent.mcpServers.map((s) => s.name),
    skills: agent.skills.map((s) => s.id),
    rules: agent.rules.map((r) => r.file_name),
  }

  // Convert web-app hooks to Rust HookConfig format
  const agentHooks: RustHookConfig[] = agent.hooks.map((h, i) => ({
    id: `${modeId}-hook-${i}`,
    trigger: h.trigger as RustHookConfig['trigger'],
    command: h.command,
    matcher: h.matcher ?? null,
    cursor_event: null,
    gemini_event: null,
  }))

  // Merge skills: base + agent (deduplicated by id)
  const baseSkills = baseLibrary.skills ?? []
  const existingSkillIds = new Set(baseSkills.map((s) => s.id))
  const newSkills = agent.skills.filter((s) => !existingSkillIds.has(s.id))
  const mergedSkills = [...baseSkills, ...newSkills]

  // Merge MCP servers: base + agent (deduplicated by name)
  const baseServers = baseLibrary.mcp_servers ?? []
  const existingServerNames = new Set(baseServers.map((s) => s.name))
  const newServers = agent.mcpServers.filter((s) => !existingServerNames.has(s.name))
  const mergedServers = [...baseServers, ...newServers]

  // Merge rules: base + agent (deduplicated by file_name)
  const baseRules = baseLibrary.rules ?? []
  const existingRuleNames = new Set(baseRules.map((r) => r.file_name))
  const newRules = agent.rules.filter((r) => !existingRuleNames.has(r.file_name))
  const mergedRules = [...baseRules, ...newRules]

  // Merge hooks: base + agent hooks
  const baseHooks = baseLibrary.hooks ?? []
  const mergedHooks = [...baseHooks, ...agentHooks]

  // Replace existing modes with the agent mode (keep others)
  const baseModes = (baseLibrary.modes ?? []).filter((m) => m.id !== modeId)
  const mergedModes = [...baseModes, mode]

  // Build the Rust-format AgentProfile for agent_profiles
  const rustProfile = {
    profile: {
      id: agent.profile.id,
      name: agent.profile.name,
      version: agent.profile.version || undefined,
      description: agent.profile.description || undefined,
      providers: agent.profile.providers,
    },
    skills: { refs: agent.skills.map((s) => s.id) },
    mcp: { servers: agent.mcpServers.map((s) => s.name) },
    permissions: {
      preset: agent.permissions?.preset || undefined,
      tools_allow: agent.permissions?.tools_allow,
      tools_deny: agent.permissions?.tools_deny,
    },
    rules: agent.rules.length > 0
      ? { inline: agent.rules.map((r) => r.content).join('\n\n') || undefined }
      : undefined,
  }

  // Add to agent_profiles (deduplicated by profile.id)
  const baseProfiles = baseLibrary.agent_profiles ?? []
  const filteredProfiles = baseProfiles.filter((p) => p.profile.id !== agent.profile.id)

  return {
    ...baseLibrary,
    modes: mergedModes,
    active_agent: modeId,
    mcp_servers: mergedServers,
    skills: mergedSkills,
    rules: mergedRules,
    hooks: mergedHooks,
    agent_profiles: [...filteredProfiles, rustProfile],
  }
}
