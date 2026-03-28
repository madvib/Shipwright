import { X, AlertTriangle, AlertCircle } from 'lucide-react'
import { useAgents } from '#/features/agents/useAgents'
import { parseFrontmatter, validateFrontmatter } from './skill-frontmatter'
import type { LibrarySkill } from './useSkillsLibrary'
import { VarsTab } from './preview-vars-tab'

type PreviewTab = 'vars' | 'info' | 'used-by'

interface Props {
  skill: LibrarySkill | null
  activeTab: string
  onTabChange: (tab: PreviewTab) => void
  onClose: () => void
  onAddFile?: (skillId: string, filePath: string, content: string) => void
}

const TABS: { id: PreviewTab; label: string }[] = [
  { id: 'vars', label: 'Variables' },
  { id: 'info', label: 'Info' },
  { id: 'used-by', label: 'Used by' },
]

// -- Info Tab -----------------------------------------------------------------

function InfoTab({ skill }: { skill: LibrarySkill }) {
  // Always validate the skill's SKILL.md content, not the active editor file
  const skillContent = skill.content
  const fm = parseFrontmatter(skillContent)
  const warnings = validateFrontmatter(skillContent)
  const tags = skill.tags.length > 0 ? skill.tags : (Array.isArray(fm.tags) ? fm.tags : [])

  return (
    <div className="space-y-4">
      {warnings.length > 0 && (
        <div className="space-y-1">
          {warnings.map((w, i) => (
            <div
              key={i}
              className={`flex items-start gap-1.5 text-[11px] px-2 py-1 rounded ${
                w.severity === 'error'
                  ? 'bg-destructive/10 text-destructive'
                  : 'bg-amber-500/10 text-amber-500'
              }`}
            >
              {w.severity === 'error'
                ? <AlertCircle className="size-3 shrink-0 mt-0.5" />
                : <AlertTriangle className="size-3 shrink-0 mt-0.5" />}
              <span>{w.message}</span>
            </div>
          ))}
        </div>
      )}

      <div>
        <h4 className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground mb-2">Identity</h4>
        <div className="space-y-0.5">
          {([
            ['stable-id', skill.stableId ?? fm['stable-id'] ?? skill.id],
            ['Authors', skill.authors.length > 0 ? skill.authors.join(', ') : '--'],
            ['License', fm.license ?? skill.license ?? '--'],
            ['Source', skill.origin],
          ] as [string, string][]).map(([key, val]) => (
            <div key={key} className="flex justify-between text-[11px] py-0.5">
              <span className="text-muted-foreground">{key}</span>
              <span className="text-foreground/80 truncate ml-2">{val}</span>
            </div>
          ))}
        </div>
      </div>

      {tags.length > 0 && (
        <div>
          <h4 className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground mb-2">Tags</h4>
          <div className="flex flex-wrap gap-1">
            {tags.map((tag) => (
              <span key={tag} className="bg-muted text-foreground/70 text-[10px] px-1.5 py-0.5 rounded">{tag}</span>
            ))}
          </div>
        </div>
      )}

      {skill.files.length > 0 && (
        <div>
          <h4 className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground mb-2">Files ({skill.files.length})</h4>
          <div className="space-y-0.5 font-mono text-[10px] text-foreground/70">
            {skill.files.map((f) => <div key={f}>{f}</div>)}
          </div>
        </div>
      )}

      {skill.description && (
        <div>
          <h4 className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground mb-2">Description</h4>
          <p className="text-[11px] text-foreground/70 leading-relaxed">{skill.description}</p>
        </div>
      )}
    </div>
  )
}

// -- Used By Tab --------------------------------------------------------------

function UsedByTab({ skill }: { skill: LibrarySkill }) {
  const { agents } = useAgents()
  const referencingAgents = agents.filter((a) => a.skills.some((s) => s.id === skill.id))

  if (referencingAgents.length === 0) {
    return (
      <div className="py-8 text-center">
        <p className="text-xs text-muted-foreground">No agents reference this skill.</p>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      {referencingAgents.map((agent) => (
        <div key={agent.profile.id} className="flex items-center gap-2 px-3 py-2 bg-card/60 border border-border rounded-md">
          <span className="text-xs font-bold text-primary">{agent.profile.name[0]?.toUpperCase() ?? '?'}</span>
          <div className="flex-1 min-w-0">
            <p className="text-xs text-foreground/80">{agent.profile.id}</p>
            <p className="text-[10px] text-muted-foreground">{agent.profile.name}</p>
          </div>
        </div>
      ))}
    </div>
  )
}

// -- Panel --------------------------------------------------------------------

export function SkillsPreviewPanel({ skill, activeTab, onTabChange, onClose, onAddFile }: Props) {
  if (!skill) return null

  const tab = (activeTab === 'vars' || activeTab === 'info' || activeTab === 'used-by') ? activeTab : 'vars'

  return (
    <div className="flex w-80 shrink-0 flex-col border-l border-border bg-card/10">
      <div className="flex items-center justify-between px-3.5 py-2 border-b border-border shrink-0">
        <h3 className="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">{skill.name || skill.id}</h3>
        <button onClick={onClose} className="text-muted-foreground hover:text-foreground transition-colors">
          <X className="size-3.5" />
        </button>
      </div>

      <div className="flex border-b border-border shrink-0">
        {TABS.map((t) => {
          const hasVars = t.id === 'vars' && skill.varsSchema != null
          return (
            <button
              key={t.id}
              onClick={() => onTabChange(t.id)}
              className={`flex-1 py-1.5 text-center text-[11px] border-b-2 transition-colors ${
                tab === t.id ? 'text-primary border-primary font-medium' : 'text-muted-foreground border-transparent hover:text-foreground'
              }`}
            >
              {t.label}
              {hasVars && <span className="ml-1 size-1.5 inline-block rounded-full bg-amber-400" />}
            </button>
          )
        })}
      </div>

      <div className="flex-1 overflow-y-auto p-3">
        {tab === 'vars' && (
          <VarsTab
            skill={skill}
            onAddVars={onAddFile ? () => {
              const defaultVars = JSON.stringify({
                $schema: 'https://getship.dev/schemas/vars.schema.json',
                example_var: {
                  type: 'string',
                  default: '',
                  'storage-hint': 'global',
                  label: 'Example variable',
                  description: 'Replace this with your first variable.',
                },
              }, null, 2)
              onAddFile(skill.id, 'assets/vars.json', defaultVars)
            } : undefined}
          />
        )}
        {tab === 'info' && <InfoTab skill={skill} />}
        {tab === 'used-by' && <UsedByTab skill={skill} />}
      </div>
    </div>
  )
}
