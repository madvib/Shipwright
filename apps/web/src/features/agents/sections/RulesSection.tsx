import { FileText } from 'lucide-react'
import type { Rule } from '@ship/ui'
import { SectionShell, OrangeDot } from './SectionShell'

interface RulesSectionProps {
  rules: Rule[]
}

export function RulesSection({ rules }: RulesSectionProps) {
  return (
    <SectionShell
      icon={<FileText className="size-4" />}
      title="Rules"
      count={`${rules.length} rules`}
      actionLabel="Add"
      showOrangeDot
    >
      <div className="flex flex-col gap-1.5">
        {rules.map((rule) => (
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
            <button className="shrink-0 text-[11px] text-muted-foreground/30 hover:text-primary transition-colors flex items-center gap-1">
              edit
              <OrangeDot />
            </button>
          </div>
        ))}
      </div>
    </SectionShell>
  )
}
