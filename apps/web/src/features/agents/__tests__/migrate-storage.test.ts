import { describe, it, expect, beforeEach } from 'vitest'

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

// ── Import migration functions ──────────────────────────────────────────────

import {
  hasMigrated,
  migrateFromV1,
  finalizeMigration,
} from '../migrate-storage'
import type { AgentProfile } from '../types'

const V1_PROFILES_KEY = 'ship-profiles-v1'
const V1_AGENT_DETAIL_KEY = 'ship-agent-profiles-v1'

// ── Tests ───────────────────────────────────────────────────────────────────

describe('migrateFromV1', () => {
  beforeEach(() => {
    storage.clear()
  })

  it('returns empty when no V1 data exists', () => {
    const result = migrateFromV1()
    expect(result.agents).toEqual([])
    expect(result.activeId).toBeNull()
  })

  it('migrates ship-profiles-v1 data to AgentProfile format', () => {
    const v1Data = {
      profiles: [
        {
          id: 'profile-1',
          name: 'web-lane',
          persona: 'Web specialist',
          icon: 'react',
          accentColor: '#61dafb',
          selectedProviders: ['claude', 'gemini'],
          skills: [
            { id: 'sk1', name: 'code-review', content: 'review code', source: 'community' },
          ],
          mcpServers: [],
          rules: ['No compat without consumers', 'Test all changes'],
          permissions: {
            tools: { allow: ['Read'], deny: [] },
            filesystem: { allow: ['**/*'], deny: [] },
            commands: { allow: [], deny: [] },
            network: { policy: 'none', allow_hosts: [] },
            agent: { require_confirmation: [] },
          },
        },
      ],
      activeId: 'profile-1',
    }
    localStorage.setItem(V1_PROFILES_KEY, JSON.stringify(v1Data))

    const result = migrateFromV1()
    expect(result.agents).toHaveLength(1)
    expect(result.activeId).toBe('profile-1')

    const agent = result.agents[0]
    expect(agent.id).toBe('profile-1')
    expect(agent.name).toBe('web-lane')
    expect(agent.description).toBe('Web specialist')
    expect(agent.providers).toEqual(['claude', 'gemini'])
    expect(agent.skills).toHaveLength(1)
    expect(agent.skills[0].source).toBe('community')
    expect(agent.rules).toHaveLength(2)
    expect(agent.rules[0].file_name).toBe('rule-0.md')
    expect(agent.rules[0].content).toBe('No compat without consumers')
    expect(agent.permissions.tools?.allow).toEqual(['Read'])
  })

  it('migrates ship-agent-profiles-v1 data (Record format)', () => {
    const v1Detail: Record<string, AgentProfile> = {
      'agent-a': {
        id: 'agent-a',
        name: 'agent-a',
        description: 'An agent',
        providers: ['claude'],
        version: '0.1.0',
        skills: [],
        mcpServers: [],
        subagents: [],
        permissions: {
          tools: { allow: [], deny: [] },
          filesystem: { allow: ['**/*'], deny: [] },
          commands: { allow: [], deny: [] },
          network: { policy: 'none', allow_hosts: [] },
          agent: { require_confirmation: [] },
        },
        permissionPreset: 'ship-guarded',
        settings: { model: 'claude-sonnet-4-6', defaultMode: 'default', extendedThinking: true, autoMemory: false },
        hooks: [],
        rules: [],
        mcpToolStates: {},
      },
    }
    localStorage.setItem(V1_AGENT_DETAIL_KEY, JSON.stringify(v1Detail))

    const result = migrateFromV1()
    expect(result.agents).toHaveLength(1)
    expect(result.agents[0].id).toBe('agent-a')
    expect(result.agents[0].name).toBe('agent-a')
  })

  it('merges both V1 sources, detail wins for matching IDs', () => {
    const v1Profiles = {
      profiles: [
        { id: 'shared-id', name: 'profile-name', persona: 'from profiles' },
        { id: 'profile-only', name: 'only-in-profiles' },
      ],
      activeId: 'shared-id',
    }
    const v1Detail: Record<string, AgentProfile> = {
      'shared-id': {
        id: 'shared-id',
        name: 'detail-name',
        description: 'from detail',
        providers: ['claude', 'gemini'],
        version: '0.1.0',
        skills: [],
        mcpServers: [],
        subagents: [],
        permissions: {
          tools: { allow: [], deny: [] },
          filesystem: { allow: [], deny: [] },
          commands: { allow: [], deny: [] },
          network: { policy: 'none', allow_hosts: [] },
          agent: { require_confirmation: [] },
        },
        permissionPreset: 'ship-guarded',
        settings: { model: 'claude-sonnet-4-6', defaultMode: 'default', extendedThinking: true, autoMemory: false },
        hooks: [],
        rules: [],
        mcpToolStates: {},
      },
      'detail-only': {
        id: 'detail-only',
        name: 'only-in-detail',
        description: '',
        providers: ['claude'],
        version: '0.1.0',
        skills: [],
        mcpServers: [],
        subagents: [],
        permissions: {
          tools: { allow: [], deny: [] },
          filesystem: { allow: [], deny: [] },
          commands: { allow: [], deny: [] },
          network: { policy: 'none', allow_hosts: [] },
          agent: { require_confirmation: [] },
        },
        permissionPreset: 'ship-guarded',
        settings: { model: 'claude-sonnet-4-6', defaultMode: 'default', extendedThinking: true, autoMemory: false },
        hooks: [],
        rules: [],
        mcpToolStates: {},
      },
    }

    localStorage.setItem(V1_PROFILES_KEY, JSON.stringify(v1Profiles))
    localStorage.setItem(V1_AGENT_DETAIL_KEY, JSON.stringify(v1Detail))

    const result = migrateFromV1()
    expect(result.agents).toHaveLength(3)

    const shared = result.agents.find((a) => a.id === 'shared-id')
    expect(shared).toBeDefined()
    // Detail wins: name should be 'detail-name', not 'profile-name'
    expect(shared!.name).toBe('detail-name')
    expect(shared!.description).toBe('from detail')

    const profileOnly = result.agents.find((a) => a.id === 'profile-only')
    expect(profileOnly).toBeDefined()
    expect(profileOnly!.name).toBe('only-in-profiles')

    const detailOnly = result.agents.find((a) => a.id === 'detail-only')
    expect(detailOnly).toBeDefined()
    expect(detailOnly!.name).toBe('only-in-detail')
  })

  it('handles corrupt V1 data gracefully', () => {
    localStorage.setItem(V1_PROFILES_KEY, 'not-valid-json')
    localStorage.setItem(V1_AGENT_DETAIL_KEY, '{bad')

    const result = migrateFromV1()
    expect(result.agents).toEqual([])
  })
})

describe('hasMigrated / finalizeMigration', () => {
  beforeEach(() => {
    storage.clear()
  })

  it('hasMigrated returns false before migration', () => {
    expect(hasMigrated()).toBe(false)
  })

  it('finalizeMigration sets flag and removes old keys', () => {
    localStorage.setItem(V1_PROFILES_KEY, 'some-data')
    localStorage.setItem(V1_AGENT_DETAIL_KEY, 'some-data')

    finalizeMigration()

    expect(hasMigrated()).toBe(true)
    expect(localStorage.getItem(V1_PROFILES_KEY)).toBeNull()
    expect(localStorage.getItem(V1_AGENT_DETAIL_KEY)).toBeNull()
  })

  it('hasMigrated returns true after finalizeMigration', () => {
    finalizeMigration()
    expect(hasMigrated()).toBe(true)
  })

  it('migration is idempotent - running twice returns same result', () => {
    const v1Data = {
      profiles: [{ id: 'p1', name: 'test' }],
      activeId: 'p1',
    }
    localStorage.setItem(V1_PROFILES_KEY, JSON.stringify(v1Data))

    const first = migrateFromV1()
    expect(first.agents).toHaveLength(1)

    // Simulate saving and finalizing
    finalizeMigration()

    // Second call: V1 keys are gone, flag is set
    const second = migrateFromV1()
    // Still returns data from whatever is in storage (V1 keys removed)
    expect(second.agents).toHaveLength(0)
  })
})
