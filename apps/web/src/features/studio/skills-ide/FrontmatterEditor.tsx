// Rich editor for SKILL.md frontmatter fields.
// Renders editable inputs for each field based on the Agent Skills spec.
// Changes call onUpdate which recomposes the frontmatter into the document.

import { useState, useCallback } from 'react'
import { Plus, X, AlertCircle } from 'lucide-react'
import type { FrontmatterEntry } from '@ship/primitives'

// Agent Skills specification field definitions
const SPEC_FIELDS = [
  { key: 'name', label: 'Name', required: true, type: 'text' as const, maxLength: 64, pattern: /^[a-z0-9]([a-z0-9-]*[a-z0-9])?$/, hint: 'Lowercase letters, numbers, hyphens. Max 64 chars.' },
  { key: 'description', label: 'Description', required: true, type: 'textarea' as const, maxLength: 1024, hint: 'What this skill does and when to use it.' },
  { key: 'stable-id', label: 'Stable ID', required: false, type: 'text' as const, pattern: /^[a-z0-9][a-z0-9-]*$/, hint: 'Persistence key. Lowercase, hyphens.' },
  { key: 'tags', label: 'Tags', required: false, type: 'tags' as const, hint: 'Keywords for discovery.' },
  { key: 'authors', label: 'Authors', required: false, type: 'tags' as const, hint: 'Skill authors.' },
  { key: 'version', label: 'Version', required: false, type: 'text' as const, hint: 'Semantic version.' },
  { key: 'license', label: 'License', required: false, type: 'text' as const, hint: 'SPDX identifier or license file reference.' },
  { key: 'compatibility', label: 'Compatibility', required: false, type: 'text' as const, maxLength: 500, hint: 'Environment requirements.' },
  { key: 'allowed-tools', label: 'Allowed Tools', required: false, type: 'tags' as const, hint: 'Pre-approved tools. Space-delimited.' },
] as const

const KNOWN_KEYS: Set<string> = new Set(SPEC_FIELDS.map((f) => f.key))

interface FrontmatterEditorProps {
  entries: FrontmatterEntry[]
  raw: string | null
  onUpdate: (newRaw: string) => void
}

export function FrontmatterEditor({ entries, raw, onUpdate }: FrontmatterEditorProps) {
  const [newKey, setNewKey] = useState('')
  const [newValue, setNewValue] = useState('')

  const entryMap = new Map<string, string>(entries.map((e) => [e.key, e.value]))
  const customEntries = entries.filter((e) => !KNOWN_KEYS.has(e.key))

  const updateField = useCallback((key: string, value: string) => {
    const lines = (raw ?? '').split('\n')
    const pattern = new RegExp(`^${escapeRegex(key)}\\s*:\\s*.*$`)
    const newLine = `${key}: ${value.includes('\n') ? `"${value.replace(/"/g, '\\"')}"` : value}`

    const idx = lines.findIndex((l) => pattern.test(l))
    if (idx >= 0) {
      if (value === '') {
        lines.splice(idx, 1)
      } else {
        lines[idx] = newLine
      }
    } else if (value !== '') {
      lines.push(newLine)
    }

    onUpdate(lines.join('\n'))
  }, [raw, onUpdate])

  const updateTagsField = useCallback((key: string, tags: string[]) => {
    const value = tags.length > 0 ? `[${tags.map((t) => t.includes(',') ? `"${t}"` : t).join(', ')}]` : ''
    updateField(key, value)
  }, [updateField])

  const addCustomField = useCallback(() => {
    if (!newKey.trim()) return
    updateField(newKey.trim(), newValue.trim() || '""')
    setNewKey('')
    setNewValue('')
  }, [newKey, newValue, updateField])

  return (
    <div className="space-y-3 px-4 py-3">
      {/* Spec fields */}
      {SPEC_FIELDS.map((field) => {
        const value = entryMap.get(field.key) ?? ''
        const isMissing = field.required && !value
        return (
          <FieldRow key={field.key} label={field.label} required={field.required} error={isMissing ? `${field.label} is required` : undefined}>
            {field.type === 'textarea' ? (
              <textarea
                value={unquote(value)}
                onChange={(e) => updateField(field.key, e.target.value)}
                maxLength={'maxLength' in field ? field.maxLength : undefined}
                rows={3}
                className="w-full rounded-md border border-border bg-background px-2.5 py-1.5 text-xs text-foreground outline-none focus:border-primary/50 resize-none"
                placeholder={field.hint}
              />
            ) : field.type === 'tags' ? (
              <TagsInput
                value={parseInlineArray(value)}
                onChange={(tags) => updateTagsField(field.key, tags)}
                placeholder={`Add ${field.label.toLowerCase()}...`}
              />
            ) : (
              <input
                type="text"
                value={unquote(value)}
                onChange={(e) => updateField(field.key, e.target.value)}
                maxLength={'maxLength' in field ? field.maxLength : undefined}
                className="w-full rounded-md border border-border bg-background px-2.5 py-1.5 text-xs text-foreground outline-none focus:border-primary/50"
                placeholder={field.hint}
              />
            )}
          </FieldRow>
        )
      })}

      {/* Custom metadata fields */}
      {customEntries.length > 0 && (
        <>
          <div className="border-t border-border/40 pt-3">
            <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/60">Custom Fields</span>
          </div>
          {customEntries.map((entry) => (
            <div key={entry.key} className="flex items-start gap-2">
              <div className="flex-1">
                <label className="text-[11px] font-medium text-muted-foreground mb-0.5 block">{entry.key}</label>
                <input
                  type="text"
                  value={unquote(entry.value)}
                  onChange={(e) => updateField(entry.key, e.target.value)}
                  className="w-full rounded-md border border-border bg-background px-2.5 py-1.5 text-xs text-foreground outline-none focus:border-primary/50"
                />
              </div>
              <button onClick={() => updateField(entry.key, '')} className="mt-5 p-1 rounded text-muted-foreground hover:text-destructive transition">
                <X className="size-3" />
              </button>
            </div>
          ))}
        </>
      )}

      {/* Add custom field */}
      <div className="border-t border-border/40 pt-3">
        <div className="flex items-end gap-2">
          <div className="flex-1">
            <label className="text-[10px] text-muted-foreground/60 mb-0.5 block">Key</label>
            <input
              type="text"
              value={newKey}
              onChange={(e) => setNewKey(e.target.value)}
              placeholder="field-name"
              className="w-full rounded-md border border-border bg-background px-2 py-1 text-xs outline-none focus:border-primary/50"
              onKeyDown={(e) => { if (e.key === 'Enter') addCustomField() }}
            />
          </div>
          <div className="flex-1">
            <label className="text-[10px] text-muted-foreground/60 mb-0.5 block">Value</label>
            <input
              type="text"
              value={newValue}
              onChange={(e) => setNewValue(e.target.value)}
              placeholder="value"
              className="w-full rounded-md border border-border bg-background px-2 py-1 text-xs outline-none focus:border-primary/50"
              onKeyDown={(e) => { if (e.key === 'Enter') addCustomField() }}
            />
          </div>
          <button onClick={addCustomField} disabled={!newKey.trim()} className="p-1.5 rounded-md border border-border text-muted-foreground hover:text-foreground hover:bg-muted/30 transition disabled:opacity-40">
            <Plus className="size-3.5" />
          </button>
        </div>
      </div>
    </div>
  )
}

// ── Sub-components ──

function FieldRow({ label, required, error, children }: {
  label: string; required: boolean; error?: string; children: React.ReactNode
}) {
  return (
    <div>
      <div className="flex items-center gap-1 mb-1">
        <label className="text-[11px] font-medium text-foreground/80">{label}</label>
        {required && <span className="text-[9px] text-primary font-semibold">required</span>}
        {error && (
          <span className="flex items-center gap-0.5 text-[9px] text-destructive ml-auto">
            <AlertCircle className="size-2.5" />
            {error}
          </span>
        )}
      </div>
      {children}
    </div>
  )
}

function TagsInput({ value, onChange, placeholder }: { value: string[]; onChange: (tags: string[]) => void; placeholder: string }) {
  const [input, setInput] = useState('')

  const addTag = () => {
    const tag = input.trim()
    if (!tag || value.includes(tag)) return
    onChange([...value, tag])
    setInput('')
  }

  return (
    <div className="rounded-md border border-border bg-background p-1.5">
      <div className="flex flex-wrap gap-1 mb-1">
        {value.map((tag) => (
          <span key={tag} className="flex items-center gap-1 rounded bg-muted px-1.5 py-0.5 text-[10px] font-medium text-foreground/80">
            {tag}
            <button onClick={() => onChange(value.filter((t) => t !== tag))} className="text-muted-foreground hover:text-destructive transition">
              <X className="size-2.5" />
            </button>
          </span>
        ))}
      </div>
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ',') { e.preventDefault(); addTag(); } }}
        onBlur={addTag}
        placeholder={value.length === 0 ? placeholder : ''}
        className="w-full bg-transparent text-xs outline-none px-1"
      />
    </div>
  )
}

// ── Helpers ──

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function unquote(value: string): string {
  const t = value.trim()
  if ((t.startsWith('"') && t.endsWith('"')) || (t.startsWith("'") && t.endsWith("'"))) {
    return t.slice(1, -1)
  }
  return t
}

function parseInlineArray(raw: string): string[] {
  const t = raw.trim()
  if (!t.startsWith('[') || !t.endsWith(']')) return t ? [t] : []
  return t.slice(1, -1).split(',').map((s) => unquote(s.trim())).filter(Boolean)
}
