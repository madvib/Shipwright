import { useState, useDeferredValue } from 'react'
import {
  CommandDialog, CommandInput, CommandList, CommandEmpty, CommandGroup, CommandItem,
} from '@ship/primitives'
import { Badge } from '@ship/primitives'
import { Zap } from 'lucide-react'
import { useLibrary } from '#/features/compiler/useLibrary'
import type { Skill } from '@ship/ui'

interface AddSkillDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  existingIds: string[]
  onAdd: (skill: Skill) => void
}

export function AddSkillDialog({ open, onOpenChange, existingIds, onAdd }: AddSkillDialogProps) {
  const { library } = useLibrary()
  const [query, setQuery] = useState('')
  const deferredQuery = useDeferredValue(query)

  const allSkills = library.skills ?? []
  const available = allSkills.filter(
    (s) => !existingIds.includes(s.id) &&
      (s.name.toLowerCase().includes(deferredQuery.toLowerCase()) ||
       s.id.toLowerCase().includes(deferredQuery.toLowerCase())),
  )

  const handleSelect = (skill: Skill) => {
    onAdd(skill)
    onOpenChange(false)
    setQuery('')
  }

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput placeholder="Search skills..." value={query} onValueChange={setQuery} />
      <CommandList>
        <CommandEmpty>
          <div className="flex flex-col items-center py-6 text-center">
            <Zap className="size-5 text-muted-foreground mb-2" />
            <p className="text-sm text-muted-foreground">No skills found</p>
            <p className="text-xs text-muted-foreground/60 mt-1">Create one in the Skills IDE or browse the registry</p>
          </div>
        </CommandEmpty>
        {available.length > 0 && (
          <CommandGroup heading="Available skills">
            {available.map((skill) => (
              <CommandItem key={skill.id} onSelect={() => handleSelect(skill)} className="flex items-center gap-3">
                <div className="size-7 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
                  <Zap className="size-3.5 text-primary" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium">{skill.name}</span>
                    <Badge variant="secondary" className="text-[9px]">{skill.source ?? 'custom'}</Badge>
                  </div>
                  {skill.description && (
                    <p className="text-xs text-muted-foreground truncate">{skill.description}</p>
                  )}
                </div>
              </CommandItem>
            ))}
          </CommandGroup>
        )}
      </CommandList>
    </CommandDialog>
  )
}
