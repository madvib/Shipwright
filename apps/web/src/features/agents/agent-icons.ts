// Agent icon preferences — stored separately from the agent profile.
// This is UI-only state (the compiler doesn't need icons).

const STORAGE_KEY = 'ship-agent-icons'

export function getAgentIcon(agentId: string): string | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return null
    const icons = JSON.parse(raw) as Record<string, string>
    return icons[agentId] ?? null
  } catch {
    return null
  }
}

export function setAgentIcon(agentId: string, icon: string) {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    const icons = raw ? (JSON.parse(raw) as Record<string, string>) : {}
    icons[agentId] = icon
    localStorage.setItem(STORAGE_KEY, JSON.stringify(icons))
  } catch {
    // ignore
  }
}
