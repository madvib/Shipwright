import { Terminal } from 'lucide-react'
import { SettingsSection, SettingsRow } from './SettingsLayout'

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
