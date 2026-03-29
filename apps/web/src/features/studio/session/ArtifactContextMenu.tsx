// Right-click context menu for session artifacts.
// Fixed-position overlay with delete confirmation, following the SkillContextMenu pattern.

import { useState, useRef, useEffect, useCallback } from 'react'
import { Trash2, Copy } from 'lucide-react'
import type { SessionFile } from './types'

export interface ArtifactMenuState {
  file: SessionFile
  x: number
  y: number
}

interface Props {
  menu: ArtifactMenuState
  onDelete: (path: string) => void
  onClose: () => void
}

export function ArtifactContextMenu({ menu, onDelete, onClose }: Props) {
  const menuRef = useRef<HTMLDivElement>(null)
  const [confirmDelete, setConfirmDelete] = useState(false)

  useEffect(() => {
    if (!menuRef.current) return
    const el = menuRef.current
    const rect = el.getBoundingClientRect()
    if (rect.right > window.innerWidth) el.style.left = `${menu.x - rect.width}px`
    if (rect.bottom > window.innerHeight) el.style.top = `${menu.y - rect.height}px`
  }, [menu.x, menu.y])

  const handleCopyPath = useCallback(() => {
    void navigator.clipboard.writeText(menu.file.path)
    onClose()
  }, [menu.file.path, onClose])

  const handleDelete = useCallback(() => {
    onDelete(menu.file.path)
    onClose()
  }, [menu.file.path, onDelete, onClose])

  return (
    <>
      <div className="fixed inset-0 z-40" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose() }} />
      <div
        ref={menuRef}
        style={{ top: menu.y, left: menu.x }}
        className="fixed z-50 w-48 rounded-lg border border-border bg-popover shadow-lg animate-in fade-in zoom-in-95 duration-100"
      >
        <div className="px-3 py-1.5 border-b border-border">
          <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide truncate block">
            {menu.file.name}
          </span>
        </div>

        {confirmDelete ? (
          <div className="p-2 space-y-2">
            <p className="text-xs text-foreground">Delete {menu.file.name}?</p>
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
