// ── Schema-driven hints for agent editor UI ─────────────────────────────────
// Parses agent.schema.json to extract field descriptions, defaults, and enums
// so the UI never hardcodes values that the schema already defines.

import agentSchema from '../../../../../schemas/agent.schema.json'

type SchemaNode = Record<string, unknown>

// ── Path resolution ─────────────────────────────────────────────────────────
// Paths use dot notation matching the schema structure:
//   "agent.name", "agent.providers", "permissions.preset", "plugins.scope"

function walkToNode(path: string): SchemaNode | null {
  const segments = path.split('.')
  let current: SchemaNode = agentSchema as SchemaNode

  for (const seg of segments) {
    // Navigate into properties
    const props = current.properties as SchemaNode | undefined
    if (!props || !props[seg]) return null
    current = props[seg] as SchemaNode
  }

  return current
}

// ── Public API ──────────────────────────────────────────────────────────────

/**
 * Get the human-readable description for a schema field.
 * Returns empty string if the path is not found.
 */
export function getFieldDescription(path: string): string {
  const node = walkToNode(path)
  if (!node) return ''
  return (node.description as string) ?? ''
}

/**
 * Get enum values for a schema field.
 * Handles both direct enums and array items with enums.
 * Returns empty array if the path has no enum.
 */
export function getFieldEnum(path: string): string[] {
  const node = walkToNode(path)
  if (!node) return []

  // Direct enum on the node
  if (Array.isArray(node.enum)) return node.enum as string[]

  // Array type with items.enum
  if (node.type === 'array' && node.items) {
    const items = node.items as SchemaNode
    if (Array.isArray(items.enum)) return items.enum as string[]
  }

  return []
}

/**
 * Get the default value for a schema field.
 * Returns undefined if no default is specified.
 */
export function getFieldDefault(path: string): unknown {
  const node = walkToNode(path)
  if (!node) return undefined
  return node.default
}

/**
 * Get the type constraint for a schema field.
 * Returns the "type" value or undefined.
 */
export function getFieldType(path: string): string | undefined {
  const node = walkToNode(path)
  if (!node) return undefined
  return node.type as string | undefined
}

/**
 * Get the pattern constraint for a schema field.
 * Returns the regex pattern string or undefined.
 */
export function getFieldPattern(path: string): string | undefined {
  const node = walkToNode(path)
  if (!node) return undefined
  return node.pattern as string | undefined
}

/**
 * Check if a field is required within its parent object.
 */
export function isFieldRequired(path: string): boolean {
  const segments = path.split('.')
  if (segments.length < 2) {
    // Top-level required
    const required = agentSchema.required as string[] | undefined
    return required?.includes(segments[0]) ?? false
  }

  // Walk to the parent, then check its required array
  const parentPath = segments.slice(0, -1).join('.')
  const fieldName = segments[segments.length - 1]
  const parent = walkToNode(parentPath)
  if (!parent) return false

  const required = parent.required as string[] | undefined
  return required?.includes(fieldName) ?? false
}

/**
 * Get all property names defined on a schema object node.
 */
export function getFieldProperties(path: string): string[] {
  const node = walkToNode(path)
  if (!node) return []
  const props = node.properties as SchemaNode | undefined
  if (!props) return []
  return Object.keys(props)
}
