import { betterAuth } from 'better-auth'
import { github } from 'better-auth/providers/github'
import { tanstackStartCookies } from 'better-auth/tanstack-start'

function getEnv(key: string): string {
  return (
    ((globalThis as Record<string, unknown>)[key] as string | undefined) ??
    process.env[key] ??
    ''
  )
}

function makeAuth() {
  return betterAuth({
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
