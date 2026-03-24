import { createFileRoute } from '@tanstack/react-router'
import { CLISection, DangerZoneSection } from '#/features/settings/sections'
import { SettingsSkeleton } from '#/features/studio/StudioSkeleton'

export const Route = createFileRoute('/studio/settings')({
  component: SettingsPage,
  pendingComponent: SettingsSkeleton,
  ssr: false,
})

function SettingsPage() {
  return (
    <div className="mx-auto max-w-[680px] px-5 py-6 pb-24">
      <div className="mb-6">
        <h1 className="font-display text-xl font-bold text-foreground">Settings</h1>
        <p className="text-[13px] text-muted-foreground">CLI and project configuration</p>
      </div>

      <CLISection />
      <DangerZoneSection />
    </div>
  )
}
