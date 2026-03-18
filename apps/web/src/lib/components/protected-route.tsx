import { useEffect, useRef, createContext, useContext } from 'react'
import { authClient } from '#/lib/auth-client'

interface AuthContextValue {
  isAuthenticated: boolean
  isPending: boolean
  user: { id: string; name: string; email: string; image?: string | null } | null
}

const AuthContext = createContext<AuthContextValue>({
  isAuthenticated: false,
  isPending: true,
  user: null,
})

export function useAuth(): AuthContextValue {
  return useContext(AuthContext)
}

interface ProtectedRouteProps {
  children: React.ReactNode
}

/**
 * ProtectedRoute — wraps studio routes.
 *
 * NOT a gate. Unauthenticated users can still use the studio (localStorage mode).
 * When authenticated, enables sync features (server-side persistence, etc.).
 *
 * Checks Better Auth session on mount and provides auth context to children.
 */
export function ProtectedRoute({ children }: ProtectedRouteProps) {
  const { data: session, isPending } = authClient.useSession()
  const hasChecked = useRef(false)

  const isAuthenticated = !!session?.user
  const user = session?.user
    ? {
        id: session.user.id,
        name: session.user.name,
        email: session.user.email,
        image: session.user.image,
      }
    : null

  useEffect(() => {
    if (isPending || hasChecked.current) return
    hasChecked.current = true

    // No redirect — unauthenticated access is allowed.
    // This effect is a hook point for future sync-on-mount logic.
  }, [isPending])

  const value: AuthContextValue = { isAuthenticated, isPending, user }

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  )
}
