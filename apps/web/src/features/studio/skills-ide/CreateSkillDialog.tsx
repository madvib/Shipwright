import { useState } from 'react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'

interface CreateSkillDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onCreateSkill: (id: string) => void
  existingIds: string[]
}

export function CreateSkillDialog({ open, onOpenChange, onCreateSkill, existingIds }: CreateSkillDialogProps) {
  const [id, setId] = useState('')
  const [error, setError] = useState('')

  const validate = (value: string) => {
    if (!value) return ''
    if (!/^[a-z0-9-]+$/.test(value)) return 'Use lowercase letters, numbers, and hyphens only'
    if (existingIds.includes(value)) return 'A skill with this ID already exists'
    return ''
  }

  const handleCreate = () => {
    const trimmed = id.trim()
    const err = validate(trimmed)
    if (err || !trimmed) {
      setError(err || 'ID is required')
      return
    }
    onCreateSkill(trimmed)
    onOpenChange(false)
    setId('')
    setError('')
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Create skill</DialogTitle>
          <DialogDescription>
            Skills are markdown instructions that teach agents how to work. The ID becomes the folder name.
          </DialogDescription>
        </DialogHeader>

        <div className="py-4">
          <label className="text-xs font-medium text-foreground mb-1.5 block">Skill ID</label>
          <Input
            value={id}
            onChange={(e) => { setId(e.target.value); setError(validate(e.target.value)) }}
            placeholder="e.g. code-review, tdd, api-design"
            autoFocus
            className={error ? 'border-destructive' : ''}
          />
          {error && <p className="text-[11px] text-destructive mt-1.5">{error}</p>}
          <p className="text-[10px] text-muted-foreground mt-2">
            Creates <code className="bg-muted px-1 rounded text-[9px]">.ship/agents/skills/{id || '<id>'}/SKILL.md</code>
          </p>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button onClick={handleCreate} disabled={!id.trim() || !!error}>Create</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
