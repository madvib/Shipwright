import { createFileRoute } from '@tanstack/react-router'
import { getTokenFromCookie, listRepos, getUser } from '#/lib/github-app'

export const Route = createFileRoute('/api/github/repos')({
  server: {
    handlers: {
      /** List authenticated user's repos. Requires gh_token cookie. */
      GET: async ({ request }) => {
        const token = getTokenFromCookie(request)
        if (!token) {
          return Response.json({ error: 'Not authenticated' }, { status: 401 })
        }

        const url = new URL(request.url)
        const page = Number(url.searchParams.get('page') ?? '1')

        try {
          const [user, repos] = await Promise.all([
            getUser(token),
            listRepos(token, page),
          ])

          return Response.json({
            user: { login: user.login, avatar_url: user.avatar_url },
            repos: repos.map((r) => ({
              full_name: r.full_name,
              name: r.name,
              owner: r.owner.login,
              private: r.private,
              default_branch: r.default_branch,
              description: r.description,
            })),
          })
        } catch (err) {
          const msg = err instanceof Error ? err.message : 'Failed to fetch repos'
          return Response.json({ error: msg }, { status: 502 })
        }
      },
    },
  },
})
