import { Users } from 'lucide-react'
import type { SubagentRef } from '../types'
import { SectionShell, Chip, ChipIcon, AddChip } from './SectionShell'

function getInitials(name: string): string {
  const words = name.split(/[-_\s]/)
  if (words.length >= 2) return (words[0][0] + words[1][0]).toUpperCase()
  return name.slice(0, 2).toUpperCase()
}

interface SubagentsSectionProps {
  subagents: SubagentRef[]
  onRemove: (id: string) => void
}

export function SubagentsSection({ subagents, onRemove }: SubagentsSectionProps) {
  return (
    <SectionShell
      icon={<Users className="size-4" />}
      title="Subagents"
      count={`${subagents.length} attached`}
      actionLabel="Add"
      showOrangeDot
    >
      <div className="flex flex-wrap gap-1.5">
        {subagents.map((agent) => (
          <Chip
            key={agent.id}
            icon={<ChipIcon letters={getInitials(agent.name)} variant="agent" />}
            name={agent.name}
            meta={agent.description}
            onRemove={() => onRemove(agent.id)}
          />
        ))}
        <AddChip label="Add agent" showOrangeDot />
      </div>
    </SectionShell>
  )
}
