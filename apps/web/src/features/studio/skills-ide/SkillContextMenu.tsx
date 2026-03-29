import { useState, useRef, useEffect, useCallback } from 'react'
import {
  FileJson, BookOpen, FileText, Terminal, FileCode, FilePlus, Trash2, Copy,
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

export function getAvailableFiles(skill: LibrarySkill) {
  const existingFiles = new Set(skill.files)
  const hasScripts = skill.files.some((f) => f.startsWith('scripts/'))
  const hasTemplates = skill.files.some((f) => f.startsWith('assets/templates/'))
  const hasApiRef = skill.files.some((f) => f.startsWith('references/api/'))

  return ADDABLE_FILES.filter((f) => {
    if (existingFiles.has(f.path)) return false
    if (f.id === 'script' && hasScripts) return false
    if (f.id === 'template' && hasTemplates) return false
    if (f.id === 'api' && hasApiRef) return false
    return true
  })
}

export type ContextMenuState =
  | { mode: 'folder'; skill: LibrarySkill; x: number; y: number }
  | { mode: 'file'; skill: LibrarySkill; filePath: string; x: number; y: number }

interface Props {
  menu: ContextMenuState
  onAddFile: (skillId: string, filePath: string, content: string) => void
  onDeleteFile: (skillId: string, filePath: string) => void
  onClose: () => void
}

export function SkillContextMenu({ menu, onAddFile, onDeleteFile, onClose }: Props) {
  if (menu.mode === 'file') {
    return <FileContextMenu menu={menu} onDeleteFile={onDeleteFile} onClose={onClose} />
  }
  return <FolderContextMenu menu={menu} onAddFile={onAddFile} onClose={onClose} />
}

/** Context menu shown on right-click of individual files. */
function FileContextMenu({ menu, onDeleteFile, onClose }: {
  menu: ContextMenuState & { mode: 'file' }
  onDeleteFile: (skillId: string, filePath: string) => void
  onClose: () => void
}) {
  const menuRef = useRef<HTMLDivElement>(null)
  const [confirmDelete, setConfirmDelete] = useState(false)
  const fileName = menu.filePath.split('/').pop() ?? menu.filePath

  useEffect(() => {
    if (!menuRef.current) return
    const el = menuRef.current
    const rect = el.getBoundingClientRect()
    if (rect.right > window.innerWidth) el.style.left = `${menu.x - rect.width}px`
    if (rect.bottom > window.innerHeight) el.style.top = `${menu.y - rect.height}px`
  }, [menu.x, menu.y])

  const handleCopyPath = useCallback(() => {
    void navigator.clipboard.writeText(menu.filePath)
    onClose()
  }, [menu.filePath, onClose])

  const handleDelete = useCallback(() => {
    onDeleteFile(menu.skill.id, menu.filePath)
    onClose()
  }, [menu.skill.id, menu.filePath, onDeleteFile, onClose])

  return (
    <>
      <div className="fixed inset-0 z-40" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose() }} />
      <div
        ref={menuRef}
        style={{ top: menu.y, left: menu.x }}
        className="fixed z-50 w-52 rounded-lg border border-border bg-popover shadow-lg animate-in fade-in zoom-in-95 duration-100"
      >
        <div className="px-3 py-1.5 border-b border-border">
          <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide truncate block">
            {fileName}
          </span>
        </div>

        {confirmDelete ? (
          <div className="p-2 space-y-2">
            <p className="text-xs text-foreground">Delete {fileName}?</p>
            <div className="flex items-center gap-1.5">
              <button
                onClick={onClose}
                className="flex-1 px-2 py-1 text-xs rounded border border-border hover:bg-muted/50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleDelete}
                className="flex-1 px-2 py-1 text-xs rounded bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors"
              >
                Delete
              </button>
            </div>
          </div>
        ) : (
          <div className="p-1">
            <button
              onClick={handleCopyPath}
              className="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-md text-left hover:bg-muted/50 transition-colors"
            >
              <Copy className="size-3.5 shrink-0 text-muted-foreground" />
              <span className="text-xs text-foreground">Copy path</span>
            </button>
            <div className="h-px bg-border mx-2 my-1" />
            <button
              onClick={() => setConfirmDelete(true)}
              className="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-md text-left hover:bg-destructive/10 transition-colors"
            >
              <Trash2 className="size-3.5 shrink-0 text-destructive" />
              <span className="text-xs text-destructive">Delete</span>
            </button>
          </div>
        )}
      </div>
    </>
  )
}

/** Context menu shown on right-click of skill folders. */
function FolderContextMenu({ menu, onAddFile, onClose }: {
  menu: ContextMenuState & { mode: 'folder' }
  onAddFile: (skillId: string, filePath: string, content: string) => void
  onClose: () => void
}) {
  const menuRef = useRef<HTMLDivElement>(null)
  const [customInput, setCustomInput] = useState(false)
  const [customPath, setCustomPath] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)
  const available = getAvailableFiles(menu.skill)

  useEffect(() => {
    if (!menuRef.current) return
    const el = menuRef.current
    const rect = el.getBoundingClientRect()
    if (rect.right > window.innerWidth) el.style.left = `${menu.x - rect.width}px`
    if (rect.bottom > window.innerHeight) el.style.top = `${menu.y - rect.height}px`
  }, [menu.x, menu.y])

  useEffect(() => {
    if (customInput && inputRef.current) inputRef.current.focus()
  }, [customInput])

  const handleCustomSubmit = useCallback(() => {
    const path = customPath.trim()
    if (!path) return
    onAddFile(menu.skill.id, path, '')
    onClose()
  }, [customPath, menu.skill.id, onAddFile, onClose])

  return (
    <>
      <div className="fixed inset-0 z-40" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose() }} />
      <div
        ref={menuRef}
        style={{ top: menu.y, left: menu.x }}
        className="fixed z-50 w-52 rounded-lg border border-border bg-popover shadow-lg animate-in fade-in zoom-in-95 duration-100"
      >
        <div className="px-3 py-1.5 border-b border-border">
          <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide">
            Add to {menu.skill.id}
          </span>
        </div>

        <div className="p-1">
          {available.map((file) => {
            const Icon = file.icon
            return (
              <button
                key={file.id}
                onClick={() => {
                  onAddFile(menu.skill.id, file.path, defaultContent(file.id, menu.skill.stableId ?? menu.skill.id))
                  onClose()
                }}
                className="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-md text-left hover:bg-muted/50 transition-colors"
              >
                <Icon className={`size-3.5 shrink-0 ${file.color}`} />
                <span className="text-xs text-foreground">{file.label}</span>
              </button>
            )
          })}

          {available.length > 0 && <div className="h-px bg-border mx-2 my-1" />}

          {!customInput ? (
            <button
              onClick={() => setCustomInput(true)}
              className="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-md text-left hover:bg-muted/50 transition-colors"
            >
              <FilePlus className="size-3.5 shrink-0 text-muted-foreground" />
              <span className="text-xs text-foreground">New file...</span>
            </button>
          ) : (
            <form
              onSubmit={(e) => { e.preventDefault(); handleCustomSubmit() }}
              className="px-2.5 py-1.5"
            >
              <input
                ref={inputRef}
                value={customPath}
                onChange={(e) => setCustomPath(e.target.value)}
                placeholder="path/to/file.md"
                className="w-full rounded border border-border bg-background px-2 py-1 text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:border-primary/50"
                onKeyDown={(e) => { if (e.key === 'Escape') onClose() }}
              />
            </form>
          )}
        </div>
      </div>
    </>
  )
}
