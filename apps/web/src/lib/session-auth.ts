// Session-based auth for web API routes.
// Uses Better Auth's cookie session (via getAuth().api) as the primary check.
// Returns a lightweight SessionUser or a 401 Response.

import { getAuth } from '#/lib/auth'

export interface SessionUser {
  sub: string
  org: string
}

/**
 * Authenticate a request using Better Auth cookie session.
 *
 * Returns the user/org context, or a 401 Response if not authenticated.
 */
export async function requireSession(
  request: Request,
): Promise<SessionUser | Response> {
  try {
    const auth = await getAuth()
    const sessionResponse = await auth.api.getSession({
      headers: request.headers,
    })

    if (sessionResponse?.user?.id) {
      return {
        sub: sessionResponse.user.id,
        org: sessionResponse.user.id,
      }
    }
  } catch {
    // Session check failed
  }

  return Response.json(
    { error: 'Authentication required' },
    { status: 401 },
  )
}

/**
 * Try to extract session user from request, returning null if not authenticated.
 * Use this for optional auth (e.g. publish can work with or without auth).
 */
export async function optionalSession(
  request: Request,
): Promise<SessionUser | null> {
  try {
    const auth = await getAuth()
    const sessionResponse = await auth.api.getSession({
      headers: request.headers,
    })

    if (sessionResponse?.user?.id) {
      return {
        sub: sessionResponse.user.id,
        org: sessionResponse.user.id,
      }
    }
  } catch {
    // Not authenticated
  }

  return null
}
