// ── Schema-driven validation for agent profiles ─────────────────────────────
// Uses agent.schema.json as the single source of truth.
// No external validation library — focused validator for our own schema.

import agentSchema from '../../../../../schemas/agent.schema.json'
import type { ResolvedAgentProfile } from './types'

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

const DEFAULT_MODE_ENUM: string[] = (() => {
  const permsProps = (agentSchema.properties.permissions as SchemaNode)?.properties as SchemaNode | undefined
  const node = permsProps?.default_mode as SchemaNode | undefined
  return node ? (getEnumFromNode(node) ?? []) : []
})()

const PLUGIN_SCOPE_ENUM = getEnumFromNode(
  (agentSchema.properties.plugins as SchemaNode).properties
    ? ((agentSchema.properties.plugins as SchemaNode).properties as SchemaNode).scope as SchemaNode
    : {},
) ?? []

// ── Validate ────────────────────────────────────────────────────────────────

export function validateAgentProfile(profile: ResolvedAgentProfile): ValidationResult {
  const errors: FieldError[] = []
  const meta = profile.profile

  // Required: name (maps to agent.name in schema)
  if (!meta.name || typeof meta.name !== 'string' || !meta.name.trim()) {
    errors.push({ path: 'agent.name', message: 'Name is required.' })
  }

  // ID pattern: ^[a-z0-9-]+$
  if (meta.id && !/^[a-z0-9-]+$/.test(meta.id)) {
    errors.push({
      path: 'agent.id',
      message: 'ID must contain only lowercase letters, digits, and hyphens.',
    })
  }

  // Providers: each must be in enum
  if (Array.isArray(meta.providers)) {
    for (const p of meta.providers) {
      if (PROVIDER_ENUM.length > 0 && !PROVIDER_ENUM.includes(p)) {
        errors.push({
          path: 'agent.providers',
          message: `Invalid provider "${p}". Must be one of: ${PROVIDER_ENUM.join(', ')}`,
        })
      }
    }
  }

  // Permission preset: must be in enum (if set and non-empty)
  const permPreset = profile.permissions?.preset
  if (
    permPreset &&
    permPreset !== 'custom' &&
    PERMISSION_PRESET_ENUM.length > 0 &&
    !PERMISSION_PRESET_ENUM.includes(permPreset)
  ) {
    errors.push({
      path: 'permissions.preset',
      message: `Invalid permission preset "${permPreset}". Must be one of: ${PERMISSION_PRESET_ENUM.join(', ')}`,
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
