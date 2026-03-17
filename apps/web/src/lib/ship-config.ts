/**
 * Convert a ProjectLibrary into .ship/ directory file contents.
 * Returns a map of file paths (relative to repo root) to their contents.
 */

import type { ProjectLibrary, McpServerConfig, Skill, Rule, Permissions } from '@ship/ui'

export function libraryToShipFiles(library: ProjectLibrary, modeName = 'default'): Record<string, string> {
  const files: Record<string, string> = {}

  // ship.toml — project identity
  files['.ship/ship.toml'] = buildShipToml(modeName)

  // agents/rules/*.md
  for (const rule of library.rules) {
    const filename = rule.file_name ?? `rule-${library.rules.indexOf(rule)}.md`
    const safeName = filename.replace(/[^a-zA-Z0-9._-]/g, '-')
    files[`.ship/agents/rules/${safeName}`] = rule.content
  }

  // agents/skills/<id>/SKILL.md
  for (const skill of library.skills) {
    files[`.ship/agents/skills/${skill.id}/SKILL.md`] = skill.content
  }

  // agents/mcp.toml
  if (library.mcp_servers.length > 0) {
    files['.ship/agents/mcp.toml'] = buildMcpToml(library.mcp_servers)
  }

  // agents/permissions.toml
  if (library.permissions) {
    files['.ship/agents/permissions.toml'] = buildPermissionsToml(library.permissions)
  }

  // agents/presets/<modeName>.toml — profile referencing all skills/mcp/rules
  files[`.ship/agents/presets/${modeName}.toml`] = buildPresetToml(library, modeName)

  return files
}

function buildShipToml(modeName: string): string {
  const id = nanoid(8)
  return [
    'version = "1"',
    `id = "${id}"`,
    '',
    '[defaults]',
    `profile = "${modeName}"`,
    'providers = ["claude", "gemini", "codex", "cursor"]',
    '',
  ].join('\n')
}

function buildMcpToml(servers: McpServerConfig[]): string {
  return servers
    .map((s) => {
      const lines = ['[[servers]]', `id = "${s.name}"`]
      if (s.command) lines.push(`command = "${s.command}"`)
      if (s.url) lines.push(`url = "${s.url}"`)
      if (s.args && s.args.length > 0) {
        const args = s.args.map((a) => `"${a}"`).join(', ')
        lines.push(`args = [${args}]`)
      }
      if (s.env && Object.keys(s.env).length > 0) {
        lines.push('')
        lines.push(`[servers.env]`)
        // This notation doesn't work for TOML arrays of tables;
        // emit env as inline key=value under the last [[servers]] block
        for (const [k, v] of Object.entries(s.env)) {
          lines.push(`${k} = "${v}"`)
        }
      }
      return lines.join('\n')
    })
    .join('\n\n')
    + '\n'
}

function buildPermissionsToml(permissions: Permissions): string {
  const lines: string[] = []

  if (permissions.allow && permissions.allow.length > 0) {
    const items = permissions.allow.map((a) => `"${a}"`).join(', ')
    lines.push(`allow = [${items}]`)
  }
  if (permissions.deny && permissions.deny.length > 0) {
    const items = permissions.deny.map((d) => `"${d}"`).join(', ')
    lines.push(`deny = [${items}]`)
  }

  return lines.join('\n') + '\n'
}

function buildPresetToml(library: ProjectLibrary, modeName: string): string {
  const lines = [
    `name = "${modeName}"`,
    `description = "Imported configuration"`,
    '',
  ]

  if (library.mcp_servers.length > 0) {
    const refs = library.mcp_servers.map((s) => `"${s.name}"`).join(', ')
    lines.push(`mcp_servers = [${refs}]`)
  }

  if (library.skills.length > 0) {
    const refs = library.skills.map((s) => `"${s.id}"`).join(', ')
    lines.push(`skills = [${refs}]`)
  }

  lines.push('')
  return lines.join('\n')
}

function nanoid(len: number): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
  const bytes = crypto.getRandomValues(new Uint8Array(len))
  return Array.from(bytes, (b) => chars[b % chars.length]).join('')
}
