import { betterAuth } from 'better-auth'
import { d1 } from 'better-auth/adapters/d1'
import { github } from 'better-auth/providers/github'
import { tanstackStartCookies } from 'better-auth/tanstack-start'
import { getDb } from '#/lib/cloud-auth'

function getEnv(key: string): string {
  return (
    ((globalThis as Record<string, unknown>)[key] as string | undefined) ??
    process.env[key] ??
    ''
  )
}

function makeAuth() {
  const db = getDb()
  return betterAuth({
    ...(db ? { database: d1(db as Parameters<typeof d1>[0]) } : {}),
    socialProviders: {
      github: {
        clientId: getEnv('GITHUB_APP_CLIENT_ID'),
        clientSecret: getEnv('GITHUB_APP_CLIENT_SECRET'),
      },
    },
    plugins: [tanstackStartCookies()],
  })
}

let _auth: ReturnType<typeof makeAuth> | undefined

export function getAuth() {
  if (!_auth) _auth = makeAuth()
  return _auth
}
