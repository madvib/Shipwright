// Agent icon resolution — reads from profile first, falls back to localStorage.
// Icons now persist in .ship/agents/*.jsonc via the `icon` field.

const STORAGE_KEY = 'ship-agent-icons'

/** Get icon for an agent. Checks profile icon first, then localStorage. */
export function getAgentIcon(agentId: string, profileIcon?: string | null): string | null {
  // Profile icon is the source of truth (persists in .jsonc)
  if (profileIcon) return profileIcon
  // Fall back to localStorage (legacy, pre-icon-field)
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return null
    const icons = JSON.parse(raw) as Record<string, string>
    return icons[agentId] ?? null
  } catch {
    return null
  }
}

/** Save icon to localStorage (legacy). Prefer setting via agent profile. */
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
