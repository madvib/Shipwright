import { X, FileText } from 'lucide-react'
import type { Skill } from '@ship/ui'
import { useAgentStore } from '#/features/agents/useAgentStore'
import { parseFrontmatter } from './skill-frontmatter'

interface Props {
  skill: Skill | null
  content: string
  activeTab: 'metadata' | 'output' | 'used-by'
  onTabChange: (tab: 'metadata' | 'output' | 'used-by') => void
  onClose: () => void
}

const TABS: { id: 'metadata' | 'output' | 'used-by'; label: string }[] = [
  { id: 'metadata', label: 'Metadata' },
  { id: 'output', label: 'Output' },
  { id: 'used-by', label: 'Used by' },
]

function MetadataTab({ skill, content }: { skill: Skill; content: string }) {
  const fm = parseFrontmatter(content)
  const tools = fm.allowed_tools ?? skill.allowed_tools ?? []
  const { agents } = useAgentStore()
  const attachedAgents = agents.filter((a) => a.skills.some((s) => s.id === skill.id))

  return (
    <div className="space-y-4">
      {/* Identity */}
      <div>
        <h4 className="text-[10px] font-semibold uppercase tracking-[0.04em] text-muted-foreground/40 mb-2">
          Identity
        </h4>
        <div className="space-y-0.5">
          {[
            ['ID', fm.id ?? skill.id],
            ['Version', fm.version ?? '0.1.0'],
            ['Author', fm.author ?? 'unknown'],
            ['License', fm.license ?? skill.license ?? '--'],
            ['Source', skill.source ?? 'project'],
          ].map(([key, val]) => (
            <div key={key} className="flex justify-between text-[11px] py-0.5">
              <span className="text-muted-foreground/40">{key}</span>
              <span className="text-muted-foreground/70">{val}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Allowed Tools */}
      {tools.length > 0 && (
        <div>
          <h4 className="text-[10px] font-semibold uppercase tracking-[0.04em] text-muted-foreground/40 mb-2">
            Allowed Tools
          </h4>
          <div className="flex flex-wrap gap-1">
            {tools.map((tool) => (
              <span
                key={tool}
                className="bg-primary/10 text-primary text-[10px] px-1.5 py-0.5 rounded"
              >
                {tool}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Attached to Agents */}
      <div>
        <h4 className="text-[10px] font-semibold uppercase tracking-[0.04em] text-muted-foreground/40 mb-2">
          Attached to Agents
        </h4>
        {attachedAgents.length === 0 ? (
          <p className="text-[11px] italic text-muted-foreground/30">Not attached to any agents.</p>
        ) : (
          <div className="space-y-1.5">
            {attachedAgents.map((agent) => (
              <div
                key={agent.profile.id}
                className="flex items-center gap-2 px-2 py-1.5 bg-card/60 border border-border/30 rounded-md text-[11px]"
              >
                <span className="font-semibold text-primary">{agent.profile.name[0]?.toUpperCase() ?? '?'}</span>
                <span className="text-muted-foreground/70">{agent.profile.id}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

function OutputTab({ skill, content }: { skill: Skill; content: string }) {
  const fm = parseFrontmatter(content)
  const tools = fm.allowed_tools ?? skill.allowed_tools ?? []

  // Build a simplified compiled output preview
  const previewLines = [
    { text: '---', cls: 'text-muted-foreground/30' },
    { text: `name: ${fm.name ?? skill.name}`, cls: '', key: 'name:', val: fm.name ?? skill.name },
    ...(tools.length > 0
      ? [
          { text: 'allowed_tools:', cls: '', key: 'allowed_tools:', val: '' },
          ...tools.map((t) => ({ text: `  - ${t}`, cls: '', key: '', val: t })),
        ]
      : []),
    { text: '---', cls: 'text-muted-foreground/30' },
    { text: '', cls: '' },
    { text: `# ${fm.name ?? skill.name}`, cls: 'text-emerald-300' },
    { text: fm.description ?? skill.description ?? '...', cls: 'text-muted-foreground/40' },
  ]

  return (
    <div>
      <div className="bg-card/60 border border-border/30 rounded-lg p-3">
        <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground/40 font-semibold mb-2">
          <FileText className="size-3" />
          .claude/skills/{skill.id}/SKILL.md
        </div>
        <div className="font-mono text-[10px] leading-[1.6] space-y-0">
          {previewLines.map((line, i) => (
            <div key={i} className={line.cls}>
              {line.key ? (
                <>
                  <span className="text-sky-300">{line.key}</span>
                  {line.val && <span className="text-amber-300"> {line.val}</span>}
                </>
              ) : (
                <span>{line.text || '\u00A0'}</span>
              )}
            </div>
          ))}
        </div>
      </div>
      <div className="mt-3 flex items-center gap-1.5 text-[10px] text-muted-foreground/30">
        <span className="size-1.5 rounded-full bg-primary" />
        WASM compiler output preview
      </div>
    </div>
  )
}

function UsedByTab({ skill }: { skill: Skill }) {
  const { agents } = useAgentStore()
  const referencingAgents = agents.filter((a) => a.skills.some((s) => s.id === skill.id))

  return (
    <div className="space-y-2">
      <p className="text-[11px] text-muted-foreground/40 mb-3">
        Agents and profiles that reference this skill.
      </p>
      {referencingAgents.length === 0 ? (
        <p className="text-[11px] italic text-muted-foreground/30">No agents reference this skill.</p>
      ) : (
        referencingAgents.map((agent) => (
          <div
            key={agent.profile.id}
            className="flex items-center gap-2 px-3 py-2 bg-card/60 border border-border/30 rounded-md"
          >
            <span className="text-xs font-bold text-primary">{agent.profile.name[0]?.toUpperCase() ?? '?'}</span>
            <div className="flex-1 min-w-0">
              <p className="text-xs text-foreground/80">{agent.profile.id}</p>
              <p className="text-[10px] text-muted-foreground/30">{agent.profile.name}</p>
            </div>
          </div>
        ))
      )}
    </div>
  )
}

export function SkillsPreviewPanel({ skill, content, activeTab, onTabChange, onClose }: Props) {
  if (!skill) return null

  return (
    <div className="flex w-80 shrink-0 flex-col border-l border-border/30 bg-card/10">
      {/* Header */}
      <div className="flex items-center justify-between px-3.5 py-2 border-b border-border/30 shrink-0">
        <h3 className="text-[11px] font-semibold uppercase tracking-[0.05em] text-muted-foreground/40">
          Skill Info
        </h3>
        <button
          onClick={onClose}
          className="text-muted-foreground/30 hover:text-muted-foreground transition-colors"
        >
          <X className="size-3.5" />
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-border/30 shrink-0">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={`flex-1 py-1.5 text-center text-[10px] border-b-2 transition-colors ${
              activeTab === tab.id
                ? 'text-primary border-primary'
                : 'text-muted-foreground/30 border-transparent hover:text-muted-foreground/50'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto p-3">
        {activeTab === 'metadata' && <MetadataTab skill={skill} content={content} />}
        {activeTab === 'output' && <OutputTab skill={skill} content={content} />}
        {activeTab === 'used-by' && <UsedByTab skill={skill} />}
      </div>
    </div>
  )
}
