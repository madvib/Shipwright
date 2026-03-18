import { useEffect, useState, useCallback } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { classifyError, type ApiErrorAction } from '#/lib/api-errors'
import { authKeys } from '#/lib/query-keys'
import { authClient } from '#/lib/auth-client'

interface ErrorNotification {
  id: number
  action: ApiErrorAction
  timestamp: number
}

let nextId = 0

/**
 * Global API error handler hook.
 * Components call `handleApiError(error)` to surface errors.
 * 401 errors clear the session. Network errors show offline indicator.
 */
export function useApiErrorHandler() {
  const queryClient = useQueryClient()
  const [notifications, setNotifications] = useState<ErrorNotification[]>([])

  const handleApiError = useCallback(
    (error: unknown) => {
      const action = classifyError(error)

      if (action.type === 'auth_expired') {
        // Clear session cache so auth-aware components react
        void queryClient.invalidateQueries({ queryKey: authKeys.all })
        void authClient.signOut()
      }

      const notification: ErrorNotification = {
        id: nextId++,
        action,
        timestamp: Date.now(),
      }

      setNotifications((prev) => [...prev.slice(-4), notification])
    },
    [queryClient],
  )

  const dismiss = useCallback((id: number) => {
    setNotifications((prev) => prev.filter((n) => n.id !== id))
  }, [])

  // Auto-dismiss after 6 seconds
  useEffect(() => {
    if (notifications.length === 0) return
    const timer = setTimeout(() => {
      setNotifications((prev) => prev.slice(1))
    }, 6000)
    return () => clearTimeout(timer)
  }, [notifications])

  return { notifications, handleApiError, dismiss }
}

/**
 * Renders a small notification toast for API errors.
 */
export function ApiErrorToast({
  notification,
  onDismiss,
}: {
  notification: ErrorNotification
  onDismiss: () => void
}) {
  const { action } = notification

  let message: string
  let variant: 'error' | 'warning' | 'info' = 'error'

  switch (action.type) {
    case 'auth_expired':
      message = 'Session expired. Please sign in again.'
      variant = 'warning'
      break
    case 'offline':
      message = 'You appear to be offline. Changes are saved locally.'
      variant = 'info'
      break
    case 'server_error':
      message = action.message
      break
    case 'validation':
      message = action.message
      variant = 'warning'
      break
    case 'not_found':
      message = action.message
      variant = 'warning'
      break
  }

  const colors = {
    error: 'border-red-500/30 bg-red-500/10 text-red-400',
    warning: 'border-amber-500/30 bg-amber-500/10 text-amber-400',
    info: 'border-sky-500/30 bg-sky-500/10 text-sky-400',
  }

  return (
    <div
      className={`rounded-lg border px-4 py-3 text-xs font-medium shadow-lg backdrop-blur-md animate-in fade-in slide-in-from-bottom-2 duration-200 ${colors[variant]}`}
    >
      <div className="flex items-start justify-between gap-3">
        <span>{message}</span>
        <button
          onClick={onDismiss}
          className="text-current opacity-50 hover:opacity-100 transition-opacity shrink-0"
        >
          &times;
        </button>
      </div>
    </div>
  )
}
