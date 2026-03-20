import { User, Plus } from 'lucide-react'
import { TechIcon } from './TechIcon'
// EmptyState removed
import type { Profile } from './useProfiles'

interface ProfileListProps {
  profiles: Profile[]
  activeId: string | null
  onSelect: (id: string) => void
  onNew: () => void
  onDelete?: (id: string) => void
}

export function ProfileList({ profiles, activeId, onSelect, onNew, onDelete }: ProfileListProps) {
  if (profiles.length === 0) {
    return (
      <div className="flex flex-col items-center py-10 text-center">
        <User className="size-5 text-muted-foreground mb-3" />
        <p className="text-sm font-medium text-foreground mb-1">No profiles yet</p>
        <p className="text-xs text-muted-foreground mb-4">Profiles define your agent configuration.</p>
        <button
          onClick={onNew}
          className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:opacity-90"
        >
          Create your first profile
        </button>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {/* Grid */}
      <div className="flex-1 overflow-auto p-5">
        <div className="flex items-center justify-between mb-4">
          <span className="text-xs font-semibold uppercase tracking-widest text-muted-foreground/60">
            {profiles.length} profile{profiles.length !== 1 ? 's' : ''}
          </span>
          <button
            onClick={onNew}
            className="h-7 px-2.5 rounded bg-primary text-[11px] font-medium text-primary-foreground transition-colors hover:bg-primary/90 inline-flex items-center gap-1"
          >
            <Plus className="size-3" />
            New profile
          </button>
        </div>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
          {profiles.map((profile) => (
            <ProfileCard
              key={profile.id}
              profile={profile}
              active={profile.id === activeId}
              onClick={() => onSelect(profile.id)}
              onDelete={onDelete ? () => onDelete(profile.id) : undefined}
            />
          ))}

          {/* New profile dashed card */}
          <button
            onClick={onNew}
            className="rounded-lg border-2 border-dashed border-border/50 hover:border-border bg-card/30 flex flex-col items-center justify-center cursor-pointer min-h-[130px] gap-2 transition-colors"
          >
            <div className="size-9 rounded-lg border border-dashed border-border/60 bg-muted/40 flex items-center justify-center">
              <Plus className="size-4 text-muted-foreground/40" />
            </div>
            <span className="text-[10px] text-muted-foreground/40">New profile</span>
          </button>
        </div>
      </div>
    </div>
  )
}

function ProfileCard({ profile, active, onClick, onDelete }: { profile: Profile; active: boolean; onClick: () => void; onDelete?: () => void }) {
  const providerLabel = profile.selectedProviders[0] ?? '---'

  return (
    <div
      data-active={active || undefined}
      className={`group relative rounded-lg border bg-card p-3.5 cursor-pointer text-left transition-all duration-150 hover:border-border ${
        active
          ? 'ring-2 ring-primary/40 border-primary/40'
          : 'border-border/30 hover:shadow-sm'
      }`}
      onClick={onClick}
    >
      {onDelete && (
        <button
          onClick={(e) => { e.stopPropagation(); onDelete() }}
          className="absolute top-2 right-2 hidden size-5 items-center justify-center rounded text-muted-foreground/40 transition hover:bg-destructive/10 hover:text-destructive group-hover:flex"
          title="Delete profile"
          aria-label="Delete profile"
        >
          <span className="text-xs leading-none">&times;</span>
        </button>
      )}

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
    </div>
  )
}

function Chip({ children }: { children: React.ReactNode }) {
  return (
    <span className="text-[10px] bg-muted rounded px-1.5 py-0.5 text-muted-foreground/60 leading-tight">
      {children}
    </span>
  )
}
