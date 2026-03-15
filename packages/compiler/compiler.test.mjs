/**
 * WASM integration tests for @ship/compiler.
 *
 * Runs against the actual compiled WASM binary — the same module the web
 * Studio loads. Tests the public API surface: compileLibrary,
 * compileLibraryAll, listProviders.
 *
 * Run: node compiler.test.mjs
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join, dirname } from 'node:path'
import { initSync, compileLibraryAll, compileLibrary, listProviders } from './compiler.js'

const __dirname = dirname(fileURLToPath(import.meta.url))

// ── Bootstrap WASM ────────────────────────────────────────────────────────────

const wasmBytes = readFileSync(join(__dirname, 'compiler_bg.wasm'))
initSync({ module: wasmBytes })

// ── Minimal test harness ─────────────────────────────────────────────────────

let passed = 0
let failed = 0
let suiteName = ''

function suite(name) {
  suiteName = name
  console.log(`\n${name}`)
}

function assert(condition, message) {
  if (condition) {
    console.log(`  ✓  ${message}`)
    passed++
  } else {
    console.error(`  ✗  ${message}`)
    failed++
  }
}

function assertThrows(fn, message) {
  try {
    fn()
    console.error(`  ✗  ${message} (expected throw, got none)`)
    failed++
  } catch {
    console.log(`  ✓  ${message}`)
    passed++
  }
}

// ── Fixtures ─────────────────────────────────────────────────────────────────

const DEFAULT_PERMISSIONS = {
  tools: { allow: [], deny: [] },
  filesystem: { allow: ['**/*'], deny: [] },
  commands: { allow: [], deny: [] },
  network: { policy: 'none', allow_hosts: [] },
  agent: { require_confirmation: [] },
}

const MINIMAL_LIBRARY = JSON.stringify({
  modes: [],
  active_mode: null,
  mcp_servers: [],
  skills: [],
  rules: [],
  permissions: DEFAULT_PERMISSIONS,
})

const FULL_LIBRARY = JSON.stringify({
  // Providers are driven by the active mode's target_agents.
  // Without a mode, only claude is produced (default fallback).
  modes: [
    {
      id: 'default',
      name: 'default',
      description: 'Default mode',
      target_agents: ['claude', 'gemini', 'codex'],
      mcp_servers: [],
      skills: [],
      rules: [],
    },
  ],
  active_mode: 'default',
  mcp_servers: [
    // NOTE: Rust McpServerConfig has both `id` (map key) and `name` (display).
    // `id` defaults to "" if omitted — always supply it explicitly.
    {
      id: 'github',
      name: 'GitHub',
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-github'],
      env: { GITHUB_TOKEN: '$GITHUB_TOKEN' },
    },
    {
      id: 'memory',
      name: 'Memory',
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-memory'],
    },
  ],
  skills: [
    {
      id: 'commit',
      name: 'Smart Commit',
      content: '# Smart Commit\n\nWrite atomic, well-described git commits.\n',
      description: 'Git commit discipline',
      // source omitted — defaults to "custom" in Rust; never pass null (non-optional enum)
      author: null,
      version: '1.0.0',
    },
  ],
  rules: [
    { file_name: 'code-style.md', content: 'Use TypeScript. Prefer explicit types.' },
    { file_name: 'no-hacks.md', content: 'No workarounds without a linked issue.' },
  ],
  permissions: {
    tools: { allow: [], deny: ['Bash(rm -rf*)'] },
    filesystem: { allow: ['**/*'], deny: ['/etc/**', '~/.ssh/**'] },
    commands: { allow: [], deny: [] },
    network: { policy: 'none', allow_hosts: [] },
    agent: { require_confirmation: [] },
  },
})

// ── Tests: listProviders ──────────────────────────────────────────────────────

suite('listProviders()')

const providers = listProviders()
assert(Array.isArray(providers), 'returns an array')
assert(providers.includes('claude'), 'includes claude')
assert(providers.includes('gemini'), 'includes gemini')
assert(providers.includes('codex'), 'includes codex')
assert(providers.length >= 3, 'at least 3 providers')

// ── Tests: compileLibraryAll — minimal library ────────────────────────────────

suite('compileLibraryAll() — minimal library')

const minimalRaw = compileLibraryAll(MINIMAL_LIBRARY)
assert(typeof minimalRaw === 'string', 'returns a string')
const minimal = JSON.parse(minimalRaw)
assert(typeof minimal === 'object' && minimal !== null, 'parses as JSON object')
assert('claude' in minimal || 'gemini' in minimal || 'codex' in minimal, 'has at least one provider key')

// ── Tests: compileLibraryAll — full library ───────────────────────────────────

suite('compileLibraryAll() — full library (MCP + skills + rules + permissions)')

const fullRaw = compileLibraryAll(FULL_LIBRARY)
const full = JSON.parse(fullRaw)

// Claude output
assert('claude' in full, 'claude key present')
const claude = full.claude

// context_content = rules (always-on instructions), not skill content
assert(typeof claude.context_content === 'string', 'claude has context_content')
assert(claude.context_content.includes('Use TypeScript'), 'claude context includes rule content')
assert(claude.context_content.includes('No workarounds'), 'claude context includes second rule')

// Skills go to skill_files (path → content map), not context_content
assert(typeof claude.skill_files === 'object', 'claude has skill_files')
const skillPaths = Object.keys(claude.skill_files)
assert(skillPaths.length > 0, 'claude has at least one skill file')
assert(skillPaths.some((p) => p.includes('commit')), 'commit skill file is present')
const commitContent = Object.values(claude.skill_files).find((v) => v.includes('Smart Commit'))
assert(commitContent !== undefined, 'commit skill content includes skill name')

// MCP servers — keyed by `id` field, Ship MCP always auto-injected
assert(claude.mcp_servers !== null, 'claude has mcp_servers')
assert('github' in claude.mcp_servers, 'claude mcp_servers includes github by id')
assert('memory' in claude.mcp_servers, 'claude mcp_servers includes memory by id')
assert('ship' in claude.mcp_servers, 'ship MCP server always auto-injected')
assert(claude.mcp_servers.github.command === 'npx', 'github server command is npx')

// Gemini output
assert('gemini' in full, 'gemini key present')
const gemini = full.gemini
assert(typeof gemini.context_content === 'string', 'gemini has context_content')
assert(typeof gemini.skill_files === 'object', 'gemini has skill_files')

// Codex output
assert('codex' in full, 'codex key present')
const codex = full.codex
assert(typeof codex.context_content === 'string', 'codex has context_content')

// Rule files (Cursor .mdc format)
assert(typeof claude.rule_files === 'object', 'claude has rule_files')

// Permissions deny wired to settings patch
if (claude.claude_settings_patch !== null) {
  assert(typeof claude.claude_settings_patch === 'object', 'claude settings patch is object')
}

// ── Tests: compileLibrary — single provider ───────────────────────────────────

suite('compileLibrary() — single provider')

const claudeRaw = compileLibrary(FULL_LIBRARY, 'claude')
const claudeSingle = JSON.parse(claudeRaw)
assert(typeof claudeSingle.context_content === 'string', 'claude single result has context_content')
assert(claudeSingle.provider === 'claude', 'provider field matches requested provider')
assert(typeof claudeSingle.skill_files === 'object', 'claude single result has skill_files')
assert(Object.values(claudeSingle.skill_files).some((c) => c.includes('Smart Commit')), 'skill content present')

const geminiRaw = compileLibrary(FULL_LIBRARY, 'gemini')
const geminiSingle = JSON.parse(geminiRaw)
assert(typeof geminiSingle.context_content === 'string', 'gemini single result has context_content')
assert(geminiSingle.provider === 'gemini', 'gemini provider field correct')

const codexRaw = compileLibrary(FULL_LIBRARY, 'codex')
const codexSingle = JSON.parse(codexRaw)
assert(typeof codexSingle.context_content === 'string', 'codex single result has context_content')
assert(codexSingle.provider === 'codex', 'codex provider field correct')

// ── Tests: compileLibraryAll — active_mode ────────────────────────────────────

suite('compileLibraryAll() — active_mode filter')

const libraryWithModes = JSON.stringify({
  modes: [
    {
      id: 'planning',
      name: 'Planning',
      description: 'Planning mode',
      target_agents: ['claude'],
    },
  ],
  active_mode: 'planning',
  mcp_servers: [],
  skills: [],
  rules: [],
  permissions: DEFAULT_PERMISSIONS,
})

const withModeRaw = compileLibraryAll(libraryWithModes, 'planning')
const withMode = JSON.parse(withModeRaw)
assert(typeof withMode === 'object', 'mode-filtered result is object')

// ── Tests: error handling ─────────────────────────────────────────────────────

suite('error handling')

assertThrows(
  () => compileLibrary('not valid json', 'claude'),
  'compileLibrary throws on invalid JSON'
)

assertThrows(
  () => compileLibraryAll('not valid json'),
  'compileLibraryAll throws on invalid JSON'
)

// ── Tests: output stability ───────────────────────────────────────────────────

suite('output stability — same input produces same output')

const run1 = compileLibraryAll(FULL_LIBRARY)
const run2 = compileLibraryAll(FULL_LIBRARY)
assert(run1 === run2, 'identical inputs produce identical outputs')

// ── Summary ───────────────────────────────────────────────────────────────────

console.log(`\n${'─'.repeat(50)}`)
console.log(`  ${passed} passed  ${failed > 0 ? failed + ' failed' : ''}`)
console.log(`${'─'.repeat(50)}\n`)

if (failed > 0) {
  process.exit(1)
}
