// ── Schema-driven validation for agent profiles ─────────────────────────────
// Uses agent.schema.json as the single source of truth.
// No external validation library — focused validator for our own schema.

import agentSchema from '../../../../../schemas/agent.schema.json'
import type { AgentProfile } from './types'

export interface FieldError {
  path: string
  message: string
}

export interface ValidationResult {
  valid: boolean
  errors: FieldError[]
}

// ── Schema traversal helpers ────────────────────────────────────────────────

type SchemaNode = Record<string, unknown>

function resolveSchemaNode(path: string): SchemaNode | null {
  const segments = path.split('.')
  let node: SchemaNode = agentSchema.properties as SchemaNode

  for (const seg of segments) {
    if (!node[seg]) return null
    const child = node[seg] as SchemaNode
    if (child.properties) {
      node = child.properties as SchemaNode
    } else {
      return child
    }
  }
  return null
}

function getEnumFromNode(node: SchemaNode): string[] | null {
  // Direct enum
  if (Array.isArray(node.enum)) return node.enum as string[]

  // Array with items that have enum
  if (node.type === 'array' && node.items) {
    const items = node.items as SchemaNode
    if (Array.isArray(items.enum)) return items.enum as string[]
  }

  return null
}

// ── Profile-to-schema mapping ───────────────────────────────────────────────
// AgentProfile fields map to schema paths. The schema uses nested objects
// (agent.name, agent.providers, permissions.preset) while AgentProfile
// flattens some of them.

const PROVIDER_ENUM = getEnumFromNode(
  (agentSchema.properties.agent as SchemaNode).properties
    ? ((agentSchema.properties.agent as SchemaNode).properties as SchemaNode).providers as SchemaNode
    : {},
) ?? []

const PERMISSION_PRESET_ENUM = getEnumFromNode(
  (agentSchema.properties.permissions as SchemaNode).properties
    ? ((agentSchema.properties.permissions as SchemaNode).properties as SchemaNode).preset as SchemaNode
    : {},
) ?? []

const DEFAULT_MODE_ENUM = getEnumFromNode(
  (agentSchema.properties.permissions as SchemaNode).properties
    ? ((agentSchema.properties.permissions as SchemaNode).properties as SchemaNode).default_mode as SchemaNode
    : {},
) ?? []

const PLUGIN_SCOPE_ENUM = getEnumFromNode(
  (agentSchema.properties.plugins as SchemaNode).properties
    ? ((agentSchema.properties.plugins as SchemaNode).properties as SchemaNode).scope as SchemaNode
    : {},
) ?? []

// ── Validate ────────────────────────────────────────────────────────────────

export function validateAgentProfile(profile: AgentProfile): ValidationResult {
  const errors: FieldError[] = []

  // Required: name (maps to agent.name in schema)
  if (!profile.name || typeof profile.name !== 'string' || !profile.name.trim()) {
    errors.push({ path: 'agent.name', message: 'Name is required.' })
  }

  // ID pattern: ^[a-z0-9-]+$
  if (profile.id && !/^[a-z0-9-]+$/.test(profile.id)) {
    errors.push({
      path: 'agent.id',
      message: 'ID must contain only lowercase letters, digits, and hyphens.',
    })
  }

  // Providers: each must be in enum
  if (Array.isArray(profile.providers)) {
    for (const p of profile.providers) {
      if (PROVIDER_ENUM.length > 0 && !PROVIDER_ENUM.includes(p)) {
        errors.push({
          path: 'agent.providers',
          message: `Invalid provider "${p}". Must be one of: ${PROVIDER_ENUM.join(', ')}`,
        })
      }
    }
  }

  // Permission preset: must be in enum (if set and non-empty)
  if (
    profile.permissionPreset &&
    profile.permissionPreset !== 'custom' &&
    PERMISSION_PRESET_ENUM.length > 0 &&
    !PERMISSION_PRESET_ENUM.includes(profile.permissionPreset)
  ) {
    errors.push({
      path: 'permissions.preset',
      message: `Invalid permission preset "${profile.permissionPreset}". Must be one of: ${PERMISSION_PRESET_ENUM.join(', ')}`,
    })
  }

  // Settings.defaultMode: must be in enum (if set)
  if (
    profile.settings?.defaultMode &&
    DEFAULT_MODE_ENUM.length > 0 &&
    !DEFAULT_MODE_ENUM.includes(profile.settings.defaultMode)
  ) {
    errors.push({
      path: 'permissions.default_mode',
      message: `Invalid default mode "${profile.settings.defaultMode}". Must be one of: ${DEFAULT_MODE_ENUM.join(', ')}`,
    })
  }

  return { valid: errors.length === 0, errors }
}

// ── Exported enums for autocomplete ─────────────────────────────────────────

export function getPermissionPresets(): string[] {
  return [...PERMISSION_PRESET_ENUM]
}

export function getProviderIds(): string[] {
  return [...PROVIDER_ENUM]
}

export function getDefaultModes(): string[] {
  return [...DEFAULT_MODE_ENUM]
}

export function getPluginScopes(): string[] {
  return [...PLUGIN_SCOPE_ENUM]
}

// Re-export for convenience
export { resolveSchemaNode, getEnumFromNode }
