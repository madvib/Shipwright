import { TechIcon } from './TechIcon'
import type { Profile } from './useProfiles'

interface ProfileListProps {
  profiles: Profile[]
  activeId: string | null
  onSelect: (id: string) => void
  onNew: () => void
}

export function ProfileList({ profiles, activeId, onSelect, onNew }: ProfileListProps) {
  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="px-4 h-11 border-b border-border/60 bg-card/30 flex items-center justify-between shrink-0">
        <span className="text-sm font-semibold text-foreground">Profiles</span>
        <button
          onClick={onNew}
          className="h-7 px-2.5 rounded bg-primary text-[11px] font-medium text-primary-foreground transition hover:bg-primary/90 inline-flex items-center gap-1"
        >
          + New profile
        </button>
      </div>

      {/* Grid */}
      <div className="flex-1 overflow-auto p-4">
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
          {profiles.map((profile) => (
            <ProfileCard
              key={profile.id}
              profile={profile}
              active={profile.id === activeId}
              onClick={() => onSelect(profile.id)}
            />
          ))}

          {/* New profile dashed card */}
          <button
            onClick={onNew}
            className="rounded-lg border-2 border-dashed border-border/50 hover:border-border bg-card/30 flex flex-col items-center justify-center cursor-pointer min-h-[130px] gap-2 transition-colors"
          >
            <div className="size-9 rounded-lg border border-dashed border-border/60 bg-muted/40 flex items-center justify-center">
              <span className="text-lg text-muted-foreground/40">+</span>
            </div>
            <span className="text-[10px] text-muted-foreground/40">New profile</span>
          </button>
        </div>
      </div>
    </div>
  )
}

function ProfileCard({ profile, active, onClick }: { profile: Profile; active: boolean; onClick: () => void }) {
  const accent = profile.accentColor
  const providerLabel = profile.selectedProviders[0] ?? '—'

  return (
    <button
      onClick={onClick}
      className="rounded-lg border bg-card p-3.5 cursor-pointer text-left transition-colors"
      style={{
        borderColor: active ? accent + '55' : accent + '22',
      }}
      onMouseEnter={(e) => { e.currentTarget.style.borderColor = accent + '55' }}
      onMouseLeave={(e) => { e.currentTarget.style.borderColor = active ? accent + '55' : accent + '22' }}
    >
      <TechIcon stack={profile.icon} size={36} style={{ marginBottom: 10, borderRadius: 8 }} />

      <div className="text-xs font-semibold text-foreground mb-0.5">
        {profile.name || 'Untitled'}
      </div>

      <div className="text-[10px] text-muted-foreground mb-2">
        {providerLabel}
      </div>

      <div className="flex flex-wrap gap-1">
        {profile.skills.length > 0 && (
          <Chip>{profile.skills.length} skill{profile.skills.length !== 1 ? 's' : ''}</Chip>
        )}
        {profile.mcpServers.length > 0 && (
          <Chip>{profile.mcpServers.length} MCP</Chip>
        )}
        {profile.skills.length === 0 && profile.mcpServers.length === 0 && (
          <Chip>empty</Chip>
        )}
      </div>
    </button>
  )
}

function Chip({ children }: { children: React.ReactNode }) {
  return (
    <span className="text-[8px] bg-muted rounded px-1.5 py-0.5 text-muted-foreground/60 leading-tight">
      {children}
    </span>
  )
}
