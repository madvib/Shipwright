import { useEffect } from 'react'

interface UseKeyboardShortcutsOptions {
  onEscape?: () => void
  onSave?: () => void
  onSubmit?: () => void
  disabled?: boolean
}

export function useKeyboardShortcuts({
  onEscape,
  onSave,
  onSubmit,
  disabled = false,
}: UseKeyboardShortcutsOptions) {
  useEffect(() => {
    if (disabled) return

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault()
        onEscape?.()
        return
      }

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault()
        onSave?.()
      }

      if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
        event.preventDefault()
        onSubmit?.()
      }
    }

    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [onEscape, onSave, onSubmit, disabled])
}

export function useDirtyKeyboardShortcuts({
  onEscape,
  onSave,
  dirty,
}: {
  onEscape?: () => void
  onSave?: () => void
  dirty: boolean
}) {
  useEffect(() => {
    if (!dirty) return

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault()
        onEscape?.()
        return
      }

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault()
        onSave?.()
      }
    }

    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [dirty, onEscape, onSave])
}
