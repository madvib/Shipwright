import {
  User, Github, Settings as SettingsIcon, Link2, Terminal,
  AlertTriangle, Plus,
} from 'lucide-react'
import { Button, Badge, Switch, Separator } from '@ship/primitives'
import { authClient } from '#/lib/auth-client'
import type { SettingsData } from './settingsData'
import {
  OrangeDot, SettingsSection, SettingsRow,
  SettingsSelect, EnvVarRow, HookRow,
} from './SettingsLayout'

type UpdateFn = (patch: Partial<SettingsData>) => void

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

export function GitHubSection({
  settings,
  update,
}: {
  settings: SettingsData
  update: UpdateFn
}) {
  const { data: session } = authClient.useSession()
  const isConnected = !!session?.user

  return (
    <SettingsSection
      icon={<Github className="size-[15px]" />}
      title="GitHub"
      action={
        isConnected ? (
          <Button variant="ghost" size="xs">Disconnect <OrangeDot /></Button>
        ) : null
      }
    >
      {isConnected ? (
        <>
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
          <Separator className="my-3" />
          <SettingsRow label="Auto-import on push" sublabel="Sync agent configs when you push to connected repos">
            <div className="flex items-center gap-2">
              <OrangeDot />
              <Switch checked={settings.autoImport} onCheckedChange={(checked: boolean) => update({ autoImport: checked })} />
            </div>
          </SettingsRow>
          <SettingsRow label="Create PR on import" sublabel="Submit a PR adding .ship/ config instead of direct commit">
            <div className="flex items-center gap-2">
              <OrangeDot />
              <Switch checked={settings.createPr} onCheckedChange={(checked: boolean) => update({ createPr: checked })} />
            </div>
          </SettingsRow>
        </>
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

// ── Global defaults ──────────────────────────────────────────────────────────

export function GlobalDefaultsSection({
  settings,
  update,
}: {
  settings: SettingsData
  update: UpdateFn
}) {
  return (
    <SettingsSection icon={<SettingsIcon className="size-[15px]" />} title="Global Defaults">
      <p className="mb-2.5 text-[10px] text-muted-foreground">
        These apply to all agents unless overridden per-agent.
      </p>
      <SettingsRow label="Default provider">
        <div className="flex items-center gap-2"><OrangeDot />
          <SettingsSelect value={settings.defaultProvider} onChange={(v) => update({ defaultProvider: v })}
            options={[
              { value: 'claude', label: 'Claude Code' },
              { value: 'gemini', label: 'Gemini CLI' },
              { value: 'codex', label: 'Codex' },
              { value: 'cursor', label: 'Cursor' },
            ]}
          />
        </div>
      </SettingsRow>
      <SettingsRow label="Model">
        <div className="flex items-center gap-2"><OrangeDot />
          <SettingsSelect value={settings.defaultModel} onChange={(v) => update({ defaultModel: v })}
            options={[
              { value: 'claude-sonnet-4-6', label: 'claude-sonnet-4-6' },
              { value: 'claude-opus-4-6', label: 'claude-opus-4-6' },
              { value: 'gemini-2.5-pro', label: 'gemini-2.5-pro' },
              { value: 'gpt-4.1', label: 'gpt-4.1' },
            ]}
          />
        </div>
      </SettingsRow>
      <SettingsRow label="Default mode">
        <div className="flex items-center gap-2"><OrangeDot />
          <SettingsSelect value={settings.defaultMode} onChange={(v) => update({ defaultMode: v })}
            options={[
              { value: 'default', label: 'default' },
              { value: 'plan', label: 'plan' },
              { value: 'review', label: 'review' },
            ]}
          />
        </div>
      </SettingsRow>
      <SettingsRow label="Extended thinking">
        <div className="flex items-center gap-2"><OrangeDot />
          <Switch checked={settings.extendedThinking} onCheckedChange={(c: boolean) => update({ extendedThinking: c })} />
        </div>
      </SettingsRow>
      <SettingsRow label="Auto memory">
        <div className="flex items-center gap-2"><OrangeDot />
          <Switch checked={settings.autoMemory} onCheckedChange={(c: boolean) => update({ autoMemory: c })} />
        </div>
      </SettingsRow>
      <SettingsRow label="Permission preset">
        <div className="flex items-center gap-2"><OrangeDot />
          <SettingsSelect value={settings.permissionPreset} onChange={(v) => update({ permissionPreset: v })}
            options={[
              { value: 'ship-guarded', label: 'ship-guarded' },
              { value: 'permissive', label: 'permissive' },
              { value: 'strict', label: 'strict' },
            ]}
          />
        </div>
      </SettingsRow>
    </SettingsSection>
  )
}

// ── Hooks ─────────────────────────────────────────────────────────────────────

export function GlobalHooksSection({
  settings,
  update,
}: {
  settings: SettingsData
  update: UpdateFn
}) {
  return (
    <SettingsSection
      icon={<Link2 className="size-[15px]" />}
      title="Global Hooks"
      action={
        <Button variant="ghost" size="xs" onClick={() => update({ hooks: [...settings.hooks, { trigger: 'Stop', command: '' }] })}>
          <Plus className="size-3" /> Add <OrangeDot />
        </Button>
      }
    >
      <p className="mb-2 text-[10px] text-muted-foreground">Applied to all agents. Override per-agent in agent settings.</p>
      {settings.hooks.map((hook, i) => (
        <HookRow key={i} trigger={hook.trigger} command={hook.command}
          onRemove={() => update({ hooks: settings.hooks.filter((_, j) => j !== i) })}
        />
      ))}
    </SettingsSection>
  )
}

// ── Env vars ─────────────────────────────────────────────────────────────────

export function EnvVarsSection({
  settings,
  update,
}: {
  settings: SettingsData
  update: UpdateFn
}) {
  const updateVar = (i: number, field: 'key' | 'value', val: string) => {
    update({ envVars: settings.envVars.map((v, j) => j === i ? { ...v, [field]: val } : v) })
  }

  return (
    <SettingsSection
      icon={
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} className="size-[15px]">
          <rect x="3" y="3" width="18" height="18" rx="2" /><path d="M3 9h18M9 21V9" />
        </svg>
      }
      title="Environment Variables"
      action={
        <Button variant="ghost" size="xs" onClick={() => update({ envVars: [...settings.envVars, { key: '', value: '' }] })}>
          <Plus className="size-3" /> Add
        </Button>
      }
    >
      {settings.envVars.map((env, i) => (
        <EnvVarRow key={i} envKey={env.key} envValue={env.value}
          onKeyChange={(v) => updateVar(i, 'key', v)} onValueChange={(v) => updateVar(i, 'value', v)}
          onRemove={() => update({ envVars: settings.envVars.filter((_, j) => j !== i) })}
        />
      ))}
    </SettingsSection>
  )
}

// ── CLI ──────────────────────────────────────────────────────────────────────

export function CLISection() {
  return (
    <SettingsSection icon={<Terminal className="size-[15px]" />} title="CLI">
      <SettingsRow label="Installed version">
        <code className="font-mono text-[11px] text-emerald-600 dark:text-emerald-400">v0.1.0</code>
      </SettingsRow>
      <SettingsRow label="Install command">
        <code className="font-mono text-[11px] text-muted-foreground">curl -fsSL https://getship.dev/install | sh</code>
      </SettingsRow>
    </SettingsSection>
  )
}

// ── Danger zone ──────────────────────────────────────────────────────────────

export function DangerZoneSection() {
  return (
    <SettingsSection icon={<AlertTriangle className="size-[15px]" />} title="Danger Zone" danger>
      <SettingsRow label="Delete all agents" sublabel="Permanently remove all agent configurations">
        <Button variant="destructive" size="xs">Delete all <OrangeDot /></Button>
      </SettingsRow>
      <SettingsRow label="Delete account" sublabel="Remove your account and all data from Ship">
        <Button variant="destructive" size="xs">Delete account <OrangeDot /></Button>
      </SettingsRow>
    </SettingsSection>
  )
}
