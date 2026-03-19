import { useState } from 'react'
import { Search, Plus, Check, Server, BookOpen, X } from 'lucide-react'
import type { McpServerConfig, Skill } from '@ship/ui'
import type { ProjectLibrary } from '#/features/compiler/types'

interface CuratedMcp {
  id: string
  displayName: string
  description: string
  icon: string
  config: McpServerConfig
}

export const CURATED_MCP: CuratedMcp[] = [
  {
    id: 'github',
    displayName: 'GitHub',
    description: 'Search repos, manage PRs, create issues',
    icon: '⬡',
    config: { name: 'github', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'], env: { GITHUB_TOKEN: '$GITHUB_TOKEN' }, url: null, timeout_secs: null },
  },
  {
    id: 'filesystem',
    displayName: 'Filesystem',
    description: 'Read and write local files safely',
    icon: '📁',
    config: { name: 'filesystem', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '.'], url: null, timeout_secs: null },
  },
  {
    id: 'brave-search',
    displayName: 'Brave Search',
    description: 'Web search via Brave API',
    icon: '🔍',
    config: { name: 'brave-search', command: 'npx', args: ['-y', '@modelcontextprotocol/server-brave-search'], env: { BRAVE_API_KEY: '$BRAVE_API_KEY' }, url: null, timeout_secs: null },
  },
  {
    id: 'slack',
    displayName: 'Slack',
    description: 'Read channels, send messages',
    icon: '💬',
    config: { name: 'slack', command: 'npx', args: ['-y', '@modelcontextprotocol/server-slack'], env: { SLACK_BOT_TOKEN: '$SLACK_BOT_TOKEN', SLACK_TEAM_ID: '$SLACK_TEAM_ID' }, url: null, timeout_secs: null },
  },
  {
    id: 'linear',
    displayName: 'Linear',
    description: 'Manage issues and project cycles',
    icon: '◈',
    config: { name: 'linear', command: 'npx', args: ['-y', '@linear/mcp-server'], env: { LINEAR_API_KEY: '$LINEAR_API_KEY' }, url: null, timeout_secs: null },
  },
  {
    id: 'playwright',
    displayName: 'Playwright',
    description: 'Browser automation and testing',
    icon: '🎭',
    config: { name: 'playwright', command: 'npx', args: ['-y', '@executeautomation/playwright-mcp-server'], url: null, timeout_secs: null },
  },
  {
    id: 'postgres',
    displayName: 'PostgreSQL',
    description: 'Query and inspect your database',
    icon: '🐘',
    config: { name: 'postgres', command: 'npx', args: ['-y', '@modelcontextprotocol/server-postgres'], env: { DATABASE_URL: '$DATABASE_URL' }, url: null, timeout_secs: null },
  },
  {
    id: 'memory',
    displayName: 'Memory',
    description: 'Persistent knowledge graph for context',
    icon: '🧠',
    config: { name: 'memory', command: 'npx', args: ['-y', '@modelcontextprotocol/server-memory'], url: null, timeout_secs: null },
  },
]

interface CuratedSkill {
  id: string
  displayName: string
  description: string
  skill: Skill
}

export const CURATED_SKILLS: CuratedSkill[] = [
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

export interface LibraryPanelProps {
  library: ProjectLibrary
  onAddMcp: (config: McpServerConfig) => void
  onAddSkill: (skill: Skill) => void
}

export function LibraryPanel({ library, onAddMcp, onAddSkill }: LibraryPanelProps) {
  const [query, setQuery] = useState('')

  const addedMcpIds = new Set((library.mcp_servers ?? []).map((s) => s.name))
  const addedSkillIds = new Set((library.skills ?? []).map((s) => s.id))

  const filteredMcp = CURATED_MCP.filter(
    (m) => !query || m.displayName.toLowerCase().includes(query.toLowerCase()) || m.description.toLowerCase().includes(query.toLowerCase()),
  )
  const filteredSkills = CURATED_SKILLS.filter(
    (s) => !query || s.displayName.toLowerCase().includes(query.toLowerCase()) || s.description.toLowerCase().includes(query.toLowerCase()),
  )

  return (
    <aside className="hidden lg:flex w-72 shrink-0 flex-col border-r border-border/60 bg-sidebar/30">
      <div className="p-2 border-b border-border/60">
        <div className="flex items-center gap-2 rounded-md border border-border/60 bg-background/60 px-2.5 py-1.5">
          <Search className="size-3 text-muted-foreground shrink-0" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search library…"
            className="flex-1 bg-transparent text-[11px] text-foreground placeholder:text-muted-foreground focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50 min-w-0"
          />
          {query && (
            <button onClick={() => setQuery('')} className="text-muted-foreground hover:text-foreground">
              <X className="size-3" />
            </button>
          )}
        </div>
      </div>
      <div className="flex-1 overflow-y-auto py-1">
        <div className="px-3 pt-3 pb-1">
          <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide flex items-center gap-1.5">
            <Server className="size-3" /> MCP Servers
          </p>
        </div>
        {filteredMcp.map((item) => (
          <LibraryItem
            key={item.id}
            name={item.displayName}
            description={item.description}
            icon={item.icon}
            isAdded={addedMcpIds.has(item.id)}
            onAdd={() => onAddMcp(item.config)}
          />
        ))}
        <div className="px-3 pt-4 pb-1">
          <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide flex items-center gap-1.5">
            <BookOpen className="size-3" /> Skills
          </p>
        </div>
        {filteredSkills.map((item) => (
          <LibraryItem
            key={item.id}
            name={item.displayName}
            description={item.description}
            icon="◆"
            isAdded={addedSkillIds.has(item.id)}
            onAdd={() => onAddSkill(item.skill)}
          />
        ))}
      </div>
      <div className="border-t border-border/60 p-2.5">
        <p className="text-[10px] text-muted-foreground leading-relaxed">
          More in <a href="#" className="text-primary hover:underline">marketplace →</a>
        </p>
      </div>
    </aside>
  )
}
