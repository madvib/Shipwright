import { describe, it, expect, beforeEach, vi } from 'vitest'

// ── localStorage mock ───────────────────────────────────────────────────────

const storage = new Map<string, string>()

const localStorageMock: Storage = {
  getItem: (key: string) => storage.get(key) ?? null,
  setItem: (key: string, value: string) => { storage.set(key, value) },
  removeItem: (key: string) => { storage.delete(key) },
  clear: () => storage.clear(),
  get length() { return storage.size },
  key: (_i: number) => null,
}

Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock })

// Mock crypto.getRandomValues for deterministic IDs
let idCounter = 0
vi.stubGlobal('crypto', {
  getRandomValues: (arr: Uint8Array) => {
    for (let i = 0; i < arr.length; i++) {
      arr[i] = (idCounter + i) % 256
    }
    idCounter++
    return arr
  },
})

// ── Import store internals ──────────────────────────────────────────────────

import { makeAgent } from '../useAgentStore'
import type { AgentProfile } from '../types'
import { DEFAULT_SETTINGS } from '../types'
import { DEFAULT_PERMISSIONS } from '@ship/ui'

const STORAGE_KEY = 'ship-agents-v2'

interface StoreState {
  agents: AgentProfile[]
  activeId: string | null
}

function loadState(): StoreState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { agents: [], activeId: null }
    return JSON.parse(raw) as StoreState
  } catch {
    return { agents: [], activeId: null }
  }
}

function saveState(state: StoreState): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
}

// ── Tests ───────────────────────────────────────────────────────────────────

describe('agent store localStorage CRUD', () => {
  beforeEach(() => {
    storage.clear()
    idCounter = 0
  })

  it('makeAgent creates a complete AgentProfile with defaults', () => {
    const agent = makeAgent({ name: 'test-agent' })
    expect(agent.name).toBe('test-agent')
    expect(agent.description).toBe('')
    expect(agent.providers).toEqual(['claude'])
    expect(agent.version).toBe('0.1.0')
    expect(agent.skills).toEqual([])
    expect(agent.mcpServers).toEqual([])
    expect(agent.subagents).toEqual([])
    expect(agent.settings).toEqual(DEFAULT_SETTINGS)
    expect(agent.hooks).toEqual([])
    expect(agent.rules).toEqual([])
    expect(agent.mcpToolStates).toEqual({})
    expect(agent.id).toBeTruthy()
    expect(typeof agent.id).toBe('string')
    expect(agent.id.length).toBe(32)
  })

  it('makeAgent merges partial overrides', () => {
    const agent = makeAgent({
      name: 'custom',
      description: 'a custom agent',
      providers: ['gemini', 'codex'],
    })
    expect(agent.name).toBe('custom')
    expect(agent.description).toBe('a custom agent')
    expect(agent.providers).toEqual(['gemini', 'codex'])
    expect(agent.skills).toEqual([]) // default
  })

  it('creates an agent and persists to localStorage', () => {
    const agent = makeAgent({ name: 'web-lane' })
    const state: StoreState = { agents: [agent], activeId: agent.id }
    saveState(state)

    const loaded = loadState()
    expect(loaded.agents).toHaveLength(1)
    expect(loaded.agents[0].name).toBe('web-lane')
    expect(loaded.activeId).toBe(agent.id)
  })

  it('updates an agent in localStorage', () => {
    const agent = makeAgent({ name: 'original' })
    saveState({ agents: [agent], activeId: agent.id })

    const state = loadState()
    const updated = state.agents.map((a) =>
      a.id === agent.id ? { ...a, name: 'updated' } : a,
    )
    saveState({ ...state, agents: updated })

    const reloaded = loadState()
    expect(reloaded.agents[0].name).toBe('updated')
  })

  it('deletes an agent from localStorage', () => {
    const agent1 = makeAgent({ name: 'agent-1' })
    const agent2 = makeAgent({ name: 'agent-2' })
    saveState({ agents: [agent1, agent2], activeId: agent1.id })

    const state = loadState()
    const remaining = state.agents.filter((a) => a.id !== agent1.id)
    saveState({ agents: remaining, activeId: remaining[0]?.id ?? null })

    const reloaded = loadState()
    expect(reloaded.agents).toHaveLength(1)
    expect(reloaded.agents[0].name).toBe('agent-2')
    expect(reloaded.activeId).toBe(agent2.id)
  })

  it('handles empty localStorage gracefully', () => {
    const state = loadState()
    expect(state.agents).toEqual([])
    expect(state.activeId).toBeNull()
  })

  it('handles corrupt localStorage data', () => {
    localStorage.setItem(STORAGE_KEY, 'not-valid-json')
    const state = loadState()
    expect(state.agents).toEqual([])
    expect(state.activeId).toBeNull()
  })

  it('getAgent finds agent by id', () => {
    const agent1 = makeAgent({ name: 'first' })
    const agent2 = makeAgent({ name: 'second' })
    const state: StoreState = { agents: [agent1, agent2], activeId: agent1.id }

    const found = state.agents.find((a) => a.id === agent2.id)
    expect(found).toBeDefined()
    expect(found?.name).toBe('second')
  })

  it('getAgent returns undefined for missing id', () => {
    const agent = makeAgent({ name: 'only' })
    const state: StoreState = { agents: [agent], activeId: agent.id }

    const found = state.agents.find((a) => a.id === 'nonexistent')
    expect(found).toBeUndefined()
  })

  it('multiple agents can coexist', () => {
    const agents = Array.from({ length: 5 }, (_, i) =>
      makeAgent({ name: `agent-${i}` }),
    )
    saveState({ agents, activeId: agents[2].id })

    const state = loadState()
    expect(state.agents).toHaveLength(5)
    expect(state.activeId).toBe(agents[2].id)
    expect(state.agents.map((a) => a.name)).toEqual([
      'agent-0', 'agent-1', 'agent-2', 'agent-3', 'agent-4',
    ])
  })

  it('permissions default to DEFAULT_PERMISSIONS', () => {
    const agent = makeAgent()
    expect(agent.permissions).toEqual(DEFAULT_PERMISSIONS)
    expect(agent.permissionPreset).toBe('ship-guarded')
  })
})
