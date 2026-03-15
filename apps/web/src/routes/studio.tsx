import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback, useEffect, useRef } from 'react'
import {
  Search, Plus, Check, Download, Copy, CheckCheck,
  Server, BookOpen, ScrollText, Shield, Bot,
  Loader2, Zap, PanelLeft, X,
} from 'lucide-react'
import { useCompiler } from '../features/compiler/useCompiler'
import type { CompileState } from '../features/compiler/useCompiler'
import { ProvidersForm } from '../features/compiler/sections/ProvidersForm'
import { McpServersForm } from '../features/compiler/sections/McpServersForm'
import { SkillsForm } from '../features/compiler/sections/SkillsForm'
import { RulesForm } from '../features/compiler/sections/RulesForm'
import { PermissionsForm } from '../features/compiler/sections/PermissionsForm'
import { DEFAULT_LIBRARY, DEFAULT_PERMISSIONS } from '../features/compiler/types'
import type { ProjectLibrary, CompileResult } from '../features/compiler/types'
import { ProviderLogo } from '../features/compiler/ProviderLogo'
import type { McpServerConfig, Skill } from '@ship/ui'

export const Route = createFileRoute('/studio')({ component: StudioPage })

// ── Curated catalog ───────────────────────────────────────────────────────────

interface CuratedMcp {
  id: string
  displayName: string
  description: string
  icon: string
  config: McpServerConfig
}

const CURATED_MCP: CuratedMcp[] = [
  {
    id: 'github',
    displayName: 'GitHub',
    description: 'Search repos, manage PRs, create issues',
    icon: '⬡',
    config: { name: 'github', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'], env: { GITHUB_TOKEN: '$GITHUB_TOKEN' } },
  },
  {
    id: 'filesystem',
    displayName: 'Filesystem',
    description: 'Read and write local files safely',
    icon: '📁',
    config: { name: 'filesystem', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '.'] },
  },
  {
    id: 'brave-search',
    displayName: 'Brave Search',
    description: 'Web search via Brave API',
    icon: '🔍',
    config: { name: 'brave-search', command: 'npx', args: ['-y', '@modelcontextprotocol/server-brave-search'], env: { BRAVE_API_KEY: '$BRAVE_API_KEY' } },
  },
  {
    id: 'slack',
    displayName: 'Slack',
    description: 'Read channels, send messages',
    icon: '💬',
    config: { name: 'slack', command: 'npx', args: ['-y', '@modelcontextprotocol/server-slack'], env: { SLACK_BOT_TOKEN: '$SLACK_BOT_TOKEN', SLACK_TEAM_ID: '$SLACK_TEAM_ID' } },
  },
  {
    id: 'linear',
    displayName: 'Linear',
    description: 'Manage issues and project cycles',
    icon: '◈',
    config: { name: 'linear', command: 'npx', args: ['-y', '@linear/mcp-server'], env: { LINEAR_API_KEY: '$LINEAR_API_KEY' } },
  },
  {
    id: 'playwright',
    displayName: 'Playwright',
    description: 'Browser automation and testing',
    icon: '🎭',
    config: { name: 'playwright', command: 'npx', args: ['-y', '@executeautomation/playwright-mcp-server'] },
  },
  {
    id: 'postgres',
    displayName: 'PostgreSQL',
    description: 'Query and inspect your database',
    icon: '🐘',
    config: { name: 'postgres', command: 'npx', args: ['-y', '@modelcontextprotocol/server-postgres'], env: { DATABASE_URL: '$DATABASE_URL' } },
  },
  {
    id: 'memory',
    displayName: 'Memory',
    description: 'Persistent knowledge graph for context',
    icon: '🧠',
    config: { name: 'memory', command: 'npx', args: ['-y', '@modelcontextprotocol/server-memory'] },
  },
]

interface CuratedSkill {
  id: string
  displayName: string
  description: string
  skill: Skill
}

const CURATED_SKILLS: CuratedSkill[] = [
  {
    id: 'shipflow',
    displayName: 'Shipflow',
    description: 'Guides AI through planning, working, and session wrap-up',
    skill: {
      id: 'shipflow',
      name: 'Shipflow',
      description: 'Project intelligence workflow',
      content: `# Shipflow\n\nUse Ship to plan, execute, and wrap up work sessions.\n\n## Planning\nStart sessions with context: goals, active feature, current workspace.\n\n## Working  \nLog progress against specs. Reference ADRs for architectural decisions.\n\n## Wrapping up\nEnd sessions with a summary. Update spec status. Log what shipped.\n`,
    },
  },
  {
    id: 'commit',
    displayName: 'Smart Commit',
    description: 'Thoughtful, well-structured git commits',
    skill: {
      id: 'commit',
      name: 'Smart Commit',
      description: 'Git commit discipline',
      content: `# Smart Commit\n\nCreate focused, well-described git commits.\n\n## Rules\n- Imperative mood: "add feature" not "added feature"\n- First line ≤72 characters\n- Body explains WHY not WHAT\n- Atomic: one logical change per commit\n- Never skip hooks unless explicitly asked\n`,
    },
  },
  {
    id: 'code-review',
    displayName: 'Code Review',
    description: 'Systematic, constructive code review',
    skill: {
      id: 'code-review',
      name: 'Code Review',
      description: 'Code review process',
      content: `# Code Review\n\nSystematic code review focusing on correctness, clarity, and maintainability.\n\n## Process\n1. Understand intent before critiquing\n2. Check: correctness, edge cases, performance, security\n3. Suggest, don't dictate\n4. Prioritize: blocker > should-fix > nit\n`,
    },
  },
  {
    id: 'debugging',
    displayName: 'Debug Expert',
    description: 'Methodical root-cause debugging',
    skill: {
      id: 'debugging',
      name: 'Debug Expert',
      description: 'Debugging methodology',
      content: `# Debug Expert\n\nMethodical debugging: isolate, reproduce, understand, fix, verify.\n\n## Process\n1. Reproduce reliably before touching code\n2. Form a hypothesis. Test it.\n3. Fix root cause, not symptoms\n4. Add a test that would have caught this\n5. Document what was learned\n`,
    },
  },
  {
    id: 'create-document',
    displayName: 'Create Document',
    description: 'Structured specs, ADRs, and planning docs',
    skill: {
      id: 'create-document',
      name: 'Create Document',
      description: 'Document creation',
      content: `# Create Document\n\nCreate well-structured planning documents, specs, and architectural decision records.\n\n## Formats\n- **Spec**: problem statement, scope, success criteria, open questions\n- **ADR**: context, decision, consequences, alternatives considered\n- **Feature**: user story, acceptance criteria, implementation notes\n`,
    },
  },
]

// ── Types ─────────────────────────────────────────────────────────────────────

type ComposerSection = 'providers' | 'mcp' | 'skills' | 'rules' | 'permissions'

// ── Helper data ───────────────────────────────────────────────────────────────

const PROVIDER_SHORT: Record<string, string> = {
  claude: 'Claude',
  gemini: 'Gemini',
  codex: 'Codex',
  cursor: 'Cursor',
}

// ── InspectorTab helpers ──────────────────────────────────────────────────────

interface InspectorTab {
  id: string
  filename: string
  content: string
}

function getInspectorTabs(provider: string, result: CompileResult): InspectorTab[] {
  const tabs: InspectorTab[] = []

  const ctx = result.context_content
  if (ctx) {
    const name =
      provider === 'claude' ? 'CLAUDE.md'
      : provider === 'gemini' ? 'GEMINI.md'
      : provider === 'codex' ? 'AGENTS.md'
      : 'AGENTS.md'
    tabs.push({ id: 'context', filename: name, content: ctx })
  }

  if (result.mcp_servers) {
    const path =
      provider === 'gemini' ? '.gemini/settings.json'
      : provider === 'cursor' ? '.cursor/mcp.json'
      : '.mcp.json'
    tabs.push({ id: 'mcp', filename: path, content: JSON.stringify(result.mcp_servers, null, 2) })
  }

  if (result.claude_settings_patch) {
    tabs.push({ id: 'claude-settings', filename: '.claude/settings.json', content: JSON.stringify(result.claude_settings_patch, null, 2) })
  }

  if (result.codex_config_patch) {
    tabs.push({ id: 'codex-config', filename: '.codex/config.toml', content: result.codex_config_patch })
  }

  if (result.gemini_settings_patch && provider === 'gemini') {
    tabs.push({ id: 'gemini-settings', filename: '.gemini/settings.json', content: JSON.stringify(result.gemini_settings_patch, null, 2) })
  }

  if (result.gemini_policy_patch) {
    tabs.push({ id: 'gemini-policy', filename: '.gemini/policies/ship.toml', content: result.gemini_policy_patch })
  }

  if (Object.keys(result.rule_files ?? {}).length > 0 && provider === 'cursor') {
    const entries = Object.entries(result.rule_files ?? {})
    entries.forEach(([path, content]) => {
      tabs.push({ id: `rule-${path}`, filename: path, content })
    })
  }

  return tabs
}

function triggerDownload(text: string, name: string) {
  const blob = new Blob([text], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = name
  a.click()
  URL.revokeObjectURL(url)
}

// ── Main page ─────────────────────────────────────────────────────────────────

const STORAGE_KEY = 'ship-studio-v1'

function loadStored(): { library: ProjectLibrary; modeName: string; selectedProviders: string[] } | null {
  try {
    const raw = typeof window !== 'undefined' ? window.localStorage.getItem(STORAGE_KEY) : null
    if (!raw) return null
    return JSON.parse(raw) as { library: ProjectLibrary; modeName: string; selectedProviders: string[] }
  } catch {
    return null
  }
}

function StudioPage() {
  const stored = useRef(loadStored())
  const [library, setLibrary] = useState<ProjectLibrary>(stored.current?.library ?? DEFAULT_LIBRARY)
  const [modeName, setModeName] = useState(stored.current?.modeName ?? 'untitled-mode')
  const [selectedProviders, setSelectedProviders] = useState<string[]>(stored.current?.selectedProviders ?? ['claude', 'gemini', 'codex'])
  const [activeSection, setActiveSection] = useState<ComposerSection>('providers')
  const [showLibrary, setShowLibrary] = useState(true)
  const { state, compile } = useCompiler()

  const updateLibrary = useCallback((patch: Partial<ProjectLibrary>) => {
    setLibrary((prev) => ({ ...prev, ...patch }))
  }, [])

  // Persist to localStorage whenever library/modeName/providers change
  useEffect(() => {
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify({ library, modeName, selectedProviders }))
    } catch { /* ignore */ }
  }, [library, modeName, selectedProviders])

  // Auto-generate on library or provider changes (debounced 600ms).
  // Inject selectedProviders into a mode so WASM produces output for all of them.
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current)
    timerRef.current = setTimeout(() => {
      const effectiveLibrary = {
        ...library,
        modes: [{
          id: 'default',
          name: modeName || 'default',
          description: '',
          target_agents: selectedProviders,
          mcp_servers: [],
          skills: [],
          rules: [],
        }],
        active_mode: 'default',
      }
      void compile(effectiveLibrary)
    }, 600)
    return () => { if (timerRef.current) clearTimeout(timerRef.current) }
  }, [library, selectedProviders, modeName, compile])

  const addMcpServer = useCallback((config: McpServerConfig) => {
    setLibrary((prev) => {
      if (prev.mcp_servers.some((s) => s.name === config.name)) return prev
      return { ...prev, mcp_servers: [...prev.mcp_servers, config] }
    })
  }, [])

  const addSkill = useCallback((skill: Skill) => {
    setLibrary((prev) => {
      if (prev.skills.some((s) => s.id === skill.id)) return prev
      return { ...prev, skills: [...prev.skills, skill] }
    })
  }, [])

  const toggleProvider = useCallback((id: string) => {
    setSelectedProviders((prev) =>
      prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id]
    )
  }, [])

  return (
    <>
      {/* Mobile fallback — Studio requires a wide viewport */}
      <div className="flex md:hidden flex-col items-center justify-center gap-4 px-8 py-20 text-center min-h-[60vh]">
        <div className="flex size-12 items-center justify-center rounded-xl border border-border/60 bg-muted/40">
          <Zap className="size-5 text-muted-foreground" />
        </div>
        <div>
          <p className="font-display text-base font-semibold">Best on desktop</p>
          <p className="mt-1 text-sm text-muted-foreground max-w-xs">
            Ship Studio is a three-panel editor — open it on a wider screen for the full experience.
          </p>
        </div>
        <a
          href="/studio"
          className="inline-flex items-center gap-1.5 rounded-full border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
        >
          Try anyway
        </a>
      </div>

    <div className="hidden md:flex flex-1 min-h-0 flex-col overflow-hidden">
      {/* Mode header strip */}
      <ModeHeader
        modeName={modeName}
        onModeNameChange={setModeName}
        library={library}
        state={state}
        selectedProviders={selectedProviders}
        showLibrary={showLibrary}
        onToggleLibrary={() => setShowLibrary((v) => !v)}
      />

      {/* Three-panel body */}
      <div className="flex flex-1 min-h-0 overflow-hidden">
        {/* Left: Library */}
        {showLibrary && (
          <LibraryPanel
            library={library}
            onAddMcp={addMcpServer}
            onAddSkill={addSkill}
          />
        )}

        {/* Center: Composer */}
        <ComposerPanel
          library={library}
          activeSection={activeSection}
          selectedProviders={selectedProviders}
          onSectionChange={setActiveSection}
          onLibraryChange={updateLibrary}
          onToggleProvider={toggleProvider}
        />

        {/* Right: Inspector */}
        <InspectorPanel
          state={state}
          selectedProviders={selectedProviders}
        />
      </div>
    </div>
    </>
  )
}

// ── ModeHeader ────────────────────────────────────────────────────────────────

interface ModeHeaderProps {
  modeName: string
  onModeNameChange: (name: string) => void
  library: ProjectLibrary
  state: CompileState
  selectedProviders: string[]
  showLibrary: boolean
  onToggleLibrary: () => void
}

function ModeHeader({
  modeName,
  onModeNameChange,
  library,
  state,
  selectedProviders,
  showLibrary,
  onToggleLibrary,
}: ModeHeaderProps) {
  const isGenerating = state.status === 'compiling'
  const mcpCount = library.mcp_servers.length
  const skillCount = library.skills.length
  const ruleCount = library.rules.length

  return (
    <div className="flex items-center gap-2 border-b border-border/60 bg-card/50 px-3 py-2 shrink-0 backdrop-blur-sm">
      <button
        onClick={onToggleLibrary}
        title={showLibrary ? 'Hide library' : 'Show library'}
        className={`flex size-7 items-center justify-center rounded-md transition hover:bg-muted ${showLibrary ? 'text-foreground' : 'text-muted-foreground'}`}
      >
        <PanelLeft className="size-3.5" />
      </button>

      <div className="h-4 w-px bg-border/60" />

      {/* Mode name input */}
      <input
        value={modeName}
        onChange={(e) => onModeNameChange(e.target.value)}
        className="min-w-0 rounded px-1.5 py-0.5 font-display text-sm font-semibold text-foreground bg-transparent border border-transparent focus:border-border/60 focus:bg-card focus:outline-none transition w-40"
        placeholder="untitled-mode"
        spellCheck={false}
      />

      {/* Config summary badges */}
      <div className="hidden sm:flex items-center gap-1.5 ml-1">
        {mcpCount > 0 && (
          <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-semibold text-primary">{mcpCount} MCP</span>
        )}
        {skillCount > 0 && (
          <span className="rounded-full bg-cyan-500/10 px-2 py-0.5 text-[10px] font-semibold text-cyan-600 dark:text-cyan-400">{skillCount} skills</span>
        )}
        {ruleCount > 0 && (
          <span className="rounded-full bg-amber-500/10 px-2 py-0.5 text-[10px] font-semibold text-amber-600 dark:text-amber-400">{ruleCount} rules</span>
        )}
      </div>

      <div className="ml-auto flex items-center gap-2">
        {isGenerating && (
          <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
            <Loader2 className="size-3 animate-spin" />
            <span className="hidden sm:inline">Generating…</span>
          </div>
        )}
        {state.status === 'ok' && (
          <span className="hidden sm:inline text-[10px] text-muted-foreground">{state.elapsed}ms · WASM</span>
        )}
        <ExportButton state={state} selectedProviders={selectedProviders} />
      </div>
    </div>
  )
}

// ── ExportButton ──────────────────────────────────────────────────────────────

interface ExportButtonProps {
  state: CompileState
  selectedProviders: string[]
}

function ExportButton({ state, selectedProviders }: ExportButtonProps) {
  const [open, setOpen] = useState(false)
  const output = state.status === 'ok' ? state.output : null

  const downloadProvider = (p: string) => {
    if (!output?.[p]) return
    const result = output[p]
    const tabs = getInspectorTabs(p, result)
    tabs.forEach((tab) => triggerDownload(tab.content, tab.filename))
    setOpen(false)
  }

  const downloadAll = () => {
    if (!output) return
    selectedProviders.forEach((p) => {
      if (output[p]) getInspectorTabs(p, output[p]).forEach((tab) => triggerDownload(tab.content, tab.filename))
    })
    setOpen(false)
  }

  return (
    <div className="relative">
      <button
        onClick={() => (output ? setOpen((v) => !v) : undefined)}
        disabled={!output}
        className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-3 py-1.5 text-xs font-semibold text-primary-foreground transition hover:opacity-90 disabled:opacity-40"
      >
        <Download className="size-3" />
        Export
      </button>
      {open && output && (
        <>
          <div className="fixed inset-0 z-10" onClick={() => setOpen(false)} />
          <div className="absolute right-0 top-full z-20 mt-1 w-44 rounded-xl border border-border/60 bg-card shadow-lg overflow-hidden">
            {selectedProviders.map((p) =>
              output[p] ? (
                <button
                  key={p}
                  onClick={() => downloadProvider(p)}
                  className="flex w-full items-center gap-2 px-3 py-2 text-xs hover:bg-muted transition text-left"
                >
                  <ProviderLogo provider={p} />
                  {PROVIDER_SHORT[p] ?? p}
                </button>
              ) : null
            )}
            <div className="border-t border-border/60" />
            <button
              onClick={downloadAll}
              className="flex w-full items-center gap-2 px-3 py-2 text-xs font-medium hover:bg-muted transition text-left"
            >
              <Download className="size-3" />
              All providers
            </button>
          </div>
        </>
      )}
    </div>
  )
}

// ── LibraryPanel ──────────────────────────────────────────────────────────────

interface LibraryPanelProps {
  library: ProjectLibrary
  onAddMcp: (config: McpServerConfig) => void
  onAddSkill: (skill: Skill) => void
}

function LibraryPanel({ library, onAddMcp, onAddSkill }: LibraryPanelProps) {
  const [query, setQuery] = useState('')

  const addedMcpIds = new Set(library.mcp_servers.map((s) => s.name))
  const addedSkillIds = new Set(library.skills.map((s) => s.id))

  const filteredMcp = CURATED_MCP.filter(
    (m) =>
      !query ||
      m.displayName.toLowerCase().includes(query.toLowerCase()) ||
      m.description.toLowerCase().includes(query.toLowerCase())
  )
  const filteredSkills = CURATED_SKILLS.filter(
    (s) =>
      !query ||
      s.displayName.toLowerCase().includes(query.toLowerCase()) ||
      s.description.toLowerCase().includes(query.toLowerCase())
  )

  return (
    <aside className="hidden lg:flex w-72 shrink-0 flex-col border-r border-border/60 bg-sidebar/30">
      {/* Search */}
      <div className="p-2 border-b border-border/60">
        <div className="flex items-center gap-2 rounded-md border border-border/60 bg-background/60 px-2.5 py-1.5">
          <Search className="size-3 text-muted-foreground shrink-0" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search library…"
            className="flex-1 bg-transparent text-[11px] text-foreground placeholder:text-muted-foreground focus:outline-none min-w-0"
          />
          {query && (
            <button onClick={() => setQuery('')} className="text-muted-foreground hover:text-foreground">
              <X className="size-3" />
            </button>
          )}
        </div>
      </div>

      {/* Scrollable catalog */}
      <div className="flex-1 overflow-y-auto py-1">
        {/* MCP Servers section */}
        <div className="px-3 pt-3 pb-1">
          <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide flex items-center gap-1.5">
            <Server className="size-3" /> MCP Servers
          </p>
        </div>
        {filteredMcp.map((item) => {
          const isAdded = addedMcpIds.has(item.id)
          return (
            <LibraryItem
              key={item.id}
              name={item.displayName}
              description={item.description}
              icon={item.icon}
              isAdded={isAdded}
              onAdd={() => onAddMcp(item.config)}
            />
          )
        })}

        {/* Skills section */}
        <div className="px-3 pt-4 pb-1">
          <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide flex items-center gap-1.5">
            <BookOpen className="size-3" /> Skills
          </p>
        </div>
        {filteredSkills.map((item) => {
          const isAdded = addedSkillIds.has(item.id)
          return (
            <LibraryItem
              key={item.id}
              name={item.displayName}
              description={item.description}
              icon="◆"
              isAdded={isAdded}
              onAdd={() => onAddSkill(item.skill)}
            />
          )
        })}
      </div>

      {/* Footer hint */}
      <div className="border-t border-border/60 p-2.5">
        <p className="text-[10px] text-muted-foreground leading-relaxed">
          More in <a href="#" className="text-primary hover:underline">marketplace →</a>
        </p>
      </div>
    </aside>
  )
}

interface LibraryItemProps {
  name: string
  description: string
  icon: string
  isAdded: boolean
  onAdd: () => void
}

function LibraryItem({ name, description, icon, isAdded, onAdd }: LibraryItemProps) {
  return (
    <div className="group flex items-start gap-2 px-2 py-1.5 mx-1 rounded-md hover:bg-muted/50 transition">
      <span className="mt-0.5 text-sm shrink-0 w-4 text-center opacity-70">{icon}</span>
      <div className="flex-1 min-w-0">
        <p className="text-[11px] font-medium text-foreground truncate">{name}</p>
        <p className="text-[10px] text-muted-foreground line-clamp-2 leading-snug mt-0.5">{description}</p>
      </div>
      <button
        onClick={onAdd}
        disabled={isAdded}
        className={`shrink-0 mt-0.5 flex size-5 items-center justify-center rounded transition ${
          isAdded
            ? 'text-emerald-600 dark:text-emerald-400'
            : 'text-muted-foreground hover:text-primary hover:bg-primary/10 opacity-0 group-hover:opacity-100'
        }`}
        title={isAdded ? 'Added' : `Add ${name}`}
      >
        {isAdded ? <Check className="size-3" /> : <Plus className="size-3" />}
      </button>
    </div>
  )
}

// ── ComposerPanel ─────────────────────────────────────────────────────────────

const COMPOSER_TABS: Array<{ id: ComposerSection; label: string; icon: React.ElementType }> = [
  { id: 'providers', label: 'Providers', icon: Bot },
  { id: 'mcp', label: 'MCP', icon: Server },
  { id: 'skills', label: 'Skills', icon: BookOpen },
  { id: 'rules', label: 'Rules', icon: ScrollText },
  { id: 'permissions', label: 'Permissions', icon: Shield },
]

const SECTION_HELP: Record<ComposerSection, { title: string; description: string }> = {
  providers: { title: 'Target providers', description: 'Choose which AI coding assistants to build for.' },
  mcp: { title: 'MCP servers', description: 'Tools, APIs, and services your agents can call during a session.' },
  skills: { title: 'Skills', description: 'Instruction files injected into agent context — workflows, domain knowledge, repeated tasks.' },
  rules: { title: 'Rules', description: 'Always-active instructions included in every session.' },
  permissions: { title: 'Permissions', description: 'Control what tools, paths, and commands your agents can access.' },
}

interface SectionHeaderProps {
  section: ComposerSection
  library: ProjectLibrary
}

function SectionHeader({ section }: SectionHeaderProps) {
  const { title, description } = SECTION_HELP[section]
  return (
    <div className="mb-5">
      <h2 className="font-display text-sm font-semibold text-foreground">{title}</h2>
      <p className="mt-0.5 text-[11px] text-muted-foreground">{description}</p>
    </div>
  )
}

interface ComposerPanelProps {
  library: ProjectLibrary
  activeSection: ComposerSection
  selectedProviders: string[]
  onSectionChange: (section: ComposerSection) => void
  onLibraryChange: (patch: Partial<ProjectLibrary>) => void
  onToggleProvider: (id: string) => void
}

function ComposerPanel({
  library,
  activeSection,
  selectedProviders,
  onSectionChange,
  onLibraryChange,
  onToggleProvider,
}: ComposerPanelProps) {
  const mcpCount = library.mcp_servers.length
  const skillCount = library.skills.length
  const ruleCount = library.rules.length

  return (
    <div className="flex flex-1 min-w-0 flex-col border-r border-border/60">
      {/* Tab bar */}
      <div className="flex items-center gap-0.5 border-b border-border/60 bg-muted/20 px-2 py-1.5 shrink-0 overflow-x-auto [scrollbar-width:none]">
        {COMPOSER_TABS.map(({ id, label, icon: Icon }) => {
          const count = id === 'mcp' ? mcpCount : id === 'skills' ? skillCount : id === 'rules' ? ruleCount : 0
          return (
            <button
              key={id}
              onClick={() => onSectionChange(id)}
              className={`flex shrink-0 items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium transition ${
                activeSection === id
                  ? 'bg-card text-foreground shadow-sm'
                  : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'
              }`}
            >
              <Icon className="size-3.5" />
              {label}
              {count > 0 && (
                <span className="rounded-full bg-primary/15 px-1.5 py-0.5 text-[9px] font-bold text-primary">
                  {count}
                </span>
              )}
            </button>
          )
        })}
      </div>

      {/* Skills get full-height treatment; everything else scrolls */}
      {activeSection === 'skills' ? (
        <div className="flex flex-1 min-h-0 flex-col p-4 lg:p-5">
          <SectionHeader section={activeSection} library={library} />
          <div className="flex-1 min-h-0">
            <SkillsForm
              skills={library.skills}
              onChange={(skills) => onLibraryChange({ skills })}
            />
          </div>
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto">
          <div className="mx-auto max-w-3xl p-4 lg:p-6">
            <SectionHeader section={activeSection} library={library} />

            {activeSection === 'providers' && (
              <ProvidersForm
                selected={selectedProviders}
                onToggle={onToggleProvider}
              />
            )}
            {activeSection === 'mcp' && (
              <McpServersForm
                servers={library.mcp_servers}
                onChange={(mcp_servers) => onLibraryChange({ mcp_servers })}
              />
            )}
            {activeSection === 'rules' && (
              <RulesForm
                rules={library.rules}
                onChange={(rules) => onLibraryChange({ rules })}
              />
            )}
            {activeSection === 'permissions' && (
              <PermissionsForm
                permissions={library.permissions ?? DEFAULT_PERMISSIONS}
                onChange={(permissions) => onLibraryChange({ permissions })}
              />
            )}
          </div>
        </div>
      )}
    </div>
  )
}

// ── InspectorPanel ────────────────────────────────────────────────────────────

interface InspectorPanelProps {
  state: CompileState
  selectedProviders: string[]
}

function InspectorPanel({ state, selectedProviders }: InspectorPanelProps) {
  const [activeProvider, setActiveProvider] = useState(selectedProviders[0] ?? 'claude')
  const [activeFile, setActiveFile] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  // Keep active provider in sync when selectedProviders changes
  useEffect(() => {
    if (!selectedProviders.includes(activeProvider) && selectedProviders.length > 0) {
      setActiveProvider(selectedProviders[0])
    }
  }, [selectedProviders, activeProvider])

  const output = state.status === 'ok' ? state.output : null
  const current = output?.[activeProvider] ?? null
  const tabs = current ? getInspectorTabs(activeProvider, current) : []

  // Reset file tab when provider or output changes
  useEffect(() => {
    if (tabs.length > 0) setActiveFile(tabs[0].id)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeProvider, state.status])

  const displayTab = tabs.find((t) => t.id === activeFile) ?? tabs[0] ?? null
  const text = displayTab?.content ?? null

  const copy = () => {
    if (!text) return
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  const download = () => {
    if (!text || !displayTab) return
    triggerDownload(text, displayTab.filename)
  }

  const downloadProviderAll = () => {
    tabs.forEach((tab) => triggerDownload(tab.content, tab.filename))
  }

  return (
    <aside className="hidden md:flex w-96 xl:w-[420px] shrink-0 flex-col bg-sidebar/20">
      {/* Provider tabs */}
      <div className="flex items-center gap-0.5 border-b border-border/60 bg-muted/20 px-2 py-1.5 shrink-0 overflow-x-auto [scrollbar-width:none]">
        {selectedProviders.map((p) => (
          <button
            key={p}
            onClick={() => setActiveProvider(p)}
            className={`flex shrink-0 items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium transition ${
              activeProvider === p
                ? 'bg-card text-foreground shadow-sm'
                : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'
            }`}
          >
            <ProviderLogo provider={p} />
            {PROVIDER_SHORT[p] ?? p}
          </button>
        ))}
      </div>

      {/* File tabs (when output available) */}
      {tabs.length > 0 && (
        <div className="flex items-center justify-between border-b border-border/60 px-2 py-1 shrink-0">
          <div className="flex items-center gap-0.5 overflow-x-auto [scrollbar-width:none]">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveFile(tab.id)}
                className={`shrink-0 rounded px-2 py-1 text-[10px] font-mono font-medium transition ${
                  activeFile === tab.id
                    ? 'bg-card text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                {tab.filename}
              </button>
            ))}
          </div>
          <div className="flex items-center gap-0.5 shrink-0">
            <button
              onClick={copy}
              disabled={!text}
              className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-40 transition"
              title="Copy"
            >
              {copied ? <CheckCheck className="size-3 text-emerald-500" /> : <Copy className="size-3" />}
            </button>
            <button
              onClick={download}
              disabled={!text}
              className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-40 transition"
              title="Download file"
            >
              <Download className="size-3" />
            </button>
          </div>
        </div>
      )}

      {/* Content */}
      <div className="flex flex-1 min-h-0 flex-col overflow-hidden">
        {state.status === 'idle' && (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center">
            <div className="flex size-10 items-center justify-center rounded-xl border border-border/60 bg-muted/40">
              <Zap className="size-4 text-muted-foreground" />
            </div>
            <div>
              <p className="text-xs font-medium text-foreground">Your config will appear here</p>
              <p className="mt-1 text-[11px] text-muted-foreground">Add an MCP server or skill from the library</p>
            </div>
          </div>
        )}

        {state.status === 'compiling' && (
          <div className="flex flex-1 items-center justify-center">
            <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
              <Loader2 className="size-3.5 animate-spin" />
              Generating…
            </div>
          </div>
        )}

        {state.status === 'error' && (
          <div className="p-4">
            <p className="text-xs font-medium text-destructive mb-2">Generation failed</p>
            <pre className="text-[10px] text-destructive/80 leading-relaxed whitespace-pre-wrap">{state.message}</pre>
          </div>
        )}

        {state.status === 'ok' && (
          <>
            {text ? (
              <div className="flex-1 overflow-auto">
                <pre className="p-4 font-mono text-[11px] leading-relaxed text-foreground/80 whitespace-pre-wrap break-all">
                  {text}
                </pre>
              </div>
            ) : (
              <div className="flex flex-1 items-center justify-center">
                <p className="text-[11px] text-muted-foreground">No output for this file.</p>
              </div>
            )}
          </>
        )}
      </div>

      {/* Footer */}
      {state.status === 'ok' && (
        <div className="shrink-0 border-t border-border/60 bg-muted/20 px-3 py-2 flex items-center justify-between">
          <p className="text-[10px] text-muted-foreground">Generated in {state.elapsed}ms · WASM</p>
          <button
            onClick={downloadProviderAll}
            className="inline-flex items-center gap-1 rounded-md bg-primary px-2.5 py-1 text-[10px] font-semibold text-primary-foreground transition hover:opacity-90"
          >
            <Download className="size-2.5" />
            Export
          </button>
        </div>
      )}
    </aside>
  )
}
