import type { ReactNode } from 'react'
import { X } from 'lucide-react'

// ── Orange dot indicator for unfinished backend features ─────────────────────

export function OrangeDot() {
  return <span className="inline-block size-2 rounded-full bg-primary" />
}

// ── Section wrapper ──────────────────────────────────────────────────────────

export function SettingsSection({
  icon,
  title,
  action,
  danger,
  children,
}: {
  icon: ReactNode
  title: string
  action?: ReactNode
  danger?: boolean
  children: ReactNode
}) {
  return (
    <div
      className={`mb-4 overflow-hidden rounded-xl border bg-card ${
        danger ? 'border-destructive/30' : 'border-border/60'
      }`}
    >
      <div
        className={`flex items-center justify-between px-4 py-3 border-b ${
          danger ? 'border-destructive/20' : 'border-border/40'
        }`}
      >
        <div
          className={`flex items-center gap-2 text-[13px] font-semibold ${
            danger ? 'text-destructive' : 'text-muted-foreground'
          }`}
        >
          <span className="text-muted-foreground">{icon}</span>
          {title}
        </div>
        {action}
      </div>
      <div className="px-4 py-3">{children}</div>
    </div>
  )
}

// ── Row within a section ─────────────────────────────────────────────────────

export function SettingsRow({
  label,
  sublabel,
  children,
}: {
  label: string
  sublabel?: string
  children: ReactNode
}) {
  return (
    <div className="flex items-center justify-between border-b border-border/30 py-2 last:border-b-0">
      <div>
        <div className="text-xs text-foreground/80">{label}</div>
        {sublabel && (
          <div className="mt-0.5 text-[10px] text-muted-foreground">
            {sublabel}
          </div>
        )}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  )
}

// ── Select dropdown ──────────────────────────────────────────────────────────

export function SettingsSelect({
  value,
  onChange,
  options,
}: {
  value: string
  onChange: (value: string) => void
  options: Array<{ value: string; label: string }>
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="rounded-md border border-border/60 bg-muted/30 px-2 py-1 text-[11px] text-foreground outline-none focus:border-ring"
    >
      {options.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  )
}

// ── Env var row ──────────────────────────────────────────────────────────────

export function EnvVarRow({
  envKey,
  envValue,
  onKeyChange,
  onValueChange,
  onRemove,
}: {
  envKey: string
  envValue: string
  onKeyChange: (v: string) => void
  onValueChange: (v: string) => void
  onRemove: () => void
}) {
  return (
    <div className="mb-1.5 flex items-center gap-1.5">
      <input
        value={envKey}
        onChange={(e) => onKeyChange(e.target.value)}
        placeholder="KEY"
        className="w-40 rounded border border-border/60 bg-muted/30 px-2 py-1 font-mono text-[11px] text-sky-600 dark:text-sky-300 outline-none focus:border-ring"
      />
      <span className="text-xs text-muted-foreground">=</span>
      <input
        value={envValue}
        onChange={(e) => onValueChange(e.target.value)}
        placeholder="value"
        className="flex-1 rounded border border-border/60 bg-muted/30 px-2 py-1 font-mono text-[11px] text-emerald-600 dark:text-emerald-300 outline-none focus:border-ring"
      />
      <button
        onClick={onRemove}
        className="shrink-0 text-muted-foreground/50 hover:text-destructive transition"
      >
        <X className="size-3" />
      </button>
    </div>
  )
}

// ── Hook row ─────────────────────────────────────────────────────────────────

export function HookRow({
  trigger,
  command,
  onRemove,
}: {
  trigger: string
  command: string
  onRemove: () => void
}) {
  return (
    <div className="flex items-center gap-2 border-b border-border/40 py-1.5 last:border-b-0">
      <span className="shrink-0 rounded bg-primary/15 px-2 py-0.5 text-[10px] font-semibold text-primary">
        {trigger}
      </span>
      <code className="flex-1 truncate font-mono text-[11px] text-muted-foreground">
        {command}
      </code>
      <button
        onClick={onRemove}
        className="shrink-0 text-muted-foreground/50 hover:text-destructive transition"
      >
        <X className="size-3" />
      </button>
    </div>
  )
}
