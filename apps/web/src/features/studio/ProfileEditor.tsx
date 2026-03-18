import { useState, useRef, useEffect } from 'react'
import { Link } from '@tanstack/react-router'
import { ChevronLeft } from 'lucide-react'
import { IconPicker } from './IconPicker'
import { PermissionsEditor } from './PermissionsEditor'
import { PROVIDERS } from '#/features/compiler/types'
import type { Profile } from './useProfiles'
import type { Permissions } from '#/features/compiler/types'

type Tab = 'overview' | 'providers' | 'permissions'

interface ProfileEditorProps {
  profile: Profile
  onChange: (patch: Partial<Profile>) => void
  onBack: () => void
}

export function ProfileEditor({ profile, onChange, onBack }: ProfileEditorProps) {
  const [tab, setTab] = useState<Tab>('overview')

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
          className="rounded px-1.5 py-px text-[8px] font-bold"
          style={{ background: profile.accentColor + '20', color: profile.accentColor }}
        >
          live
        </span>

        {/* Tabs inline */}
        <div className="flex ml-auto -mb-px">
          {(['overview', 'providers', 'permissions'] as Tab[]).map((t) => {
            const active = tab === t
            const label = t === 'overview' ? 'Overview' : t === 'providers' ? 'Providers' : 'Permissions'
            return (
              <button
                key={t}
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

// ── Section label ──────────────────────────────────────────────────────────────

function SectionLabel({ children }: { children: React.ReactNode }) {
  return (
    <div className="text-[9px] font-semibold uppercase tracking-widest text-muted-foreground mb-1.5">
      {children}
    </div>
  )
}

// ── Overview ───────────────────────────────────────────────────────────────────

function OverviewTab({ profile, onChange }: { profile: Profile; onChange: (p: Partial<Profile>) => void }) {
  return (
    <div className="space-y-4 max-w-xl">

      {/* Name + icon row */}
      <div className="flex items-center gap-3">
        <IconPicker
          icon={profile.icon}
          accentColor={profile.accentColor}
          name={profile.name}
          onChange={(icon, accentColor) => onChange({ icon, accentColor })}
        />
        <input
          value={profile.name}
          onChange={(e) => onChange({ name: e.target.value })}
          placeholder="Profile name"
          spellCheck={false}
          className="flex-1 bg-muted/30 border border-border/60 rounded-md px-3 py-1.5 text-sm font-semibold text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus:border-violet-500/50 transition-colors"
        />
      </div>

      {/* Persona */}
      <div>
        <SectionLabel>PERSONA</SectionLabel>
        <textarea
          value={profile.persona}
          onChange={(e) => onChange({ persona: e.target.value })}
          rows={2}
          placeholder="React + TailwindCSS frontend specialist. Prefer composition, keep components under 200 lines, strict TypeScript."
          spellCheck={false}
          className="w-full bg-muted/30 border border-border/60 rounded-md px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground/40 leading-relaxed resize-none focus:outline-none focus:border-violet-500/50 transition-colors"
        />
      </div>

      {/* Rules */}
      <div>
        <SectionLabel>
          RULES <span className="text-muted-foreground/40 font-normal normal-case tracking-normal">— always loaded, not progressive</span>
        </SectionLabel>
        <RulesBlock rules={profile.rules} onChange={(rules) => onChange({ rules })} />
      </div>

      {/* Skills */}
      <div>
        <div className="flex items-center justify-between mb-1.5">
          <SectionLabel>
            SKILLS <span className="text-muted-foreground/40 font-normal normal-case tracking-normal">· {profile.skills.length} · progressive context</span>
          </SectionLabel>
          <Link
            to="/studio/skills"
            className="inline-flex items-center rounded bg-violet-600 hover:bg-violet-500 transition-colors px-2 py-0.5 text-[9px] text-white no-underline"
          >
            + Add
          </Link>
        </div>
        <div className="flex flex-wrap gap-1.5">
          {profile.skills.length === 0 ? (
            <span className="text-xs text-muted-foreground/40 italic">No skills attached</span>
          ) : (
            profile.skills.map((skill) => (
              <Chip
                key={skill.id}
                dotClass="bg-emerald-500"
                onRemove={() => onChange({ skills: profile.skills.filter((s) => s.id !== skill.id) })}
              >
                {skill.name}
              </Chip>
            ))
          )}
        </div>
      </div>

      {/* MCP Servers */}
      <div>
        <div className="flex items-center justify-between mb-1.5">
          <SectionLabel>
            MCP SERVERS <span className="text-muted-foreground/40 font-normal normal-case tracking-normal">· {profile.mcpServers.length}</span>
          </SectionLabel>
          <Link
            to="/studio/mcp"
            className="inline-flex items-center rounded bg-violet-600 hover:bg-violet-500 transition-colors px-2 py-0.5 text-[9px] text-white no-underline"
          >
            + Add
          </Link>
        </div>
        <div className="flex flex-wrap gap-1.5">
          {profile.mcpServers.length === 0 ? (
            <span className="text-xs text-muted-foreground/40 italic">No MCP servers attached</span>
          ) : (
            profile.mcpServers.map((server) => (
              <Chip
                key={server.name}
                dotClass="bg-sky-400"
                onRemove={() => onChange({ mcpServers: profile.mcpServers.filter((s) => s.name !== server.name) })}
              >
                {server.name}
              </Chip>
            ))
          )}
        </div>
      </div>

      {/* Default provider */}
      <div>
        <SectionLabel>
          DEFAULT PROVIDER <span className="text-muted-foreground/40 font-normal normal-case tracking-normal">— per-provider config → Providers tab</span>
        </SectionLabel>
        <div className="flex gap-1.5 flex-wrap">
          {PROVIDERS.map((p) => {
            const active = profile.selectedProviders.includes(p.id)
            const toggle = () => {
              const next = active
                ? profile.selectedProviders.filter((id) => id !== p.id)
                : [...profile.selectedProviders, p.id]
              onChange({ selectedProviders: next })
            }
            return (
              <button
                key={p.id}
                onClick={toggle}
                className={`inline-flex items-center gap-1.5 h-7 px-2.5 rounded-md border text-[10px] transition-colors ${
                  active
                    ? 'bg-violet-500/10 border-violet-500/30 text-violet-400'
                    : 'bg-muted/30 border-border/60 text-muted-foreground hover:border-border hover:text-foreground'
                }`}
              >
                <span className={`size-1.5 rounded-full ${active ? 'bg-violet-500' : 'bg-muted-foreground/30'}`} />
                {p.name}{active ? ' ✓' : ''}
              </button>
            )
          })}
        </div>
      </div>

    </div>
  )
}

// ── Inline rules block ─────────────────────────────────────────────────────────

function RulesBlock({ rules, onChange }: { rules: string[]; onChange: (r: string[]) => void }) {
  const [editIdx, setEditIdx] = useState<number | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (editIdx !== null) inputRef.current?.focus()
  }, [editIdx])

  const add = () => {
    const next = [...rules, '']
    onChange(next)
    setEditIdx(next.length - 1)
  }

  const update = (i: number, val: string) => onChange(rules.map((r, j) => (j === i ? val : r)))
  const remove = (i: number) => { onChange(rules.filter((_, j) => j !== i)); setEditIdx(null) }

  const handleKey = (e: React.KeyboardEvent, i: number) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      if (rules[i] === '') { remove(i) } else {
        const next = [...rules.slice(0, i + 1), '', ...rules.slice(i + 1)]
        onChange(next); setEditIdx(i + 1)
      }
    } else if (e.key === 'Backspace' && rules[i] === '') {
      e.preventDefault(); remove(i); setEditIdx(i > 0 ? i - 1 : null)
    } else if (e.key === 'Escape') { setEditIdx(null) }
  }

  return (
    <div className="bg-muted/30 border border-border/60 rounded-md px-3 py-2 font-mono text-[10px] text-muted-foreground leading-relaxed">
      {rules.length === 0 && editIdx === null && (
        <div className="text-muted-foreground/40 italic mb-1">No rules yet</div>
      )}
      {rules.map((rule, i) => (
        <div key={i} className="flex items-baseline gap-1.5">
          <span className="text-muted-foreground/30 select-none">-</span>
          {editIdx === i ? (
            <input
              ref={inputRef}
              value={rule}
              onChange={(e) => update(i, e.target.value)}
              onBlur={() => { if (rule === '') remove(i); else setEditIdx(null) }}
              onKeyDown={(e) => handleKey(e, i)}
              spellCheck={false}
              className="flex-1 bg-transparent border-none outline-none font-mono text-[10px] text-foreground"
              placeholder="Enter rule..."
            />
          ) : (
            <button
              onClick={() => setEditIdx(i)}
              className="flex-1 text-left bg-transparent border-none cursor-text font-mono text-[10px] text-muted-foreground p-0"
            >
              {rule || <span className="text-muted-foreground/30 italic">Empty</span>}
            </button>
          )}
        </div>
      ))}
      <button
        onClick={add}
        className="mt-1 bg-transparent border-none cursor-pointer font-mono text-[10px] text-muted-foreground/40 hover:text-muted-foreground transition-colors p-0"
      >
        + Add rule
      </button>
    </div>
  )
}

// ── Chip ───────────────────────────────────────────────────────────────────────

function Chip({ children, dotClass, onRemove }: { children: React.ReactNode; dotClass: string; onRemove: () => void }) {
  return (
    <span className="inline-flex items-center gap-1 rounded-full border border-border/40 bg-muted/30 px-2 py-0.5 text-[10px] text-foreground">
      <span className={`size-1.5 rounded-full shrink-0 ${dotClass}`} />
      {children}
      <button
        onClick={onRemove}
        className="text-muted-foreground/40 hover:text-muted-foreground transition-colors leading-none"
      >
        ×
      </button>
    </span>
  )
}

// ── Providers ──────────────────────────────────────────────────────────────────

function ProvidersTab({ profile }: { profile: Profile; onChange: (p: Partial<Profile>) => void }) {
  return (
    <div className="max-w-xl">
      {/* Provider sub-nav */}
      <div className="flex border-b border-border mb-4">
        {PROVIDERS.map((p) => {
          const active = profile.selectedProviders.includes(p.id)
          return (
            <div
              key={p.id}
              className={`flex items-center gap-1.5 px-3 py-2 text-[10px] border-b-2 -mb-px transition-colors ${
                active
                  ? 'border-violet-500 text-violet-400'
                  : 'border-transparent text-muted-foreground/40'
              }`}
            >
              {active && <span className="size-1.5 rounded-full bg-violet-500" />}
              {p.name.split(' ')[0]}
            </div>
          )
        })}
      </div>

      {/* Placeholder */}
      <div className="rounded-lg border border-dashed border-border/60 p-6 text-center bg-muted/10">
        <p className="text-xs font-medium text-muted-foreground mb-1">Coming soon — schemas pending</p>
        <p className="text-[11px] text-muted-foreground/50">
          Model, thinking, hooks, env vars, memory — full config once Specta type generation lands.
        </p>
      </div>
    </div>
  )
}
