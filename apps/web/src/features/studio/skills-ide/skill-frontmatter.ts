/**
 * Minimal YAML-frontmatter parser for skill files.
 * Does NOT pull in a full YAML library — handles flat key:value pairs
 * and simple arrays as found in SKILL.md frontmatter blocks.
 */

export interface SkillFrontmatter {
  name?: string
  id?: string
  version?: string
  description?: string
  author?: string
  license?: string
  allowed_tools?: string[]
  [key: string]: string | string[] | undefined
}

/** Extract frontmatter string between --- fences, or null if absent. */
export function extractFrontmatterBlock(content: string): string | null {
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/)
  return match ? match[1] : null
}

/** Parse flat YAML-like frontmatter into a dictionary. */
export function parseFrontmatter(content: string): SkillFrontmatter {
  const block = extractFrontmatterBlock(content)
  if (!block) return {}

  const result: SkillFrontmatter = {}
  for (const line of block.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue
    const colonIdx = trimmed.indexOf(':')
    if (colonIdx < 1) continue
    const key = trimmed.slice(0, colonIdx).trim()
    const rawVal = trimmed.slice(colonIdx + 1).trim()

    // Handle inline array: [Bash, Read, Grep]
    if (rawVal.startsWith('[') && rawVal.endsWith(']')) {
      result[key] = rawVal
        .slice(1, -1)
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean)
    } else {
      // Strip surrounding quotes
      result[key] = rawVal.replace(/^["']|["']$/g, '')
    }
  }
  return result
}

/** Known frontmatter fields from the agentskills.io spec. */
const KNOWN_FIELDS = new Set([
  'name', 'id', 'version', 'description', 'author', 'license',
  'allowed_tools', 'allowed-tools', 'compatibility', 'metadata',
])

export interface FrontmatterWarning {
  field: string
  message: string
  severity: 'error' | 'warning'
}

/** Validate frontmatter and return warnings/errors. */
export function validateFrontmatter(content: string): FrontmatterWarning[] {
  const block = extractFrontmatterBlock(content)
  if (!block) {
    return [{ field: '', message: 'Missing frontmatter block (--- fences)', severity: 'error' }]
  }

  const fm = parseFrontmatter(content)
  const warnings: FrontmatterWarning[] = []

  if (!fm.name) {
    warnings.push({ field: 'name', message: 'name is required', severity: 'error' })
  }
  if (!fm.version) {
    warnings.push({ field: 'version', message: 'version is recommended', severity: 'warning' })
  }
  if (!fm.description) {
    warnings.push({ field: 'description', message: 'description is recommended', severity: 'warning' })
  }

  // Flag unknown fields
  for (const key of Object.keys(fm)) {
    if (!KNOWN_FIELDS.has(key)) {
      warnings.push({ field: key, message: `Unknown field: ${key}`, severity: 'warning' })
    }
  }

  return warnings
}

/** Generate a SKILL.md frontmatter template for new skills. */
export function newSkillTemplate(name: string, id: string): string {
  return `---
name: ${name}
id: ${id}
version: 0.1.0
description:
author:
license: MIT
allowed_tools: []
---

# ${name}

Describe when and how to use this skill.

## When to use

-

## Instructions

`
}
