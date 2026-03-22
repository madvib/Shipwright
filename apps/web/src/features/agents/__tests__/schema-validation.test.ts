import { describe, it, expect } from 'vitest'
import {
  validateAgentProfile,
  getPermissionPresets,
  getProviderIds,
  getDefaultModes,
  getPluginScopes,
} from '../schema-validation'
import type { AgentProfile } from '../types'
import { DEFAULT_SETTINGS } from '../types'
import { DEFAULT_PERMISSIONS } from '@ship/ui'

// ── Fixtures ────────────────────────────────────────────────────────────────

function makeValidProfile(overrides?: Partial<AgentProfile>): AgentProfile {
  return {
    id: 'test-agent',
    name: 'Test Agent',
    description: 'A test agent',
    providers: ['claude'],
    version: '0.1.0',
    skills: [],
    mcpServers: [],
    subagents: [],
    permissions: DEFAULT_PERMISSIONS,
    permissionPreset: 'ship-standard',
    settings: { ...DEFAULT_SETTINGS },
    hooks: [],
    rules: [],
    mcpToolStates: {},
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
    const result = validateAgentProfile(makeValidProfile({ name: '' }))
    expect(result.valid).toBe(false)
    expect(result.errors.some((e) => e.path === 'agent.name')).toBe(true)
  })

  it('fails when name is whitespace only', () => {
    const result = validateAgentProfile(makeValidProfile({ name: '   ' }))
    expect(result.valid).toBe(false)
    expect(result.errors.some((e) => e.path === 'agent.name')).toBe(true)
  })

  it('fails for invalid provider', () => {
    const result = validateAgentProfile(
      makeValidProfile({ providers: ['claude', 'invalid-provider'] }),
    )
    expect(result.valid).toBe(false)
    const providerError = result.errors.find((e) => e.path === 'agent.providers')
    expect(providerError).toBeDefined()
    expect(providerError!.message).toContain('invalid-provider')
  })

  it('passes for all valid providers', () => {
    const result = validateAgentProfile(
      makeValidProfile({ providers: ['claude', 'cursor', 'codex', 'gemini'] }),
    )
    expect(result.valid).toBe(true)
  })

  it('fails for invalid permission preset', () => {
    const result = validateAgentProfile(
      makeValidProfile({ permissionPreset: 'nonexistent-preset' }),
    )
    expect(result.valid).toBe(false)
    const presetError = result.errors.find((e) => e.path === 'permissions.preset')
    expect(presetError).toBeDefined()
    expect(presetError!.message).toContain('nonexistent-preset')
  })

  it('passes for "custom" permission preset (special case)', () => {
    const result = validateAgentProfile(
      makeValidProfile({ permissionPreset: 'custom' }),
    )
    expect(result.valid).toBe(true)
  })

  it('passes for valid permission presets', () => {
    for (const preset of ['ship-readonly', 'ship-standard', 'ship-autonomous', 'ship-elevated']) {
      const result = validateAgentProfile(makeValidProfile({ permissionPreset: preset }))
      expect(result.valid).toBe(true)
    }
  })

  it('fails for invalid id pattern', () => {
    const result = validateAgentProfile(
      makeValidProfile({ id: 'UPPERCASE_ID' }),
    )
    expect(result.valid).toBe(false)
    expect(result.errors.some((e) => e.path === 'agent.id')).toBe(true)
  })

  it('passes for valid id pattern', () => {
    const result = validateAgentProfile(
      makeValidProfile({ id: 'valid-agent-123' }),
    )
    expect(result.valid).toBe(true)
  })

  it('fails for invalid default mode', () => {
    const result = validateAgentProfile(
      makeValidProfile({
        settings: { ...DEFAULT_SETTINGS, defaultMode: 'invalid-mode' },
      }),
    )
    expect(result.valid).toBe(false)
    expect(result.errors.some((e) => e.path === 'permissions.default_mode')).toBe(true)
  })

  it('collects multiple errors at once', () => {
    const result = validateAgentProfile(
      makeValidProfile({
        name: '',
        providers: ['bad-provider'],
        permissionPreset: 'bad-preset',
      }),
    )
    expect(result.valid).toBe(false)
    expect(result.errors.length).toBeGreaterThanOrEqual(3)
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
    expect(providers).toHaveLength(4)
  })

  it('getPermissionPresets returns schema-defined presets', () => {
    const presets = getPermissionPresets()
    expect(presets).toContain('ship-readonly')
    expect(presets).toContain('ship-standard')
    expect(presets).toContain('ship-autonomous')
    expect(presets).toContain('ship-elevated')
    expect(presets).toHaveLength(4)
  })

  it('getDefaultModes returns schema-defined modes', () => {
    const modes = getDefaultModes()
    expect(modes).toContain('default')
    expect(modes).toContain('plan')
    expect(modes).toContain('bypassPermissions')
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
