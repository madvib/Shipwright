// Session-based auth for web API routes.
// Uses Better Auth's cookie session (via getAuth().api) as the primary check,
// falling back to the JWT bearer token used by CLI clients.

import { getAuth } from '#/lib/auth'
import { requireAuth as requireJwtAuth, type JwtPayload } from '#/lib/cloud-auth'

export interface SessionUser {
  sub: string
  org: string
}

/**
 * Authenticate a request using Better Auth cookie session or JWT bearer token.
 *
 * - Cookie session: checked via Better Auth's getSession handler.
 * - Bearer token: validated via the existing JWT-based requireAuth.
 *
 * Returns the user/org context, or a 401 Response if neither method works.
 */
export async function requireSession(request: Request): Promise<SessionUser | Response> {
  // 1. Try Better Auth cookie session first (web browser flow)
  try {
    const auth = await getAuth()
    const sessionResponse = await auth.api.getSession({
      headers: request.headers,
    })

    if (sessionResponse?.user?.id) {
      // For cookie sessions, derive org from the user id.
      // In the current schema, org_id is the user's personal org.
      // This matches the pattern established by the CLI auth token flow.
      return {
        sub: sessionResponse.user.id,
        org: sessionResponse.user.id, // Personal org fallback
      }
    }
  } catch {
    // Cookie session check failed — fall through to JWT
  }

  // 2. Fall back to JWT bearer token (CLI flow)
  const jwtResult = await requireJwtAuth(request)
  if (jwtResult instanceof Response) {
    return jwtResult
  }

  return {
    sub: (jwtResult as JwtPayload).sub,
    org: (jwtResult as JwtPayload).org,
  }
}
