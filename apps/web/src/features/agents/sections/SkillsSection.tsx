import { Zap } from 'lucide-react'
import type { Skill } from '@ship/ui'
import { SectionShell, Chip, ChipIcon, AddChip } from './SectionShell'

function getInitials(name: string): string {
  const words = name.split(/[-_\s]/)
  if (words.length >= 2) {
    return (words[0][0] + words[1][0]).toUpperCase()
  }
  return name.slice(0, 2).toUpperCase()
}

interface SkillsSectionProps {
  skills: Skill[]
  onRemove: (skillId: string) => void
  onAdd?: () => void
}

export function SkillsSection({ skills, onRemove, onAdd }: SkillsSectionProps) {
  return (
    <SectionShell
      icon={<Zap className="size-4" />}
      title="Skills"
      count={`${skills.length} attached`}
      actionLabel="Add"
      onAction={onAdd}
    >
      <div className="flex flex-wrap gap-1.5">
        {skills.map((skill) => (
          <Chip
            key={skill.id}
            icon={<ChipIcon letters={getInitials(skill.name)} variant="skill" />}
            name={skill.name}
            meta={`${skill.metadata?.version ?? 'v0.1.0'} / ${skill.source ?? 'project'}`}
            onRemove={() => onRemove(skill.id)}
          />
        ))}
        <AddChip label="Add skill" onClick={onAdd} />
      </div>
    </SectionShell>
  )
}
