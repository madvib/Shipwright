import {
  CommandDialog, CommandInput, CommandList, CommandEmpty, CommandGroup, CommandItem,
} from '@ship/primitives'
import { Badge } from '@ship/primitives'
import { Users } from 'lucide-react'
import { useProfiles } from '#/features/studio/useProfiles'
import type { SubagentRef } from '../types'

interface AddSubagentDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentAgentId: string
  existingIds: string[]
  onAdd: (ref: SubagentRef) => void
}

export function AddSubagentDialog({ open, onOpenChange, currentAgentId, existingIds, onAdd }: AddSubagentDialogProps) {
  const { profiles } = useProfiles()
  const available = profiles.filter(
    (p) => p.id !== currentAgentId && !existingIds.includes(p.id),
  )

  const handleSelect = (p: typeof profiles[0]) => {
    onAdd({ id: p.id, name: p.name, description: `${p.skills.length} skills · ${p.mcpServers.length} MCP` })
    onOpenChange(false)
  }

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput placeholder="Search agents..." />
      <CommandList>
        <CommandEmpty>
          <div className="flex flex-col items-center py-6 text-center">
            <Users className="size-5 text-muted-foreground mb-2" />
            <p className="text-sm text-muted-foreground">No other agents available</p>
            <p className="text-xs text-muted-foreground/60 mt-1">Create more agents first</p>
          </div>
        </CommandEmpty>
        {available.length > 0 && (
          <CommandGroup heading="Your agents">
            {available.map((p) => (
              <CommandItem key={p.id} onSelect={() => handleSelect(p)} className="flex items-center gap-3">
                <div className="size-7 rounded-lg bg-violet-500/10 flex items-center justify-center shrink-0">
                  <span className="text-xs font-bold text-violet-500">{p.name.charAt(0).toUpperCase()}</span>
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium">{p.name}</span>
                    <Badge variant="secondary" className="text-[9px]">
                      {p.skills.length} skills
                    </Badge>
                  </div>
                  <p className="text-xs text-muted-foreground">{p.mcpServers.length} MCP servers</p>
                </div>
              </CommandItem>
            ))}
          </CommandGroup>
        )}
      </CommandList>
    </CommandDialog>
  )
}
