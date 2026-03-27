import { Globe, FolderOpen, User } from 'lucide-react'
import type { JsonValue } from '@ship/ui'
import type { LibrarySkill } from './useSkillsLibrary'

interface VarDef {
  type?: string
  default?: JsonValue
  'storage-hint'?: string
  values?: string[]
  label?: string
  description?: string
}

function parseVarsSchema(schema: JsonValue | null): Record<string, VarDef> {
  if (!schema || typeof schema !== 'object' || Array.isArray(schema)) return {}
  const result: Record<string, VarDef> = {}
  for (const [key, val] of Object.entries(schema as Record<string, JsonValue>)) {
    if (key === '$schema') continue
    if (val && typeof val === 'object' && !Array.isArray(val)) {
      result[key] = val as unknown as VarDef
    }
  }
  return result
}

function scopeIcon(hint: string | undefined) {
  switch (hint) {
    case 'project': return <FolderOpen className="size-3 text-emerald-400" />
    case 'local': return <User className="size-3 text-amber-400" />
    default: return <Globe className="size-3 text-sky-400" />
  }
}

function scopeLabel(hint: string | undefined) {
  switch (hint) {
    case 'project': return 'project'
    case 'local': return 'local'
    default: return 'global'
  }
}

function formatDefault(val: JsonValue | undefined): string {
  if (val === undefined || val === null) return '--'
  if (typeof val === 'boolean') return val ? 'true' : 'false'
  if (typeof val === 'string') return val || '""'
  return JSON.stringify(val)
}

export function VarsTab({ skill, onAddVars }: { skill: LibrarySkill; onAddVars?: () => void }) {
  const vars = parseVarsSchema(skill.varsSchema)
  const varKeys = Object.keys(vars)

  if (varKeys.length === 0) {
    return (
      <div className="py-8 text-center">
        <p className="text-xs text-muted-foreground">No variables defined.</p>
        <p className="text-[11px] text-muted-foreground mt-2">
          Variables make this a smart skill — same skill, personalized output.
        </p>
        {onAddVars && (
          <button
            onClick={onAddVars}
            className="mt-3 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-border bg-muted/50 text-xs font-medium text-foreground hover:bg-muted transition-colors"
          >
            Add vars.json
          </button>
        )}
      </div>
    )
  }

  return (
    <div className="space-y-3">
      <p className="text-[11px] text-muted-foreground font-mono">
        ship vars set {skill.stableId ?? skill.id} &lt;key&gt; &lt;value&gt;
      </p>

      {varKeys.map((key) => {
        const v = vars[key]
        const varType = v.type ?? 'string'
        return (
          <div key={key} className="border border-border rounded-lg p-2.5 bg-card/60">
            <div className="flex items-center justify-between mb-1">
              <span className="text-xs font-medium text-foreground">{v.label ?? key}</span>
              <div className="flex items-center gap-1.5">
                <span className="text-[9px] font-mono bg-muted px-1.5 py-0.5 rounded text-muted-foreground">{varType}</span>
                <span className="flex items-center gap-0.5" title={scopeLabel(v['storage-hint'])}>{scopeIcon(v['storage-hint'])}</span>
              </div>
            </div>
            {v.description && (
              <p className="text-[11px] text-muted-foreground mb-2 leading-relaxed">{v.description}</p>
            )}
            {varType === 'bool' && (
              <div className="flex items-center gap-2">
                <div className={`w-7 h-4 rounded-full ${v.default === true ? 'bg-primary' : 'bg-muted'}`}>
                  <div className={`size-3 rounded-full bg-white mt-0.5 transition-transform ${v.default === true ? 'translate-x-3.5' : 'translate-x-0.5'}`} />
                </div>
                <span className="text-[11px] text-muted-foreground">default: {v.default === true ? 'true' : 'false'}</span>
              </div>
            )}
            {varType === 'enum' && v.values && (
              <div className="flex flex-wrap gap-1">
                {v.values.map((val) => (
                  <span
                    key={val}
                    className={`text-[10px] px-1.5 py-0.5 rounded border ${
                      val === v.default ? 'border-primary/50 bg-primary/10 text-primary font-medium' : 'border-border bg-muted text-muted-foreground'
                    }`}
                  >
                    {val}
                  </span>
                ))}
              </div>
            )}
            {(varType === 'string' || varType === 'array' || varType === 'object') && (
              <div className="text-[11px] text-muted-foreground">
                default: <span className="font-mono">{formatDefault(v.default)}</span>
              </div>
            )}
            <div className="flex items-center gap-1 text-[10px] text-muted-foreground mt-1.5">
              {scopeIcon(v['storage-hint'])}
              <span>{scopeLabel(v['storage-hint'])}</span>
              <span className="mx-0.5">·</span>
              <span className="font-mono">{key}</span>
            </div>
          </div>
        )
      })}
    </div>
  )
}
