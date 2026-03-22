// D1 database accessors and shared utilities for API routes.
import { env as cloudflareEnv } from 'cloudflare:workers'

/** Get the AUTH_DB binding (user, session, account, verification, cli_auth). */
export function getAuthDb(): D1Database | null {
  return cloudflareEnv.AUTH_DB ?? null
}

/** Get the REGISTRY_DB binding (packages, versions, skills, github_installations, mcp_servers). */
export function getRegistryDb(): D1Database | null {
  return cloudflareEnv.REGISTRY_DB ?? null
}

/** Generate a 32-character hex nanoid. */
export function nanoid(): string {
  const bytes = new Uint8Array(16)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
}
