import { useState, useMemo } from 'react'
import { useNavigate } from '@tanstack/react-router'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { makeAgent } from '#/features/agents/make-agent'
import { buildTransferBundle } from '#/features/studio/build-transfer-bundle'
import { usePushBundle } from '#/features/studio/mcp-queries'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import { getFieldEnum, getFieldDescription } from '#/features/agents/schema-hints'
import { validateAgentProfile } from '#/features/agents/schema-validation'
import { toast } from 'sonner'

interface CreateAgentDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CreateAgentDialog({ open, onOpenChange }: CreateAgentDialogProps) {
  const navigate = useNavigate()
  const pushBundle = usePushBundle()
  const { connected: isConnected } = useDaemon()

  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [selectedProviders, setSelectedProviders] = useState<string[]>(['claude'])
  const [validationErrors, setValidationErrors] = useState<string[]>([])

  const schemaProviders = useMemo(() => getFieldEnum('agent.providers'), [])
  const providerHint = useMemo(() => getFieldDescription('agent.providers'), [])

  const toggleProvider = (id: string) => {
    setSelectedProviders((prev) =>
      prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id],
    )
  }

  const handleCreate = () => {
    if (!name.trim()) return
    const agent = makeAgent({
      profile: { id: '', name: name.trim(), description: description.trim(), providers: selectedProviders },
    })
    const result = validateAgentProfile(agent)
    if (!result.valid) {
      setValidationErrors(result.errors.map((e) => e.message))
      return
    }
    setValidationErrors([])

    if (isConnected) {
      const bundle = buildTransferBundle(agent)
      pushBundle.mutate(bundle, {
        onSuccess: () => {
          onOpenChange(false)
          resetForm()
          void navigate({ to: '/studio/agents/$id', params: { id: agent.profile.id } })
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : 'Failed to create agent')
        },
      })
    } else {
      toast.error('Daemon not connected — cannot create agents')
    }
  }

  const resetForm = () => {
    setName('')
    setDescription('')
    setSelectedProviders(['claude'])
    setValidationErrors([])
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
            {providerHint && (
              <p className="text-[11px] text-muted-foreground/60 mb-1.5">{providerHint}</p>
            )}
            <div className="flex flex-wrap gap-2">
              {schemaProviders.map((id) => {
                const active = selectedProviders.includes(id)
                return (
                  <button
                    key={id}
                    type="button"
                    onClick={() => toggleProvider(id)}
                    className={`inline-flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs font-medium transition capitalize ${
                      active
                        ? 'border-primary/30 bg-primary/10 text-primary'
                        : 'border-border/60 text-muted-foreground hover:border-border'
                    }`}
                  >
                    {id}
                  </button>
                )
              })}
            </div>
          </div>

          {!isConnected && (
            <p className="text-[11px] text-amber-500">
              Daemon not connected. Agents are stored in your local .ship/ directory.
            </p>
          )}

          {validationErrors.length > 0 && (
            <div className="rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2">
              {validationErrors.map((err, i) => (
                <p key={i} className="text-xs text-destructive">{err}</p>
              ))}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button
            onClick={handleCreate}
            disabled={!name.trim() || !isConnected || pushBundle.isPending}
          >
            {pushBundle.isPending ? 'Creating...' : 'Create agent'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
