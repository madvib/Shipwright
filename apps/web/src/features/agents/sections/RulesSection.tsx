import { FileText } from 'lucide-react'
import type { Rule } from '@ship/ui'
import { SectionShell } from './SectionShell'

interface RulesSectionProps {
  rules: Rule[]
  onAdd?: () => void
  onEdit?: (index: number) => void
  onRemove?: (index: number) => void
}

export function RulesSection({ rules, onAdd, onEdit, onRemove }: RulesSectionProps) {
  return (
    <SectionShell
      icon={<FileText className="size-4" />}
      title="Rules"
      count={`${rules.length} rules`}
      actionLabel="Add"
      onAction={onAdd}
    >
      <div className="flex flex-col gap-1.5">
        {rules.map((rule, i) => (
          <div
            key={rule.file_name}
            className="flex items-center gap-2.5 rounded-lg border border-border/40 bg-card/30 px-3 py-2.5"
          >
            <span className="shrink-0 rounded bg-muted px-2 py-0.5 font-mono text-[11px] text-blue-400">
              {rule.file_name}
            </span>
            <span className="flex-1 truncate text-[11px] text-muted-foreground/40">
              {rule.content}
            </span>
            <button
              onClick={() => onEdit?.(i)}
              className="shrink-0 text-[11px] text-muted-foreground/30 hover:text-primary transition-colors"
            >
              edit
            </button>
            {onRemove && (
              <button
                onClick={() => onRemove(i)}
                className="shrink-0 text-muted-foreground/30 hover:text-destructive transition-colors text-sm"
              >
                x
              </button>
            )}
          </div>
        ))}
      </div>
    </SectionShell>
  )
}
