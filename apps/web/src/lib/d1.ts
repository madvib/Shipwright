// D1 database accessor and shared utilities for API routes.
import { env as cloudflareEnv } from 'cloudflare:workers'

/** Get the D1 binding from the Cloudflare Workers environment. */
export function getD1(): D1Database | null {
  return cloudflareEnv.DB ?? null
}

/** Generate a 32-character hex nanoid. */
export function nanoid(): string {
  const bytes = new Uint8Array(16)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
}
