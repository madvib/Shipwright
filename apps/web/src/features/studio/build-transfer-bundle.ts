// Builds a TransferBundle from a ResolvedAgentProfile for push to CLI.
// Every AgentBundle field must be mapped here — omitting a field causes
// silent data loss on round-trip.

import type { TransferBundle } from '@ship/ui'
import type { ResolvedAgentProfile } from '#/features/agents/types'

export function buildTransferBundle(agent: ResolvedAgentProfile): TransferBundle {
  return {
    agent: {
      id: agent.profile.id,
      name: agent.profile.name,
      description: agent.profile.description,
      version: agent.profile.version,
      providers: agent.profile.providers,
      model: agent.model ?? null,
      env: (agent.env as Record<string, string> | null) ?? null,
      available_models: agent.availableModels ?? null,
      agent_limits: agent.agentLimits ?? null,
      hooks: (agent.hooks ?? []) as TransferBundle['agent']['hooks'],
      skill_refs: agent.skills.map((s) => s.id),
      rule_refs: agent.rules.map((r) => r.file_name ?? r.content.slice(0, 30)),
      mcp_servers: agent.mcpServers.map((s) => s.name),
      permissions: (agent.permissions ?? null) as TransferBundle['agent']['permissions'],
      provider_settings: (agent.providerSettings ?? null) as TransferBundle['agent']['provider_settings'],
      plugins: null,
      rules_inline: null,
    },
    skills: Object.fromEntries(
      agent.skills.map((s) => [s.id, { files: { 'SKILL.md': s.content } }]),
    ),
    rules: Object.fromEntries(
      agent.rules.map((r) => [r.file_name ?? `rule-${r.content.slice(0, 20)}`, r.content]),
    ),
    dependencies: {},
  }
}
