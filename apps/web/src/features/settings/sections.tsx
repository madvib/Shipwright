import {
  Terminal, Paintbrush, Info,
  ExternalLink,
} from 'lucide-react'
import { ThemeToggle } from '@ship/primitives'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { SettingsSection, SettingsRow } from './SettingsLayout'

// ── Daemon Connection ───────────────────────────────────────────────────────

export function CLIConnectionSection() {
  const { connected, workspaces, error } = useDaemon()
  const activeWorkspace = workspaces.find((w) => w.status === 'active')

  return (
    <SettingsSection icon={<Terminal className="size-[15px]" />} title="Daemon Connection">
      <SettingsRow label="Status">
        <div className="flex items-center gap-1.5">
          <span className={`size-2 rounded-full ${connected ? 'bg-emerald-500' : 'bg-muted-foreground'}`} />
          <span className="text-[11px] text-muted-foreground">{connected ? 'Connected' : 'Disconnected'}</span>
        </div>
      </SettingsRow>

      <SettingsRow label="Endpoint">
        <span className="text-[11px] text-muted-foreground font-mono">{DAEMON_BASE_URL}</span>
      </SettingsRow>

      {activeWorkspace && (
        <SettingsRow label="Workspace">
          <span className="text-[11px] text-muted-foreground">{activeWorkspace.branch}</span>
        </SettingsRow>
      )}

      {error && (
        <SettingsRow label="Error">
          <span className="max-w-[300px] text-right text-[11px] text-destructive">{error.message}</span>
        </SettingsRow>
      )}
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
