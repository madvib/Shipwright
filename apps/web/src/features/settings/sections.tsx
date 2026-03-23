import { useState } from 'react'
import { User, Github, Terminal } from 'lucide-react'
import { Button, Badge } from '@ship/primitives'
import { toast } from 'sonner'
import { authClient } from '#/lib/auth-client'
import { SettingsSection, SettingsRow } from './SettingsLayout'

// ── Account ──────────────────────────────────────────────────────────────────

export function AccountSection({
  user,
  isPending,
}: {
  user:
    | { name?: string | null; email?: string | null; image?: string | null }
    | undefined
  isPending: boolean
}) {
  return (
    <SettingsSection
      icon={<User className="size-[15px]" />}
      title="Account"
      action={
        user ? (
          <Button variant="ghost" size="xs" onClick={() => void authClient.signOut()}>
            Sign out
          </Button>
        ) : null
      }
    >
      {isPending ? (
        <div className="h-14 animate-pulse rounded-lg bg-muted/40" />
      ) : user ? (
        <div className="flex items-center gap-3.5">
          {user.image ? (
            <img src={user.image} alt="" className="size-12 rounded-xl object-cover" />
          ) : (
            <div className="flex size-12 items-center justify-center rounded-xl bg-gradient-to-br from-primary to-primary/70 text-xl font-bold text-primary-foreground">
              {user.name?.charAt(0).toUpperCase() || 'U'}
            </div>
          )}
          <div className="flex-1">
            <div className="text-[15px] font-semibold text-foreground">{user.name || 'Unnamed'}</div>
            <div className="text-xs text-muted-foreground">{user.email || ''}</div>
            <Badge variant="default" className="mt-1">Pro</Badge>
          </div>
        </div>
      ) : (
        <div className="flex items-center justify-between">
          <p className="text-xs text-muted-foreground">Not signed in</p>
          <Button size="sm" onClick={() => void authClient.signIn.social({ provider: 'github' })}>
            Sign in
          </Button>
        </div>
      )}
    </SettingsSection>
  )
}

// ── GitHub ────────────────────────────────────────────────────────────────────

export function GitHubSection() {
  const { data: session } = authClient.useSession()
  const [disconnected, setDisconnected] = useState(false)
  const isConnected = !!session?.user && !disconnected
  const disconnect = () => fetch('/api/github/disconnect', { method: 'POST' })
    .then((r) => { if (!r.ok) throw r; setDisconnected(true); toast.success('GitHub disconnected') })
    .catch(() => toast.error('Failed to disconnect GitHub'))

  return (
    <SettingsSection icon={<Github className="size-[15px]" />} title="GitHub"
      action={isConnected ? <Button variant="ghost" size="xs" onClick={() => void disconnect()}>Disconnect</Button> : null}
    >
      {isConnected ? (
        <div className="flex items-center gap-3">
          <div className="flex size-9 items-center justify-center rounded-lg bg-muted/60">
            <Github className="size-[18px] text-foreground" />
          </div>
          <div className="flex-1">
            <div className="text-[13px] font-medium text-foreground">{session?.user?.name || 'Connected'}</div>
            <div className="flex items-center gap-1 text-[10px] text-emerald-600 dark:text-emerald-400">
              <span className="size-1.5 rounded-full bg-emerald-500" />
              Connected
            </div>
          </div>
        </div>
      ) : (
        <div className="flex items-center justify-between">
          <p className="text-xs text-muted-foreground">Connect GitHub to import repos and sync configs</p>
          <Button size="sm" variant="outline" onClick={() => void authClient.signIn.social({ provider: 'github' })}>
            <Github className="size-3.5" />
            Connect
          </Button>
        </div>
      )}
    </SettingsSection>
  )
}

// ── CLI ──────────────────────────────────────────────────────────────────────

export function CLISection() {
  return (
    <SettingsSection icon={<Terminal className="size-[15px]" />} title="CLI">
      <SettingsRow label="Install command">
        <code className="font-mono text-[11px] text-muted-foreground">curl -fsSL https://getship.dev/install | sh</code>
      </SettingsRow>
    </SettingsSection>
  )
}

// ── Danger zone ──────────────────────────────────────────────────────────────

export { DangerZoneSection } from './DangerZoneSection'
