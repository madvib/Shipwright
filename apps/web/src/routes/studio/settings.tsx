import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback } from 'react'
import { authClient } from '#/lib/auth-client'
import { loadSettings, saveSettings } from '#/features/settings/settingsData'
import type { SettingsData } from '#/features/settings/settingsData'
import {
  AccountSection, GitHubSection, GlobalDefaultsSection,
  GlobalHooksSection, EnvVarsSection, CLISection, DangerZoneSection,
} from '#/features/settings/sections'

import { SettingsSkeleton } from '#/features/studio/StudioSkeleton'

export const Route = createFileRoute('/studio/settings')({
  component: SettingsPage,
  pendingComponent: SettingsSkeleton,
  ssr: false,
})

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
