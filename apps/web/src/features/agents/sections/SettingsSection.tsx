import { Settings, ChevronDown } from 'lucide-react'
import type { AgentSettings } from '../types'
import { SectionShell, OrangeDot } from './SectionShell'

interface SettingsSectionProps {
  settings: AgentSettings
  onUpdate: (patch: Partial<AgentSettings>) => void
}

export function SettingsSection({ settings, onUpdate }: SettingsSectionProps) {
  return (
    <SectionShell
      icon={<Settings className="size-4" />}
      title="Settings"
      count="(inherits from global defaults)"
    >
      <div className="grid grid-cols-2 gap-2">
        {/* Model */}
        <div className="flex items-center justify-between rounded-lg border border-border/40 bg-card/30 px-3 py-2.5">
          <span className="text-xs text-foreground/80">Model</span>
          <span className="flex items-center gap-1 text-xs text-muted-foreground">
            {settings.model}
            <ChevronDown className="size-2.5" />
            <OrangeDot />
          </span>
        </div>

        {/* Default mode */}
        <div className="flex items-center justify-between rounded-lg border border-border/40 bg-card/30 px-3 py-2.5">
          <span className="text-xs text-foreground/80">Default mode</span>
          <span className="flex items-center gap-1 text-xs text-muted-foreground">
            {settings.defaultMode}
            <ChevronDown className="size-2.5" />
            <OrangeDot />
          </span>
        </div>

        {/* Extended thinking toggle */}
        <div className="flex items-center justify-between rounded-lg border border-border/40 bg-card/30 px-3 py-2.5">
          <span className="text-xs text-foreground/80">Extended thinking</span>
          <button
            onClick={() => onUpdate({ extendedThinking: !settings.extendedThinking })}
            className={`relative h-4 w-8 shrink-0 rounded-full transition-colors ${
              settings.extendedThinking ? 'bg-primary' : 'bg-muted'
            }`}
          >
            <span
              className={`absolute top-[2px] size-3 rounded-full bg-white transition-all ${
                settings.extendedThinking ? 'left-[18px]' : 'left-[2px]'
              }`}
            />
          </button>
        </div>

        {/* Auto memory toggle */}
        <div className="flex items-center justify-between rounded-lg border border-border/40 bg-card/30 px-3 py-2.5">
          <span className="text-xs text-foreground/80">Auto memory</span>
          <button
            onClick={() => onUpdate({ autoMemory: !settings.autoMemory })}
            className={`relative h-4 w-8 shrink-0 rounded-full transition-colors ${
              settings.autoMemory ? 'bg-primary' : 'bg-muted'
            }`}
          >
            <span
              className={`absolute top-[2px] size-3 rounded-full bg-white transition-all ${
                settings.autoMemory ? 'left-[18px]' : 'left-[2px]'
              }`}
            />
          </button>
        </div>
      </div>
    </SectionShell>
  )
}
