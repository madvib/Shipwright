import { useState } from 'react'
import { Popover, PopoverContent, PopoverTrigger } from '@ship/primitives'
import {
  Terminal, Plug, Unplug, Loader2, Settings,
  ArrowUpToLine, ArrowDownToLine, Copy, CheckCheck, ExternalLink,
} from 'lucide-react'
import { toast } from 'sonner'
import { useLocalMcpContext } from './LocalMcpContext'
import { usePushBundle, useLocalAgentIds } from './mcp-queries'
import { useAgentStore } from '#/features/agents/useAgentStore'
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

export function CliStatusPopover({ onAddSkill }: Props) {
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
        <PopoverBody mcp={mcp} onAddSkill={onAddSkill} />
      </PopoverContent>
    </Popover>
  )
}

function PopoverBody({ mcp, onAddSkill }: {
  mcp: NonNullable<ReturnType<typeof useLocalMcpContext>>
  onAddSkill: (skill: Skill) => void
}) {
  const [showSettings, setShowSettings] = useState(false)
  const [portInput, setPortInput] = useState(String(mcp.port))
  const [pulling, setPulling] = useState(false)
  const [copied, setCopied] = useState(false)
  const { agents, activeId, updateAgent } = useAgentStore()
  const activeAgent = activeId ? agents.find((a) => a.profile.id === activeId) : undefined
  const startCmd = `ship mcp serve --http --port ${mcp.port}`
  const isIdle = mcp.status !== 'connected' && mcp.status !== 'connecting'

  const pushBundle = usePushBundle()
  useLocalAgentIds() // keep agent list warm in query cache

  const handlePortSave = () => {
    const p = parseInt(portInput, 10)
    if (p > 0 && p < 65536) mcp.setPort(p)
  }

  const handlePush = () => {
    if (!activeAgent) return
    const bundle: TransferBundle = {
      agent: {
        id: activeAgent.profile.id,
        name: activeAgent.profile.name,
        description: activeAgent.profile.description,
        skills: activeAgent.skills.map((s) => s.id),
        rules: activeAgent.rules.map((r) => r.content),
        mcp_servers: activeAgent.mcpServers,
      },
      skills: Object.fromEntries(
        activeAgent.skills.map((s) => [s.id, { files: { 'SKILL.md': s.content } }]),
      ),
      dependencies: {},
    }
    pushBundle.mutate(bundle, {
      onSuccess: (result) => toast.success(result),
      onError: (err) => toast.error(err instanceof Error ? err.message : 'Push failed'),
    })
  }

  const handlePull = async () => {
    setPulling(true)
    try {
      const raw = await mcp.callTool('pull_agents')
      const parsed = JSON.parse(raw) as { agents: Array<{ profile: { id: string; name: string }; skills: Skill[] }> }
      let count = 0
      for (const a of parsed.agents) {
        for (const skill of a.skills ?? []) onAddSkill(skill)
        updateAgent(a.profile.id, a as Parameters<typeof updateAgent>[1])
        count++
      }
      toast.success(`Imported ${count} agent${count !== 1 ? 's' : ''} from CLI`)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Pull failed')
    } finally {
      setPulling(false)
    }
  }

  return (
    <div className="p-3 space-y-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className={`size-2 rounded-full ${DOT[mcp.status]}`} />
          <span className="text-xs font-medium">
            {mcp.status === 'connected' ? (mcp.serverName ?? 'Connected') : LABEL[mcp.status]}
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button onClick={() => setShowSettings(!showSettings)} className="rounded p-1 text-muted-foreground/40 hover:text-muted-foreground transition">
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

      {mcp.error && <p className="text-[10px] text-destructive leading-snug">{mcp.error}</p>}

      {showSettings && (
        <div className="rounded-lg border border-border/40 bg-background/60 p-2">
          <div className="flex items-center gap-2">
            <label className="text-[10px] text-muted-foreground whitespace-nowrap">Port</label>
            <input type="number" value={portInput} onChange={(e) => setPortInput(e.target.value)} onBlur={handlePortSave} onKeyDown={(e) => e.key === 'Enter' && handlePortSave()} className="w-20 rounded border border-border/40 bg-transparent px-1.5 py-0.5 text-[10px] font-mono text-foreground outline-none focus:border-primary/50" />
          </div>
        </div>
      )}

      {/* Disconnected: setup */}
      {isIdle && (
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
      )}

      {/* Connected: push/pull */}
      {mcp.status === 'connected' && (
        <div className="space-y-1.5">
          <SyncBtn icon={pushBundle.isPending ? <Loader2 className="size-3.5 text-primary animate-spin" /> : <ArrowUpToLine className="size-3.5 text-primary" />} label={pushBundle.isPending ? 'Pushing...' : 'Push to CLI'} desc={activeAgent ? `Write ${activeAgent.profile.id} to .ship/` : 'Select an agent first'} disabled={!activeAgent || pushBundle.isPending} primary onClick={handlePush} />
          <SyncBtn icon={pulling ? <Loader2 className="size-3.5 text-muted-foreground animate-spin" /> : <ArrowDownToLine className="size-3.5 text-muted-foreground" />} label={pulling ? 'Importing...' : 'Import from CLI'} desc="Pull .ship/ agents and skills into Studio" disabled={pulling} onClick={() => void handlePull()} />
        </div>
      )}
    </div>
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
