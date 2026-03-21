// ── Agent API client ────────────────────────────────────────────────────────
// Server-side persistence for agent profiles. All calls require authentication.
// The `content` field stores the serialized AgentProfile JSON.

import { fetchApi } from '#/lib/api-errors'
import type { AgentProfile } from './types'

interface ServerProfile {
  id: string
  orgId: string
  userId: string
  name: string
  content: string
  provider: string | null
  createdAt: number
  updatedAt: number
}

function parseServerProfile(sp: ServerProfile): AgentProfile {
  try {
    return JSON.parse(sp.content) as AgentProfile
  } catch {
    throw new Error(`Failed to parse agent profile content for id=${sp.id}`)
  }
}

function serializeAgent(agent: AgentProfile): string {
  return JSON.stringify(agent)
}

export async function fetchAgents(): Promise<AgentProfile[]> {
  const { profiles } = await fetchApi<{ profiles: ServerProfile[] }>(
    '/api/profiles',
    { credentials: 'include' },
  )
  return profiles.map(parseServerProfile)
}

export async function createAgentApi(agent: AgentProfile): Promise<void> {
  await fetchApi<{ profile: ServerProfile }>('/api/profiles', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({
      name: agent.name,
      content: serializeAgent(agent),
      provider: agent.providers[0] ?? null,
    }),
  })
}

export async function updateAgentApi(
  id: string,
  agent: AgentProfile,
): Promise<void> {
  await fetchApi<{ profile: ServerProfile }>(`/api/profiles/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({
      name: agent.name,
      content: serializeAgent(agent),
      provider: agent.providers[0] ?? null,
    }),
  })
}

export async function deleteAgentApi(id: string): Promise<void> {
  await fetchApi<{ ok: boolean }>(`/api/profiles/${id}`, {
    method: 'DELETE',
    credentials: 'include',
  })
}
