/** Schema-aware hints for vars.json editing in the Skills IDE. */

import { useMemo } from 'react'
import { CircleCheck, CircleAlert, Info } from 'lucide-react'

const VALID_TYPES = ['string', 'bool', 'enum', 'array', 'object'] as const
const VALID_STORAGE = ['global', 'local', 'project'] as const
const OPTIONAL_FIELDS = ['label', 'description', 'storage-hint'] as const

interface VarAnalysis {
  name: string
  missingFields: string[]
  invalidType: string | null
  invalidStorage: string | null
}

function analyzeVars(content: string): { vars: VarAnalysis[]; parseError: boolean } {
  try {
    const parsed = JSON.parse(content)
    if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
      return { vars: [], parseError: false }
    }

    const vars: VarAnalysis[] = []
    for (const [key, val] of Object.entries(parsed)) {
      if (key === '$schema') continue
      if (typeof val !== 'object' || val === null || Array.isArray(val)) continue
      const v = val as Record<string, unknown>

      const missingFields: string[] = []
      for (const field of OPTIONAL_FIELDS) {
        if (!v[field]) missingFields.push(field)
      }

      const invalidType = v.type && !VALID_TYPES.includes(v.type as typeof VALID_TYPES[number])
        ? String(v.type)
        : null

      const invalidStorage = v['storage-hint'] && !VALID_STORAGE.includes(v['storage-hint'] as typeof VALID_STORAGE[number])
        ? String(v['storage-hint'])
        : null

      vars.push({ name: key, missingFields, invalidType, invalidStorage })
    }

    return { vars, parseError: false }
  } catch {
    return { vars: [], parseError: true }
  }
}

export function SchemaHints({ content }: { content: string }) {
  const { vars, parseError } = useMemo(() => analyzeVars(content), [content])

  if (parseError || vars.length === 0) return null

  const hasErrors = vars.some((v) => v.invalidType || v.invalidStorage)
  const hasMissing = vars.some((v) => v.missingFields.length > 0)
  const isComplete = !hasErrors && !hasMissing

  return (
    <div className="border-t border-border px-4 py-2 text-[11px] shrink-0 bg-muted/30">
      <div className="flex items-center gap-2 mb-1.5">
        {isComplete ? (
          <>
            <CircleCheck className="size-3 text-emerald-500" />
            <span className="text-emerald-600 dark:text-emerald-400 font-medium">Schema complete</span>
          </>
        ) : (
          <>
            <Info className="size-3 text-muted-foreground" />
            <span className="text-muted-foreground font-medium">Schema hints</span>
          </>
        )}
        <div className="flex items-center gap-1 ml-auto">
          <span className="text-muted-foreground">type:</span>
          {VALID_TYPES.map((t) => (
            <span key={t} className="px-1 py-0.5 rounded bg-muted text-[10px] font-mono text-muted-foreground">{t}</span>
          ))}
        </div>
      </div>

      {!isComplete && (
        <div className="space-y-1">
          {vars.map((v) => {
            if (!v.invalidType && !v.invalidStorage && v.missingFields.length === 0) return null
            return (
              <div key={v.name} className="flex items-start gap-2">
                <span className="font-mono text-foreground/70 shrink-0">{v.name}</span>
                <div className="flex flex-wrap items-center gap-1">
                  {v.invalidType && (
                    <span className="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded bg-red-500/10 text-red-600 dark:text-red-400">
                      <CircleAlert className="size-2.5" />
                      type &quot;{v.invalidType}&quot; invalid
                    </span>
                  )}
                  {v.invalidStorage && (
                    <span className="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded bg-red-500/10 text-red-600 dark:text-red-400">
                      <CircleAlert className="size-2.5" />
                      storage-hint &quot;{v.invalidStorage}&quot; invalid
                    </span>
                  )}
                  {v.missingFields.map((f) => (
                    <span key={f} className="px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400">
                      + {f}
                    </span>
                  ))}
                </div>
              </div>
            )
          })}

          {hasErrors && (
            <div className="flex items-center gap-1 mt-1 pt-1 border-t border-border/50">
              <span className="text-muted-foreground">storage-hint:</span>
              {VALID_STORAGE.map((s) => (
                <span key={s} className="px-1 py-0.5 rounded bg-muted text-[10px] font-mono text-muted-foreground">{s}</span>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
