import { FileText, X } from 'lucide-react'
import type { DocKind } from './types'

interface SchemaPanelProps {
  docKinds: DocKind[]
  onClose: () => void
}

export function SchemaPanel({ docKinds, onClose }: SchemaPanelProps) {
  return (
    <aside className="flex w-72 shrink-0 flex-col border-l border-border/60 bg-card/50 backdrop-blur-sm">
      <div className="flex items-center justify-between border-b border-border/60 px-4 py-3">
        <div className="flex items-center gap-2">
          <FileText className="size-3.5 text-muted-foreground" />
          <h3 className="text-xs font-semibold text-foreground">Doc Schemas</h3>
        </div>
        <button
          onClick={onClose}
          className="flex size-5 items-center justify-center rounded text-muted-foreground hover:bg-muted hover:text-foreground transition"
        >
          <X className="size-3" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-2">
        {docKinds.map((dk) => (
          <div
            key={dk.kind}
            className="rounded-lg border border-border/50 bg-background/60 p-3"
          >
            <p className="text-[11px] font-mono font-semibold text-foreground mb-1.5">
              {dk.kind}
            </p>
            <div className="flex flex-wrap gap-1">
              {dk.requiredFields.map((f) => (
                <span
                  key={f}
                  className="rounded bg-muted px-1.5 py-0.5 text-[9px] font-mono text-muted-foreground"
                >
                  {f}
                </span>
              ))}
            </div>
          </div>
        ))}
      </div>

      <div className="border-t border-border/60 px-4 py-2.5">
        <p className="text-[10px] text-muted-foreground">
          Read-only · schema defined in workflow.toml
        </p>
      </div>
    </aside>
  )
}
