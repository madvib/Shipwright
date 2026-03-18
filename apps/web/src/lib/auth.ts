import { getDb } from '#/lib/cloud-auth'

function getEnv(key: string): string {
  return (
    ((globalThis as Record<string, unknown>)[key] as string | undefined) ??
    process.env[key] ??
    ''
  )
}

async function makeAuth() {
  const { betterAuth } = await import('better-auth')
  const { tanstackStartCookies } = await import('better-auth/tanstack-start')

  const d1 = getDb()
  let database: ReturnType<typeof import('better-auth/adapters/drizzle').drizzleAdapter> | undefined

  if (d1) {
    const [{ drizzle }, { drizzleAdapter }, schema] = await Promise.all([
      import('drizzle-orm/d1'),
      import('better-auth/adapters/drizzle'),
      import('#/db/schema'),
    ])
    type AnyD1 = Parameters<typeof drizzle>[0]
    const db = drizzle(d1 as unknown as AnyD1, { schema })
    database = drizzleAdapter(db, { provider: 'sqlite', schema })
  }

  return betterAuth({
    ...(database ? { database } : {}),
    socialProviders: {
      github: {
        clientId: getEnv('GITHUB_CLIENT_ID'),
        clientSecret: getEnv('GITHUB_CLIENT_SECRET'),
      },
    },
    plugins: [tanstackStartCookies()],
  })
}

type Auth = Awaited<ReturnType<typeof makeAuth>>
let _auth: Auth | undefined
let _authPromise: Promise<Auth> | undefined

export function getAuth(): Promise<Auth> {
  if (_auth) return Promise.resolve(_auth)
  if (!_authPromise) {
    _authPromise = makeAuth().then((a) => {
      _auth = a
      return a
    })
  }
  return _authPromise
}
