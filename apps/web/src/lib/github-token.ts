import { getAuth } from '#/lib/auth'

/**
 * Retrieve the GitHub access token from Better Auth's account table.
 *
 * Uses `auth.api.getAccessToken` which reads the session cookie and
 * looks up the stored OAuth token for the 'github' provider.
 *
 * Returns null if the user is not authenticated or has no linked GitHub account.
 */
export async function getGitHubToken(request: Request): Promise<string | null> {
  try {
    const auth = await getAuth()
    const result = await auth.api.getAccessToken({
      body: { providerId: 'github' },
      headers: request.headers,
    })
    return result?.accessToken ?? null
  } catch {
    return null
  }
}
