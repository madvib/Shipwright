import { useState } from 'react'
import { Popover, PopoverContent, PopoverTrigger } from '@ship/primitives'
import {
  Terminal, Plug, Unplug, Loader2, Settings,
  ArrowUpToLine, Copy, CheckCheck, ExternalLink,
} from 'lucide-react'
import { toast } from 'sonner'
import { useLocalMcpContext } from './LocalMcpContext'
import { usePushBundle, useLocalAgentIds } from './mcp-queries'
import { useAgents } from '#/features/agents/useAgents'
import { useAgentDrafts } from '#/features/agents/useAgentDrafts'
import type { Skill, TransferBundle } from '@ship/ui'

const DOT: Record<string, string> = {
  disconnected: 'bg-muted-foreground/40',
  connecting: 'bg-amber-500 animate-pulse',
  connected: 'bg-emerald-500',
  error: 'bg-destructive',
}

const LABEL: Record<string, string> = {
  disconnected: 'Local sync',
  connecting: 'Connecting...',
  connected: 'Connected',
  error: 'Connection failed',
}

interface Props { onAddSkill: (skill: Skill) => void }

export function CliStatusPopover(_props: Props) {
  const mcp = useLocalMcpContext()
  if (!mcp) return null

  return (
    <Popover>
      <PopoverTrigger
        render={
          <button className="flex items-center gap-1.5 rounded-xl px-2.5 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-muted/50 transition">
            <Terminal className="size-3.5" strokeWidth={1.8} />
            <span className="text-[11px]">CLI</span>
            <span className={`size-1.5 rounded-full ${DOT[mcp.status]}`} />
          </button>
        }
      />
      <PopoverContent side="top" sideOffset={12} className="w-80 p-0">
        <PopoverBody mcp={mcp} />
      </PopoverContent>
    </Popover>
  )
}

function PopoverBody({ mcp }: {
  mcp: NonNullable<ReturnType<typeof useLocalMcpContext>>
}) {
  const [showSettings, setShowSettings] = useState(false)
  const [portInput, setPortInput] = useState(String(mcp.port))
  const [copied, setCopied] = useState(false)
  const [confirmPush, setConfirmPush] = useState(false)
  const { agents } = useAgents()
  const { drafts, clearDraft } = useAgentDrafts()
  const startCmd = `ship mcp serve --http --port ${mcp.port}`
  const isIdle = mcp.status !== 'connected' && mcp.status !== 'connecting'
  const isConnected = mcp.status === 'connected'

  const pushBundle = usePushBundle()
  useLocalAgentIds()

  // Find agents with drafts
  const draftAgentIds = Object.keys(drafts)
  const draftAgents = agents.filter((a) => draftAgentIds.includes(a.profile.id))

  const handlePortSave = () => {
    const p = parseInt(portInput, 10)
    if (p > 0 && p < 65536) mcp.setPort(p)
  }

  const handlePush = () => {
    if (draftAgents.length === 0) {
      toast.info('No unsaved changes to push')
      return
    }
    setConfirmPush(true)
  }

  const executePush = () => {
    setConfirmPush(false)
    for (const agent of draftAgents) {
      const bundle: TransferBundle = {
        agent: {
          id: agent.profile.id,
          name: agent.profile.name,
          description: agent.profile.description,
          version: agent.profile.version,
          providers: agent.profile.providers,
          skill_refs: agent.skills.map((s) => s.id),
          rule_refs: agent.rules.map((r) => r.file_name ?? r.content.slice(0, 30)),
          mcp_servers: agent.mcpServers.map((s) => s.name ?? s),
          permissions: agent.permissions as TransferBundle['agent']['permissions'],
          provider_settings: agent.providerSettings as TransferBundle['agent']['provider_settings'],
        },
        skills: Object.fromEntries(
          agent.skills.map((s) => [s.id, { files: { 'SKILL.md': s.content } }]),
        ),
        rules: Object.fromEntries(
          agent.rules.map((r) => [r.file_name ?? `rule-${r.content.slice(0, 20)}`, r.content]),
        ),
        dependencies: {},
      }
      pushBundle.mutate(bundle, {
        onSuccess: (result) => {
          clearDraft(agent.profile.id)
          toast.success(result)
        },
        onError: (err) => toast.error(err instanceof Error ? err.message : 'Push failed'),
      })
    }
  }

  return (
    <div className="p-3 space-y-3">
      <PopoverHeader mcp={mcp} onToggleSettings={() => setShowSettings(!showSettings)} />

      {mcp.error && <p className="text-[10px] text-destructive leading-snug">{mcp.error}</p>}

      {showSettings && <PortSettings portInput={portInput} setPortInput={setPortInput} onSave={handlePortSave} />}

      {isIdle && <DisconnectedView mcp={mcp} startCmd={startCmd} copied={copied} setCopied={setCopied} />}

      {isConnected && !confirmPush && (
        <div className="space-y-1.5">
          <SyncBtn
            icon={pushBundle.isPending ? <Loader2 className="size-3.5 text-primary animate-spin" /> : <ArrowUpToLine className="size-3.5 text-primary" />}
            label={pushBundle.isPending ? 'Pushing...' : 'Push to CLI'}
            desc={draftAgents.length > 0 ? `${draftAgents.length} agent${draftAgents.length !== 1 ? 's' : ''} modified` : 'No changes to push'}
            disabled={draftAgents.length === 0 || pushBundle.isPending}
            primary
            onClick={handlePush}
          />
        </div>
      )}

      {isConnected && confirmPush && (
        <div className="rounded-lg border border-amber-500/30 bg-amber-500/5 p-2.5 space-y-2">
          <p className="text-[11px] font-medium text-foreground">Push changes to CLI?</p>
          <p className="text-[10px] text-muted-foreground">
            {draftAgents.map((a) => a.profile.id).join(', ')}
          </p>
          <div className="flex gap-1.5">
            <button onClick={() => setConfirmPush(false)} className="flex-1 rounded-md border border-border/40 px-2 py-1 text-[10px] font-medium text-muted-foreground hover:text-foreground transition">Cancel</button>
            <button onClick={executePush} className="flex-1 rounded-md bg-primary px-2 py-1 text-[10px] font-medium text-primary-foreground hover:bg-primary/90 transition">Confirm</button>
          </div>
        </div>
      )}
    </div>
  )
}

// ── Sub-components ───────────────────────────────────────────────────────

function PopoverHeader({ mcp, onToggleSettings }: {
  mcp: NonNullable<ReturnType<typeof useLocalMcpContext>>
  onToggleSettings: () => void
}) {
  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-2">
        <span className={`size-2 rounded-full ${DOT[mcp.status]}`} />
        <span className="text-xs font-medium">
          {mcp.status === 'connected' ? (mcp.serverName ?? 'Connected') : LABEL[mcp.status]}
        </span>
      </div>
      <div className="flex items-center gap-1">
        <button onClick={onToggleSettings} className="rounded p-1 text-muted-foreground/40 hover:text-muted-foreground transition">
          <Settings className="size-3" />
        </button>
        {mcp.status === 'connected' ? (
          <button onClick={mcp.disconnect} className="rounded-md px-2 py-1 text-[10px] font-medium text-muted-foreground hover:text-foreground transition">
            <Unplug className="size-3" />
          </button>
        ) : (
          <button onClick={() => void mcp.connect()} disabled={mcp.status === 'connecting'} className="inline-flex items-center gap-1 rounded-md px-2 py-1 text-[10px] font-medium text-primary-foreground bg-primary hover:bg-primary/90 disabled:opacity-50 transition">
            {mcp.status === 'connecting' ? <Loader2 className="size-3 animate-spin" /> : <Plug className="size-3" />}
            Connect
          </button>
        )}
      </div>
    </div>
  )
}

function PortSettings({ portInput, setPortInput, onSave }: { portInput: string; setPortInput: (v: string) => void; onSave: () => void }) {
  return (
    <div className="rounded-lg border border-border/40 bg-background/60 p-2">
      <div className="flex items-center gap-2">
        <label className="text-[10px] text-muted-foreground whitespace-nowrap">Port</label>
        <input type="number" value={portInput} onChange={(e) => setPortInput(e.target.value)} onBlur={onSave} onKeyDown={(e) => e.key === 'Enter' && onSave()} className="w-20 rounded border border-border/40 bg-transparent px-1.5 py-0.5 text-[10px] font-mono text-foreground outline-none focus:border-primary/50" />
      </div>
    </div>
  )
}

function DisconnectedView({ mcp, startCmd, copied, setCopied }: {
  mcp: NonNullable<ReturnType<typeof useLocalMcpContext>>
  startCmd: string; copied: boolean; setCopied: (v: boolean) => void
}) {
  return (
    <>
      <div className="rounded-lg border border-border/40 bg-background/60 px-3 py-2">
        <div className="flex items-center gap-1.5">
          <code className="flex-1 text-[10px] font-mono text-emerald-400 truncate">{startCmd}</code>
          <button onClick={() => { void navigator.clipboard.writeText(startCmd).then(() => { setCopied(true); setTimeout(() => setCopied(false), 1500) }) }} className="shrink-0 rounded p-1 text-muted-foreground/40 hover:text-foreground transition">
            {copied ? <CheckCheck className="size-3 text-emerald-500" /> : <Copy className="size-3" />}
          </button>
        </div>
      </div>
      <p className="text-[9px] text-muted-foreground/50">Sync agents and skills between Studio and your local project.</p>
      {!mcp.hasEverConnected && (
        <a href="https://github.com/madvib/Ship#installation" target="_blank" rel="noopener noreferrer" className="flex items-center gap-2 rounded-lg border border-border/40 px-3 py-2 text-left transition hover:border-primary/30 hover:bg-primary/5 no-underline">
          <Terminal className="size-3.5 text-muted-foreground" />
          <span className="flex-1 text-[11px] font-medium text-foreground">Get the CLI</span>
          <ExternalLink className="size-3 text-muted-foreground/30" />
        </a>
      )}
    </>
  )
}

function SyncBtn({ icon, label, desc, disabled, primary, onClick }: {
  icon: React.ReactNode; label: string; desc: string; disabled?: boolean; primary?: boolean; onClick: () => void
}) {
  return (
    <button disabled={disabled} onClick={onClick} className={`w-full flex items-center gap-2.5 rounded-lg border px-3 py-2 text-left transition ${disabled ? 'border-border/40 opacity-30 cursor-not-allowed' : primary ? 'border-primary/30 bg-primary/5 hover:bg-primary/10' : 'border-border/40 hover:border-primary/30 hover:bg-primary/5'}`}>
      {icon}
      <div className="flex-1 min-w-0">
        <span className="text-[11px] font-medium text-foreground">{label}</span>
        <p className="text-[9px] text-muted-foreground/60">{desc}</p>
      </div>
    </button>
  )
}
