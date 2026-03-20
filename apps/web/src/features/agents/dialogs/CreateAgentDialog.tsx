import { useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { useProfiles } from '#/features/studio/useProfiles'
import { PROVIDERS } from '#/features/compiler/types'

interface CreateAgentDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CreateAgentDialog({ open, onOpenChange }: CreateAgentDialogProps) {
  const { addProfile, updateProfile } = useProfiles()
  const navigate = useNavigate()
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [selectedProviders, setSelectedProviders] = useState<string[]>(['claude'])

  const toggleProvider = (id: string) => {
    setSelectedProviders((prev) =>
      prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id],
    )
  }

  const handleCreate = () => {
    if (!name.trim()) return
    const id = addProfile()
    updateProfile(id, {
      name: name.trim(),
      persona: description.trim(),
      selectedProviders,
    })
    onOpenChange(false)
    setName('')
    setDescription('')
    setSelectedProviders(['claude'])
    void navigate({ to: '/studio/agents/$id', params: { id } })
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create agent</DialogTitle>
          <DialogDescription>
            Define a new agent configuration. You can add skills, MCP servers, and permissions after creation.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Name</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. web-lane, rust-runtime, qa-engineer"
              autoFocus
            />
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Description</label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What does this agent specialize in?"
            />
          </div>

          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Target providers</label>
            <div className="flex flex-wrap gap-2">
              {PROVIDERS.map((p) => {
                const active = selectedProviders.includes(p.id)
                return (
                  <button
                    key={p.id}
                    type="button"
                    onClick={() => toggleProvider(p.id)}
                    className={`inline-flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs font-medium transition ${
                      active
                        ? 'border-primary/30 bg-primary/10 text-primary'
                        : 'border-border/60 text-muted-foreground hover:border-border'
                    }`}
                  >
                    {p.name.split(' ')[0]}
                  </button>
                )
              })}
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button onClick={handleCreate} disabled={!name.trim()}>Create agent</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
