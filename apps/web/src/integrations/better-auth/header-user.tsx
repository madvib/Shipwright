import { authClient } from '#/lib/auth-client'
import { Button } from '@ship/primitives'
import { Link } from '@tanstack/react-router'

export default function BetterAuthHeader() {
  const { data: session, isPending } = authClient.useSession()

  if (isPending) {
    return (
      <div className="h-8 w-8 bg-muted animate-pulse rounded-md" />
    )
  }

  if (session?.user) {
    return (
      <div className="flex items-center gap-2">
        {session.user.image ? (
          <img src={session.user.image} alt="" className="h-8 w-8 rounded-md" />
        ) : (
          <div className="h-8 w-8 bg-muted flex items-center justify-center rounded-md">
            <span className="text-xs font-medium text-muted-foreground">
              {session.user.name?.charAt(0).toUpperCase() || 'U'}
            </span>
          </div>
        )}
        <Button
          variant="outline"
          size="default"
          onClick={() => {
            void authClient.signOut()
          }}
        >
          Sign out
        </Button>
      </div>
    )
  }

  return (
    <Button variant="outline" size="default" render={<Link to="/studio" />}>
      Sign in
    </Button>
  )
}
