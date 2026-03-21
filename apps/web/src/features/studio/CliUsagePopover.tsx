import { useState, useCallback } from 'react'
import { Copy, Check, Terminal } from 'lucide-react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription,
} from '@ship/primitives'

interface CliUsagePopoverProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  agentName?: string
}

export function CliUsagePopover({ open, onOpenChange, agentName }: CliUsagePopoverProps) {
  const useCommand = agentName ? `ship use ${agentName}` : 'ship use'
  const installCommand = agentName ? `ship install ${agentName}` : null

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Terminal className="size-4 text-muted-foreground" />
            Use with CLI
          </DialogTitle>
          <DialogDescription>
            Install this agent's config in your project with a single command.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <CommandBlock label="Activate in current project" command={useCommand} />
          {installCommand && (
            <CommandBlock label="Install from registry" command={installCommand} />
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}

function CommandBlock({ label, command }: { label: string; command: string }) {
  const [copied, setCopied] = useState(false)

  const copy = useCallback(() => {
    void navigator.clipboard.writeText(command).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }, [command])

  return (
    <div>
      <p className="text-[11px] font-medium text-muted-foreground mb-1.5">{label}</p>
      <div className="flex items-center gap-2 rounded-lg border border-border/60 bg-muted/40 px-3 py-2">
        <code className="flex-1 font-mono text-xs text-foreground select-all">{command}</code>
        <button
          type="button"
          onClick={copy}
          className="shrink-0 rounded-md p-1 text-muted-foreground transition hover:text-foreground hover:bg-muted"
          aria-label={copied ? 'Copied' : `Copy command: ${command}`}
        >
          {copied
            ? <Check className="size-3.5 text-emerald-500" />
            : <Copy className="size-3.5" />}
        </button>
      </div>
    </div>
  )
}
