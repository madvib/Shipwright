import { useMemo } from 'react'
import { Lock, Wrench, FileText, Terminal, Globe } from 'lucide-react'
import type { Permissions } from '@ship/ui'
import { SectionShell } from './SectionShell'
import { getFieldEnum } from '#/features/agents/schema-hints'

interface PermissionsSectionProps {
  permissions: Permissions
  activePreset: string
  onPresetChange: (preset: string) => void
  onEdit?: () => void
}

export function PermissionsSection({
  permissions,
  activePreset,
  onPresetChange,
  onEdit,
}: PermissionsSectionProps) {
  const schemaPresets = useMemo(() => getFieldEnum('permissions.preset'), [])

  return (
    <SectionShell
      icon={<Lock className="size-4" />}
      title="Permissions"
      actionLabel="Edit"
      onAction={onEdit}
    >
      {/* Preset bar */}
      <div className="flex gap-1 mb-3">
        {schemaPresets.map((preset) => (
          <button
            key={preset}
            onClick={() => onPresetChange(preset)}
            className={`flex-1 rounded-md border px-2 py-1.5 text-[11px] text-center transition-colors ${
              activePreset === preset
                ? 'border-primary text-primary bg-primary/5'
                : 'border-border/40 text-muted-foreground/50 bg-card/30 hover:border-border hover:text-muted-foreground'
            }`}
          >
            {preset}
          </button>
        ))}
      </div>

      {/* Permission cards */}
      <div className="grid grid-cols-2 gap-2">
        <PermCard
          icon={<Wrench className="size-3" />}
          label="Tools"
          allow={permissions.tools?.allow ?? []}
          deny={permissions.tools?.deny ?? []}
        />
        <PermCard
          icon={<FileText className="size-3" />}
          label="Filesystem"
          allow={permissions.filesystem?.allow ?? []}
          deny={permissions.filesystem?.deny ?? []}
        />
        <PermCard
          icon={<Terminal className="size-3" />}
          label="Commands"
          allow={permissions.commands?.allow ?? []}
          deny={permissions.commands?.deny ?? []}
        />
        <PermCard
          icon={<Globe className="size-3" />}
          label="Network"
          allow={permissions.network?.allow_hosts ?? []}
          deny={[]}
        />
      </div>
    </SectionShell>
  )
}

function PermCard({
  icon,
  label,
  allow,
  deny,
}: {
  icon: React.ReactNode
  label: string
  allow: string[]
  deny: string[]
}) {
  return (
    <div className="rounded-lg border border-border/40 bg-card/30 px-3 py-2.5">
      <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground/60 mb-1.5">
        {icon}
        {label}
      </div>
      {allow.length > 0 && (
        <div className="text-[11px] text-emerald-600 dark:text-emerald-400 truncate">
          {allow.join(', ')}
        </div>
      )}
      {deny.length > 0 && (
        <div className="mt-0.5 text-[11px] text-destructive truncate">
          deny: {deny.join(', ')}
        </div>
      )}
      {allow.length === 0 && deny.length === 0 && (
        <div className="text-[10px] text-muted-foreground/30 italic">no rules</div>
      )}
    </div>
  )
}
