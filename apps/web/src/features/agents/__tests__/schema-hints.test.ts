import { describe, it, expect } from 'vitest'
import {
  getFieldDescription,
  getFieldEnum,
  getFieldDefault,
  getFieldType,
  getFieldPattern,
  isFieldRequired,
  getFieldProperties,
} from '../schema-hints'

// ── Description tests ───────────────────────────────────────────────────────

describe('getFieldDescription', () => {
  it('returns description for agent.name', () => {
    const desc = getFieldDescription('agent.name')
    expect(desc).toBe('Human-readable agent name.')
  })

  it('returns description for permissions.preset', () => {
    const desc = getFieldDescription('permissions.preset')
    expect(desc).toContain('permission')
  })

  it('returns description for agent.providers', () => {
    const desc = getFieldDescription('agent.providers')
    expect(desc.length).toBeGreaterThan(0)
  })

  it('returns empty string for nonexistent path', () => {
    expect(getFieldDescription('nonexistent.path')).toBe('')
  })
})

// ── Enum tests ──────────────────────────────────────────────────────────────

describe('getFieldEnum', () => {
  it('returns empty enum for permissions.preset (freeform string)', () => {
    // preset is a freeform string referencing permissions.jsonc keys — no enum
    const values = getFieldEnum('permissions.preset')
    expect(values).toEqual([])
  })

  it('returns enum from array items for agent.providers', () => {
    const values = getFieldEnum('agent.providers')
    expect(values).toContain('claude')
    expect(values).toContain('cursor')
    expect(values).toContain('codex')
    expect(values).toContain('gemini')
    expect(values).toContain('opencode')
  })

  it('returns enum for permissions.default_mode', () => {
    const values = getFieldEnum('permissions.default_mode')
    expect(values).toContain('default')
    expect(values).toContain('plan')
  })

  it('returns enum for plugins.scope', () => {
    const values = getFieldEnum('plugins.scope')
    expect(values).toContain('project')
    expect(values).toContain('user')
  })

  it('returns empty array for field without enum', () => {
    expect(getFieldEnum('agent.name')).toEqual([])
  })

  it('returns empty array for nonexistent path', () => {
    expect(getFieldEnum('nonexistent.path')).toEqual([])
  })
})

// ── Default tests ───────────────────────────────────────────────────────────

describe('getFieldDefault', () => {
  it('returns default for agent.version', () => {
    expect(getFieldDefault('agent.version')).toBe('0.1.0')
  })

  it('returns default for plugins.scope', () => {
    expect(getFieldDefault('plugins.scope')).toBe('project')
  })

  it('returns undefined when no default', () => {
    expect(getFieldDefault('agent.name')).toBeUndefined()
  })

  it('returns undefined for nonexistent path', () => {
    expect(getFieldDefault('nonexistent')).toBeUndefined()
  })
})

// ── Type tests ──────────────────────────────────────────────────────────────

describe('getFieldType', () => {
  it('returns "string" for agent.name', () => {
    expect(getFieldType('agent.name')).toBe('string')
  })

  it('returns "array" for agent.providers', () => {
    expect(getFieldType('agent.providers')).toBe('array')
  })

  it('returns "object" for permissions', () => {
    expect(getFieldType('permissions')).toBe('object')
  })

  it('returns undefined for nonexistent path', () => {
    expect(getFieldType('nonexistent')).toBeUndefined()
  })
})

// ── Pattern tests ───────────────────────────────────────────────────────────

describe('getFieldPattern', () => {
  it('returns pattern for agent.id', () => {
    const pattern = getFieldPattern('agent.id')
    expect(pattern).toBe('^[a-z0-9-]+$')
  })

  it('returns undefined for field without pattern', () => {
    expect(getFieldPattern('agent.name')).toBeUndefined()
  })
})

// ── Required tests ──────────────────────────────────────────────────────────

describe('isFieldRequired', () => {
  it('agent is required at top level', () => {
    expect(isFieldRequired('agent')).toBe(true)
  })

  it('agent.name is required', () => {
    expect(isFieldRequired('agent.name')).toBe(true)
  })

  it('agent.description is not required', () => {
    expect(isFieldRequired('agent.description')).toBe(false)
  })
})

// ── Properties tests ────────────────────────────────────────────────────────

describe('getFieldProperties', () => {
  it('returns agent properties', () => {
    const props = getFieldProperties('agent')
    expect(props).toContain('name')
    expect(props).toContain('id')
    expect(props).toContain('providers')
    expect(props).toContain('description')
  })

  it('returns permissions properties', () => {
    const props = getFieldProperties('permissions')
    expect(props).toContain('preset')
    expect(props).toContain('tools_allow')
    expect(props).toContain('tools_deny')
    expect(props).toContain('default_mode')
  })

  it('returns empty array for nonexistent path', () => {
    expect(getFieldProperties('nonexistent')).toEqual([])
  })
})
