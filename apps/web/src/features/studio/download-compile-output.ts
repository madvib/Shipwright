import type { CompileResult, ProjectLibrary } from '#/features/compiler/types'
import type { AgentProfile } from '#/features/agents/types'

/** File entry for ZIP assembly */
interface ZipEntry {
  path: string
  data: Uint8Array
}

/**
 * Collect all output files from a CompileResult for a given provider.
 * Uses the same logic as getFileTabs in PublishPanel, plus skill_files and agent_files.
 */
function collectProviderFiles(provider: string, result: CompileResult): Array<{ path: string; content: string }> {
  const files: Array<{ path: string; content: string }> = []

  // Context file (CLAUDE.md, GEMINI.md, AGENTS.md)
  if (result.context_content) {
    const name = provider === 'claude' ? 'CLAUDE.md' : provider === 'gemini' ? 'GEMINI.md' : 'AGENTS.md'
    files.push({ path: name, content: result.context_content })
  }

  // MCP servers config
  if (result.mcp_servers) {
    const path = provider === 'gemini'
      ? '.gemini/settings.json'
      : provider === 'cursor'
        ? '.cursor/mcp.json'
        : '.mcp.json'
    files.push({ path, content: JSON.stringify(result.mcp_servers, null, 2) })
  }

  // Claude settings patch
  if (result.claude_settings_patch) {
    files.push({ path: '.claude/settings.json', content: JSON.stringify(result.claude_settings_patch, null, 2) })
  }

  // Codex config patch
  if (result.codex_config_patch) {
    files.push({ path: '.codex/config.toml', content: result.codex_config_patch })
  }

  // Gemini settings patch (gemini provider only)
  if (result.gemini_settings_patch && provider === 'gemini') {
    files.push({ path: '.gemini/settings.json', content: JSON.stringify(result.gemini_settings_patch, null, 2) })
  }

  // Gemini policy patch
  if (result.gemini_policy_patch) {
    files.push({ path: '.gemini/policies/ship.toml', content: result.gemini_policy_patch })
  }

  // Cursor hooks patch
  if (result.cursor_hooks_patch) {
    files.push({ path: '.cursor/hooks.json', content: JSON.stringify(result.cursor_hooks_patch, null, 2) })
  }

  // Cursor CLI permissions
  if (result.cursor_cli_permissions) {
    files.push({ path: '.cursor/cli.json', content: JSON.stringify(result.cursor_cli_permissions, null, 2) })
  }

  // Cursor environment JSON
  if (result.cursor_environment_json) {
    files.push({ path: '.cursor/environment.json', content: JSON.stringify(result.cursor_environment_json, null, 2) })
  }

  // OpenCode config patch
  if (result.opencode_config_patch) {
    files.push({ path: 'opencode.json', content: JSON.stringify(result.opencode_config_patch, null, 2) })
  }

  // Rule files
  for (const [path, content] of Object.entries(result.rule_files ?? {})) {
    if (content != null) files.push({ path, content })
  }

  // Skill files
  for (const [path, content] of Object.entries(result.skill_files ?? {})) {
    if (content != null) files.push({ path, content })
  }

  // Agent files
  for (const [path, content] of Object.entries(result.agent_files ?? {})) {
    if (content != null) files.push({ path, content })
  }

  return files
}

/** Build a minimal ZIP archive (STORE method, no compression, zero deps). */
function buildZip(entries: ZipEntry[]): Uint8Array {
  const encoder = new TextEncoder()
  const localParts: Uint8Array[] = []
  const centralParts: Uint8Array[] = []
  let offset = 0

  for (const entry of entries) {
    const nameBytes = encoder.encode(entry.path)
    const dataLen = entry.data.length

    // Local file header (30 bytes + name + data)
    const local = new ArrayBuffer(30 + nameBytes.length)
    const lv = new DataView(local)
    lv.setUint32(0, 0x04034b50, true)   // local file header signature
    lv.setUint16(4, 20, true)            // version needed (2.0)
    lv.setUint16(6, 0, true)             // general purpose bit flag
    lv.setUint16(8, 0, true)             // compression method: STORE
    lv.setUint16(10, 0, true)            // last mod file time
    lv.setUint16(12, 0, true)            // last mod file date
    lv.setUint32(14, crc32(entry.data), true)  // crc-32
    lv.setUint32(18, dataLen, true)      // compressed size
    lv.setUint32(22, dataLen, true)      // uncompressed size
    lv.setUint16(26, nameBytes.length, true) // file name length
    lv.setUint16(28, 0, true)            // extra field length
    new Uint8Array(local).set(nameBytes, 30)

    localParts.push(new Uint8Array(local))
    localParts.push(entry.data)

    // Central directory entry (46 bytes + name)
    const central = new ArrayBuffer(46 + nameBytes.length)
    const cv = new DataView(central)
    cv.setUint32(0, 0x02014b50, true)    // central directory signature
    cv.setUint16(4, 20, true)            // version made by
    cv.setUint16(6, 20, true)            // version needed
    cv.setUint16(8, 0, true)             // general purpose bit flag
    cv.setUint16(10, 0, true)            // compression method: STORE
    cv.setUint16(12, 0, true)            // last mod file time
    cv.setUint16(14, 0, true)            // last mod file date
    cv.setUint32(16, crc32(entry.data), true) // crc-32
    cv.setUint32(20, dataLen, true)      // compressed size
    cv.setUint32(24, dataLen, true)      // uncompressed size
    cv.setUint16(28, nameBytes.length, true)  // file name length
    cv.setUint16(30, 0, true)            // extra field length
    cv.setUint16(32, 0, true)            // file comment length
    cv.setUint16(34, 0, true)            // disk number start
    cv.setUint16(36, 0, true)            // internal file attributes
    cv.setUint32(38, 0, true)            // external file attributes
    cv.setUint32(42, offset, true)       // relative offset of local header
    new Uint8Array(central).set(nameBytes, 46)

    centralParts.push(new Uint8Array(central))

    offset += 30 + nameBytes.length + dataLen
  }

  const centralDirOffset = offset
  let centralDirSize = 0
  for (const part of centralParts) centralDirSize += part.length

  // End of central directory record (22 bytes)
  const eocd = new ArrayBuffer(22)
  const ev = new DataView(eocd)
  ev.setUint32(0, 0x06054b50, true)      // EOCD signature
  ev.setUint16(4, 0, true)               // disk number
  ev.setUint16(6, 0, true)               // disk with central dir
  ev.setUint16(8, entries.length, true)   // entries on this disk
  ev.setUint16(10, entries.length, true)  // total entries
  ev.setUint32(12, centralDirSize, true)  // central directory size
  ev.setUint32(16, centralDirOffset, true) // central directory offset
  ev.setUint16(20, 0, true)              // comment length

  // Concatenate all parts
  const totalSize = offset + centralDirSize + 22
  const result = new Uint8Array(totalSize)
  let pos = 0
  for (const parts of [localParts, centralParts]) {
    for (const part of parts) {
      result.set(part, pos)
      pos += part.length
    }
  }
  result.set(new Uint8Array(eocd), pos)

  return result
}

/** CRC-32 lookup table, computed once. */
const CRC_TABLE = (() => {
  const table = new Uint32Array(256)
  for (let i = 0; i < 256; i++) {
    let c = i
    for (let j = 0; j < 8; j++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
    }
    table[i] = c
  }
  return table
})()

/** Compute CRC-32 for a Uint8Array. */
function crc32(data: Uint8Array): number {
  let crc = 0xffffffff
  for (let i = 0; i < data.length; i++) {
    crc = CRC_TABLE[(crc ^ data[i]) & 0xff] ^ (crc >>> 8)
  }
  return (crc ^ 0xffffffff) >>> 0
}

/** Build a .ship/ship.jsonc manifest from library and agent data. */
function buildShipManifest(library?: ProjectLibrary, agent?: AgentProfile): string {
  const name = agent?.name ?? 'my-project'
  const description = agent?.description ?? ''
  const skillRefs = (agent?.skills ?? library?.skills ?? []).map((s) => s.id)
  const agentExports = agent ? [`agents/${agent.id}.jsonc`] : []

  const manifest: Record<string, unknown> = {
    $schema: 'https://getship.dev/schemas/ship.schema.json',
    module: { name, version: '0.1.0', description },
    exports: { skills: skillRefs, agents: agentExports },
  }
  return JSON.stringify(manifest, null, 2)
}

/** Build a .ship/agents/<id>.jsonc agent config from an AgentProfile. */
function buildAgentConfig(agent: AgentProfile): string {
  const mcpServerNames = agent.mcpServers.map((s) => s.name)
  const skillRefs = agent.skills.map((s) => s.id)

  const config: Record<string, unknown> = {
    $schema: 'https://getship.dev/schemas/agent.schema.json',
    agent: {
      id: agent.id,
      name: agent.name,
      providers: agent.providers,
    },
    skills: { refs: skillRefs },
    mcp: { servers: mcpServerNames },
    permissions: agent.permissions,
  }
  return JSON.stringify(config, null, 2)
}

/** Collect .ship/ source files for inclusion in the ZIP. */
function collectShipSourceFiles(
  library?: ProjectLibrary,
  activeAgent?: AgentProfile,
): Array<{ path: string; content: string }> {
  const files: Array<{ path: string; content: string }> = []
  files.push({ path: '.ship/ship.jsonc', content: buildShipManifest(library, activeAgent) })
  if (activeAgent) {
    files.push({
      path: `.ship/agents/${activeAgent.id}.jsonc`,
      content: buildAgentConfig(activeAgent),
    })
  }
  return files
}

/**
 * Download all compiled output files as a ZIP archive.
 *
 * Files are organized as `<provider>/<filename>` inside the ZIP.
 * Also includes `.ship/` source config for round-trippable CLI usage.
 * Triggers a browser download via a temporary object URL.
 */
export async function downloadCompileOutput(
  output: Record<string, CompileResult>,
  selectedProviders: string[],
  library?: ProjectLibrary,
  activeAgent?: AgentProfile,
): Promise<void> {
  const encoder = new TextEncoder()
  const entries: ZipEntry[] = []

  for (const provider of selectedProviders) {
    const result = output[provider]
    if (!result) continue

    const files = collectProviderFiles(provider, result)
    for (const file of files) {
      entries.push({
        path: `${provider}/${file.path}`,
        data: encoder.encode(file.content),
      })
    }
  }

  // Always include .ship/ source files regardless of selected providers
  const shipFiles = collectShipSourceFiles(library, activeAgent)
  for (const file of shipFiles) {
    entries.push({ path: file.path, data: encoder.encode(file.content) })
  }

  if (entries.length === 0) {
    throw new Error('No files to download')
  }

  const zipData = buildZip(entries)
  const blob = new Blob([zipData], { type: 'application/zip' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'ship-config.zip'
  a.click()
  URL.revokeObjectURL(url)
}

// Exported for testing
export { collectProviderFiles, collectShipSourceFiles, buildShipManifest, buildAgentConfig, buildZip, crc32 }
export type { ZipEntry }
