import { createFileRoute, Link } from '@tanstack/react-router'
import { useState, useCallback } from 'react'
import { ArrowLeft } from 'lucide-react'
import { authClient } from '#/lib/auth-client'
import { loadSettings, saveSettings } from '#/features/settings/settingsData'
import type { SettingsData } from '#/features/settings/settingsData'
import {
  AccountSection, GitHubSection, GlobalDefaultsSection,
  GlobalHooksSection, EnvVarsSection, CLISection, DangerZoneSection,
} from '#/features/settings/sections'

export const Route = createFileRoute('/settings')({ component: SettingsPage })

function SettingsPage() {
  const { data: session, isPending } = authClient.useSession()
  const [settings, setSettings] = useState<SettingsData>(loadSettings)

  const update = useCallback(
    (patch: Partial<SettingsData>) => {
      setSettings((prev) => {
        const next = { ...prev, ...patch }
        saveSettings(next)
        return next
      })
    },
    [],
  )

  const user = session?.user

  return (
    <div className="mx-auto max-w-[680px] px-5 py-6 pb-24">
      <div className="mb-2">
        <Link
          to="/"
          className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition mb-3"
        >
          <ArrowLeft className="size-3" />
          Back
        </Link>
      </div>

      <div className="mb-6">
        <h1 className="font-display text-xl font-bold text-foreground">
          Settings
        </h1>
        <p className="text-[13px] text-muted-foreground">
          Account, global defaults, and integrations
        </p>
      </div>

      <AccountSection user={user} isPending={isPending} />
      <GitHubSection settings={settings} update={update} />
      <GlobalDefaultsSection settings={settings} update={update} />
      <GlobalHooksSection settings={settings} update={update} />
      <EnvVarsSection settings={settings} update={update} />
      <CLISection />
      <DangerZoneSection />
    </div>
  )
}
