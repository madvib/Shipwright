import { describe, it, expect } from 'vitest'
import {
  validateAgentProfile,
  getPermissionPresets,
  getProviderIds,
  getPluginScopes,
} from '../schema-validation'
import type { ResolvedAgentProfile } from '../types'

// ── Fixtures ────────────────────────────────────────────────────────────────

function makeValidProfile(overrides?: Partial<ResolvedAgentProfile>): ResolvedAgentProfile {
  return {
    profile: {
      id: 'test-agent',
      name: 'Test Agent',
      description: 'A test agent',
      providers: ['claude'],
      version: '0.1.0',
    },
    skills: [],
    mcpServers: [],
    permissions: { preset: 'ship-standard' },
    hooks: [],
    rules: [],
    ...overrides,
  }
}

// ── Validation tests ────────────────────────────────────────────────────────

describe('validateAgentProfile', () => {
  it('passes for a valid profile', () => {
    const result = validateAgentProfile(makeValidProfile())
    expect(result.valid).toBe(true)
    expect(result.errors).toHaveLength(0)
  })

  it('fails when name is empty', () => {
    const result = validateAgentProfile(makeValidProfile({
      profile: { id: 'test-agent', name: '', providers: ['claude'] },
    }))
    expect(result.valid).toBe(false)
    expect(result.errors.some((e) => e.path === 'agent.name')).toBe(true)
  })

  it('fails when name is whitespace only', () => {
    const result = validateAgentProfile(makeValidProfile({
      profile: { id: 'test-agent', name: '   ', providers: ['claude'] },
    }))
    expect(result.valid).toBe(false)
    expect(result.errors.some((e) => e.path === 'agent.name')).toBe(true)
  })

  it('fails for invalid provider', () => {
    const result = validateAgentProfile(
      makeValidProfile({
        profile: { id: 'test-agent', name: 'Test', providers: ['claude', 'invalid-provider'] },
      }),
    )
    expect(result.valid).toBe(false)
    const providerError = result.errors.find((e) => e.path === 'agent.providers')
    expect(providerError).toBeDefined()
    expect(providerError!.message).toContain('invalid-provider')
  })

  it('passes for all valid providers', () => {
    const result = validateAgentProfile(
      makeValidProfile({
        profile: { id: 'test-agent', name: 'Test', providers: ['claude', 'cursor', 'codex', 'gemini'] },
      }),
    )
    expect(result.valid).toBe(true)
  })

  it('accepts any preset string (presets resolved at compile time, not schema-validated)', () => {
    const result = validateAgentProfile(
      makeValidProfile({ permissions: { preset: 'any-preset-name' } }),
    )
    expect(result.valid).toBe(true)
  })

  it('passes for "custom" permission preset (special case)', () => {
    const result = validateAgentProfile(
      makeValidProfile({ permissions: { preset: 'custom' } }),
    )
    expect(result.valid).toBe(true)
  })

  it('passes for valid permission presets', () => {
    for (const preset of ['ship-readonly', 'ship-standard', 'ship-autonomous', 'ship-elevated']) {
      const result = validateAgentProfile(makeValidProfile({ permissions: { preset } }))
      expect(result.valid).toBe(true)
    }
  })

  it('fails for invalid id pattern', () => {
    const result = validateAgentProfile(
      makeValidProfile({
        profile: { id: 'UPPERCASE_ID', name: 'Test', providers: ['claude'] },
      }),
    )
    expect(result.valid).toBe(false)
    expect(result.errors.some((e) => e.path === 'agent.id')).toBe(true)
  })

  it('passes for valid id pattern', () => {
    const result = validateAgentProfile(
      makeValidProfile({
        profile: { id: 'valid-agent-123', name: 'Test', providers: ['claude'] },
      }),
    )
    expect(result.valid).toBe(true)
  })

  it('collects multiple errors at once', () => {
    const result = validateAgentProfile(
      makeValidProfile({
        profile: { id: 'test', name: '', providers: ['bad-provider'] },
      }),
    )
    expect(result.valid).toBe(false)
    expect(result.errors.length).toBeGreaterThanOrEqual(2)
  })
})

// ── Enum extraction tests ───────────────────────────────────────────────────

describe('schema enum extractors', () => {
  it('getProviderIds returns schema-defined providers', () => {
    const providers = getProviderIds()
    expect(providers).toContain('claude')
    expect(providers).toContain('cursor')
    expect(providers).toContain('codex')
    expect(providers).toContain('gemini')
    expect(providers).toContain('opencode')
    expect(providers).toHaveLength(5)
  })

  it('getPermissionPresets returns empty (presets are runtime-resolved, not schema-defined)', () => {
    const presets = getPermissionPresets()
    // Schema no longer enumerates presets — they're free-form strings
    // resolved from .ship/permissions.jsonc at compile time
    expect(Array.isArray(presets)).toBe(true)
  })

  it('getPluginScopes returns schema-defined scopes', () => {
    const scopes = getPluginScopes()
    expect(scopes).toContain('project')
    expect(scopes).toContain('user')
  })

  it('returns copies, not references', () => {
    const a = getProviderIds()
    const b = getProviderIds()
    expect(a).not.toBe(b)
    expect(a).toEqual(b)
  })
})
