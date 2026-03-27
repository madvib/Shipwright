import { useState } from 'react'
import {
  Terminal, Paintbrush, Info, Check, Copy, Wifi, WifiOff, Loader2,
  ExternalLink,
} from 'lucide-react'
import { ThemeToggle } from '@ship/primitives'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import type { McpConnectionStatus } from '#/features/studio/useLocalMcp'
import { SettingsSection, SettingsRow } from './SettingsLayout'

// ── CLI Connection ──────────────────────────────────────────────────────────

function StatusBadge({ status }: { status: McpConnectionStatus }) {
  const config: Record<McpConnectionStatus, { label: string; color: string }> = {
    connected: { label: 'Connected', color: 'bg-emerald-500' },
    connecting: { label: 'Connecting', color: 'bg-amber-500 animate-pulse' },
    disconnected: { label: 'Disconnected', color: 'bg-muted-foreground' },
    error: { label: 'Error', color: 'bg-destructive' },
  }
  const { label, color } = config[status]
  return (
    <div className="flex items-center gap-1.5">
      <span className={`size-2 rounded-full ${color}`} />
      <span className="text-[11px] text-muted-foreground">{label}</span>
    </div>
  )
}

function CopyCommand({ port }: { port: number }) {
  const [copied, setCopied] = useState(false)
  const command = `ship mcp serve --http --port ${port}`

  function handleCopy() {
    void navigator.clipboard.writeText(command)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  return (
    <div className="flex items-center gap-2 rounded-md border border-border/60 bg-muted/30 px-2.5 py-1.5">
      <code className="flex-1 font-mono text-[11px] text-muted-foreground">{command}</code>
      <button
        onClick={handleCopy}
        className="shrink-0 text-muted-foreground transition hover:text-foreground"
        title="Copy command"
      >
        {copied ? <Check className="size-3" /> : <Copy className="size-3" />}
      </button>
    </div>
  )
}

export function CLIConnectionSection() {
  const mcp = useLocalMcpContext()

  if (!mcp) {
    return (
      <SettingsSection icon={<Terminal className="size-[15px]" />} title="CLI Connection">
        <p className="text-[11px] text-muted-foreground">MCP context unavailable.</p>
      </SettingsSection>
    )
  }

  const isConnected = mcp.status === 'connected'
  const isConnecting = mcp.status === 'connecting'

  return (
    <SettingsSection icon={<Terminal className="size-[15px]" />} title="CLI Connection">
      <SettingsRow label="Status">
        <StatusBadge status={mcp.status} />
      </SettingsRow>

      {isConnected && mcp.serverName && (
        <SettingsRow label="Server">
          <span className="text-[11px] text-muted-foreground">{mcp.serverName}</span>
        </SettingsRow>
      )}

      {mcp.error && (
        <SettingsRow label="Error">
          <span className="max-w-[300px] text-right text-[11px] text-destructive">{mcp.error}</span>
        </SettingsRow>
      )}

      <SettingsRow label="Port">
        <input
          type="number"
          value={mcp.port}
          onChange={(e) => mcp.setPort(Number(e.target.value))}
          disabled={isConnected || isConnecting}
          className="w-24 rounded-md border border-border/60 bg-muted/30 px-2 py-1 text-right text-[11px] text-foreground outline-none focus:border-ring disabled:opacity-50"
        />
      </SettingsRow>

      <SettingsRow label="Connection">
        {isConnected ? (
          <button
            onClick={mcp.disconnect}
            className="flex items-center gap-1.5 rounded-md border border-border/60 bg-muted/30 px-3 py-1 text-[11px] text-muted-foreground transition hover:border-destructive hover:text-destructive"
          >
            <WifiOff className="size-3" />
            Disconnect
          </button>
        ) : (
          <button
            onClick={() => void mcp.connect()}
            disabled={isConnecting}
            className="flex items-center gap-1.5 rounded-md border border-primary/40 bg-primary/10 px-3 py-1 text-[11px] text-primary transition hover:bg-primary/20 disabled:opacity-50"
          >
            {isConnecting ? (
              <Loader2 className="size-3 animate-spin" />
            ) : (
              <Wifi className="size-3" />
            )}
            Connect
          </button>
        )}
      </SettingsRow>

      <div className="mt-2 border-t border-border/30 pt-2">
        <p className="mb-1.5 text-[11px] text-muted-foreground">Start the MCP server with:</p>
        <CopyCommand port={mcp.port} />
      </div>
    </SettingsSection>
  )
}

// ── Appearance ───────────────────────────────────────────────────────────────

export function AppearanceSection() {
  return (
    <SettingsSection icon={<Paintbrush className="size-[15px]" />} title="Appearance">
      <SettingsRow label="Theme">
        <ThemeToggle variant="switch" />
      </SettingsRow>
    </SettingsSection>
  )
}

// ── About ────────────────────────────────────────────────────────────────────

export function AboutSection() {
  return (
    <SettingsSection icon={<Info className="size-[15px]" />} title="About">
      <SettingsRow label="Version">
        <span className="text-[11px] text-muted-foreground">0.1.0</span>
      </SettingsRow>
      <SettingsRow label="Documentation">
        <a
          href="https://getship.dev"
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-1 text-[11px] text-primary transition hover:underline"
        >
          getship.dev
          <ExternalLink className="size-3" />
        </a>
      </SettingsRow>
      <SettingsRow label="Source">
        <a
          href="https://github.com/madvib/Ship"
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-1 text-[11px] text-primary transition hover:underline"
        >
          GitHub
          <ExternalLink className="size-3" />
        </a>
      </SettingsRow>
    </SettingsSection>
  )
}
