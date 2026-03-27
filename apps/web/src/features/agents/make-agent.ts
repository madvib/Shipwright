import type { ResolvedAgentProfile } from './types'

function slugify(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '') || `agent-${Date.now()}`
}

/** Create a ResolvedAgentProfile with sensible defaults. */
export function makeAgent(partial?: Partial<ResolvedAgentProfile>): ResolvedAgentProfile {
  const pp = partial?.profile
  const name = pp?.name ?? 'New Agent'
  return {
    profile: {
      id: pp?.id || slugify(name),
      name,
      description: pp?.description ?? '',
      providers: pp?.providers ?? ['claude'],
      version: pp?.version ?? '0.1.0',
    },
    skills: partial?.skills ?? [],
    mcpServers: partial?.mcpServers ?? [],
    permissions: partial?.permissions ?? { preset: 'ship-standard' },
    hooks: partial?.hooks ?? [],
    rules: partial?.rules ?? [],
    model: partial?.model ?? null,
    env: partial?.env ?? null,
    availableModels: partial?.availableModels ?? null,
    agentLimits: partial?.agentLimits ?? null,
    providerSettings: partial?.providerSettings ?? {},
    toolPermissions: partial?.toolPermissions ?? {},
    source: partial?.source ?? 'project',
  }
}
