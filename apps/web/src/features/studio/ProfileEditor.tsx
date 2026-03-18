import { useState, useRef, useEffect } from 'react'
import { ChevronLeft, Trash2 } from 'lucide-react'
import { toast } from 'sonner'
import { OverviewTab, ProvidersTab } from './ProfileEditorFields'
import { PermissionsEditor } from './PermissionsEditor'
import type { Profile } from './useProfiles'
import type { Permissions } from '#/features/compiler/types'

type Tab = 'overview' | 'providers' | 'permissions'

interface ProfileEditorProps {
  profile: Profile
  onChange: (patch: Partial<Profile>) => void
  onBack: () => void
  onDelete?: () => void
}

export function ProfileEditor({ profile, onChange, onBack, onDelete }: ProfileEditorProps) {
  const [tab, setTab] = useState<Tab>('overview')

  // Debounced save toast: fires 1.5s after last edit, skips initial render
  const mountedRef = useRef(false)
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    if (!mountedRef.current) {
      mountedRef.current = true
      return
    }
    if (toastTimer.current) clearTimeout(toastTimer.current)
    toastTimer.current = setTimeout(() => {
      toast.success('Profile updated')
    }, 1500)
    return () => {
      if (toastTimer.current) clearTimeout(toastTimer.current)
    }
  }, [profile])

  return (
    <div className="flex flex-col h-full">
      {/* Compact breadcrumb + tabs in one row */}
      <div className="flex items-center gap-3 border-b border-border/60 px-5 shrink-0">
        <button
          onClick={onBack}
          className="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors py-2.5"
        >
          <ChevronLeft className="size-3.5" />
          Profiles
        </button>
        <span className="text-muted-foreground/20">/</span>
        <span className="text-xs font-semibold text-foreground">{profile.name || 'Untitled'}</span>
        <span
          className="rounded px-1.5 py-px text-[10px] font-bold"
          style={{ background: profile.accentColor + '20', color: profile.accentColor }}
        >
          live
        </span>

        {/* Delete + Tabs inline */}
        <div className="flex ml-auto items-center gap-2 -mb-px">
        {onDelete && (
          <button
            onClick={onDelete}
            className="flex items-center gap-1 rounded-md px-2 py-1 text-[10px] text-muted-foreground/60 transition hover:bg-destructive/10 hover:text-destructive"
            title="Delete profile"
            aria-label="Delete profile"
          >
            <Trash2 className="size-3" />
          </button>
        )}
        <div className="flex" role="tablist">
          {(['overview', 'providers', 'permissions'] as Tab[]).map((t) => {
            const active = tab === t
            const label = t === 'overview' ? 'Overview' : t === 'providers' ? 'Providers' : 'Permissions'
            return (
              <button
                key={t}
                role="tab"
                aria-selected={active}
                onClick={() => setTab(t)}
                className={`px-3 py-2.5 text-[11px] font-medium border-b-2 transition-colors ${
                  active
                    ? 'border-violet-500 text-foreground'
                    : 'border-transparent text-muted-foreground hover:text-foreground'
                }`}
              >
                {label}
              </button>
            )
          })}
        </div>
        </div>
      </div>

      {/* Body */}
      <div className="flex-1 overflow-auto p-5">
        {tab === 'overview' && <OverviewTab profile={profile} onChange={onChange} />}
        {tab === 'providers' && <ProvidersTab profile={profile} onChange={onChange} />}
        {tab === 'permissions' && (
          <PermissionsEditor
            permissions={profile.permissions}
            onChange={(permissions: Permissions) => onChange({ permissions })}
          />
        )}
      </div>
    </div>
  )
}
