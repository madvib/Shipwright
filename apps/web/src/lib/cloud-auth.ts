// Cloud auth helpers — JWT signing/validation for Ship API
// Uses HS256 with BETTER_AUTH_SECRET as the signing key.
// Intentionally separate from Better Auth (apps/web/src/lib/auth.ts).

const ALG = 'HS256'
const TOKEN_TTL_SECONDS = 60 * 60 * 24 * 7 // 7 days

export interface JwtPayload {
  sub: string // user id
  org: string // org id
  iat: number
  exp: number
}

function base64url(input: ArrayBuffer | Uint8Array): string {
  const bytes = input instanceof Uint8Array ? input : new Uint8Array(input)
  let binary = ''
  for (const b of bytes) binary += String.fromCharCode(b)
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '')
}

function base64urlDecode(input: string): Uint8Array {
  const padded = input.replace(/-/g, '+').replace(/_/g, '/').padEnd(
    input.length + ((4 - (input.length % 4)) % 4),
    '=',
  )
  const binary = atob(padded)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
  return bytes
}

async function importKey(secret: string): Promise<CryptoKey> {
  const enc = new TextEncoder()
  return crypto.subtle.importKey(
    'raw',
    enc.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign', 'verify'],
  )
}

export async function signJwt(payload: Omit<JwtPayload, 'iat' | 'exp'>, secret: string): Promise<string> {
  const now = Math.floor(Date.now() / 1000)
  const full: JwtPayload = { ...payload, iat: now, exp: now + TOKEN_TTL_SECONDS }

  const header = base64url(new TextEncoder().encode(JSON.stringify({ alg: ALG, typ: 'JWT' })))
  const body = base64url(new TextEncoder().encode(JSON.stringify(full)))
  const signingInput = `${header}.${body}`

  const key = await importKey(secret)
  const sig = await crypto.subtle.sign('HMAC', key, new TextEncoder().encode(signingInput))

  return `${signingInput}.${base64url(sig)}`
}

export async function verifyJwt(token: string, secret: string): Promise<JwtPayload | null> {
  const parts = token.split('.')
  if (parts.length !== 3) return null

  const [header, body, sig] = parts
  const signingInput = `${header}.${body}`

  const key = await importKey(secret)
  const valid = await crypto.subtle.verify(
    'HMAC',
    key,
    base64urlDecode(sig),
    new TextEncoder().encode(signingInput),
  )
  if (!valid) return null

  let payload: unknown
  try {
    payload = JSON.parse(new TextDecoder().decode(base64urlDecode(body)))
  } catch {
    return null
  }

  if (
    typeof payload !== 'object' ||
    payload === null ||
    typeof (payload as Record<string, unknown>).sub !== 'string' ||
    typeof (payload as Record<string, unknown>).exp !== 'number'
  ) {
    return null
  }

  const p = payload as JwtPayload
  if (p.exp < Math.floor(Date.now() / 1000)) return null

  return p
}

// D1 database type used across API routes
export type D1DB = {
  prepare: (query: string) => {
    bind: (...args: unknown[]) => {
      first: <T = unknown>() => Promise<T | null>
      all: <T = unknown>() => Promise<{ results: T[] }>
      run: () => Promise<unknown>
    }
    first: <T = unknown>() => Promise<T | null>
    all: <T = unknown>() => Promise<{ results: T[] }>
    run: () => Promise<unknown>
  }
}

export function getDb(): D1DB | null {
  return (globalThis as Record<string, unknown>)['DB'] as D1DB | undefined ?? null
}

export function getSecret(): string | null {
  return (
    (globalThis as Record<string, unknown>)['BETTER_AUTH_SECRET'] as string | undefined ??
    process.env['BETTER_AUTH_SECRET'] ??
    null
  )
}

export async function requireAuth(request: Request): Promise<JwtPayload | Response> {
  const authHeader = request.headers.get('Authorization')
  if (!authHeader?.startsWith('Bearer ')) {
    return Response.json({ error: 'Missing or invalid Authorization header' }, { status: 401 })
  }

  const token = authHeader.slice(7)
  const secret = getSecret()
  if (!secret) {
    return Response.json({ error: 'Server misconfiguration: missing secret' }, { status: 500 })
  }

  const payload = await verifyJwt(token, secret)
  if (!payload) {
    return Response.json({ error: 'Invalid or expired token' }, { status: 401 })
  }

  return payload
}
