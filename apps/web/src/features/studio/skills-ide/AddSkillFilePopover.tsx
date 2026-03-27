import { useState, useRef, useEffect } from 'react'
import {
  Plus, FileJson, BookOpen, FileText, X, Terminal, FileCode,
} from 'lucide-react'
import type { LibrarySkill } from './useSkillsLibrary'

const ADDABLE_FILES = [
  {
    id: 'vars',
    path: 'assets/vars.json',
    label: 'Variables',
    description: 'Typed config — makes this a smart skill',
    icon: FileJson,
    color: 'text-amber-400',
  },
  {
    id: 'docs',
    path: 'references/docs/index.md',
    label: 'Reference docs',
    description: 'Human + agent readable documentation',
    icon: BookOpen,
    color: 'text-emerald-400',
  },
  {
    id: 'api',
    path: 'references/api/index.md',
    label: 'API reference',
    description: 'API tables, external specs',
    icon: FileCode,
    color: 'text-sky-400',
  },
  {
    id: 'script',
    path: 'scripts/run.sh',
    label: 'Script',
    description: 'Helper script referenced in SKILL.md',
    icon: Terminal,
    color: 'text-violet-400',
  },
  {
    id: 'template',
    path: 'assets/templates/config.md',
    label: 'Template',
    description: 'Reusable config snippet',
    icon: FileText,
    color: 'text-muted-foreground',
  },
] as const

type AddableFileId = (typeof ADDABLE_FILES)[number]['id']

function defaultContent(id: AddableFileId, skillId: string): string {
  switch (id) {
    case 'vars':
      return JSON.stringify(
        {
          $schema: 'https://getship.dev/schemas/vars.schema.json',
          example_var: {
            type: 'string',
            default: '',
            'storage-hint': 'global',
            label: 'Example variable',
            description: 'Replace this with your first variable.',
          },
        },
        null,
        2,
      )
    case 'docs':
      return `---\ntitle: ${skillId}\ndescription: Reference documentation for the ${skillId} skill.\n---\n\n# ${skillId}\n\n`
    case 'api':
      return `---\ntitle: ${skillId} API\ndescription: API reference for the ${skillId} skill.\n---\n\n# API Reference\n\n| Endpoint | Method | Description |\n|----------|--------|-------------|\n| | | |\n`
    case 'script':
      return `#!/usr/bin/env bash\nset -euo pipefail\n\n# Helper script for ${skillId}\n# Referenced from SKILL.md\n\n`
    case 'template':
      return `# Config template for ${skillId}\n\n`
    default:
      return ''
  }
}

interface Props {
  skill: LibrarySkill
  onAddFile: (skillId: string, filePath: string, content: string) => void
}

export function AddSkillFilePopover({ skill, onAddFile }: Props) {
  const [open, setOpen] = useState(false)
  const triggerRef = useRef<HTMLButtonElement>(null)
  const popoverRef = useRef<HTMLDivElement>(null)
  const existingFiles = new Set(skill.files)

  const hasScripts = skill.files.some((f) => f.startsWith('scripts/'))
  const hasTemplates = skill.files.some((f) => f.startsWith('assets/templates/'))
  const hasApiRef = skill.files.some((f) => f.startsWith('references/api/'))

  const available = ADDABLE_FILES.filter((f) => {
    if (existingFiles.has(f.path)) return false
    if (f.id === 'script' && hasScripts) return false
    if (f.id === 'template' && hasTemplates) return false
    if (f.id === 'api' && hasApiRef) return false
    return true
  })

  // Position the popover relative to the trigger, using fixed positioning to escape overflow
  useEffect(() => {
    if (!open || !triggerRef.current || !popoverRef.current) return
    const rect = triggerRef.current.getBoundingClientRect()
    const pop = popoverRef.current
    pop.style.top = `${rect.bottom + 4}px`
    // Position to the right of the sidebar to avoid clipping
    pop.style.left = `${Math.max(8, rect.left)}px`
  }, [open])

  if (available.length === 0) return null

  return (
    <>
      <button
        ref={triggerRef}
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground transition-colors"
        title="Add file to skill"
      >
        <Plus className="size-3" />
        <span>Add file</span>
      </button>

      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />

          <div
            ref={popoverRef}
            className="fixed z-50 w-56 rounded-lg border border-border bg-popover shadow-lg animate-in fade-in slide-in-from-top-1 duration-150"
          >
            <div className="flex items-center justify-between px-3 py-2 border-b border-border">
              <span className="text-[11px] font-semibold text-foreground">Add to {skill.id}</span>
              <button onClick={() => setOpen(false)} className="text-muted-foreground hover:text-foreground">
                <X className="size-3" />
              </button>
            </div>

            <div className="p-1">
              {available.map((file) => {
                const Icon = file.icon
                return (
                  <button
                    key={file.id}
                    onClick={() => {
                      onAddFile(skill.id, file.path, defaultContent(file.id, skill.stableId ?? skill.id))
                      setOpen(false)
                    }}
                    className="flex items-start gap-2.5 w-full px-2.5 py-2 rounded-md text-left hover:bg-muted/50 transition-colors"
                  >
                    <Icon className={`size-3.5 shrink-0 mt-0.5 ${file.color}`} />
                    <div className="min-w-0">
                      <div className="text-xs font-medium text-foreground">{file.label}</div>
                      <div className="text-[10px] text-muted-foreground leading-snug">{file.description}</div>
                    </div>
                  </button>
                )
              })}
            </div>
          </div>
        </>
      )}
    </>
  )
}
