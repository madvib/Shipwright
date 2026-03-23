// Agent API client -- localStorage only.
// Server-side persistence (profiles table) has been removed.
// These functions are kept as stubs to avoid breaking callers.
// Agent data lives exclusively in localStorage via useAgentStore.

import type { ResolvedAgentProfile } from './types'

export async function fetchAgents(): Promise<ResolvedAgentProfile[]> {
  // No server storage -- return empty to let localStorage be authoritative
  return []
}

export async function createAgentApi(_agent: ResolvedAgentProfile): Promise<void> {
  // No-op: server profiles table removed
}

export async function updateAgentApi(
  _id: string,
  _agent: ResolvedAgentProfile,
): Promise<void> {
  // No-op: server profiles table removed
}

export async function deleteAgentApi(_id: string): Promise<void> {
  // No-op: server profiles table removed
}
