// Webhook signature verification and payload age helpers.
// Extracted from the webhook handler to keep route files under 300 lines.

/**
 * Verify GitHub webhook signature (HMAC SHA-256).
 * Uses constant-time comparison to prevent timing attacks.
 */
export async function verifySignature(
  secret: string,
  body: string,
  signature: string,
): Promise<boolean> {
  const encoder = new TextEncoder()
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign'],
  )
  const sig = await crypto.subtle.sign('HMAC', key, encoder.encode(body))
  const expected =
    'sha256=' +
    Array.from(new Uint8Array(sig), (b) =>
      b.toString(16).padStart(2, '0'),
    ).join('')

  // Constant-time comparison
  if (expected.length !== signature.length) return false
  let result = 0
  for (let i = 0; i < expected.length; i++) {
    result |= expected.charCodeAt(i) ^ signature.charCodeAt(i)
  }
  return result === 0
}

/**
 * Extract timestamp from a GitHub webhook payload and return its age in ms.
 * Checks repository.pushed_at (Unix epoch) and common ISO 8601 fields.
 * Returns null if no parseable timestamp is found (caller should allow).
 */
export function getPayloadAgeMs(payload: Record<string, unknown>): number | null {
  const now = Date.now()

  // GitHub create/push events: repository.pushed_at is a Unix epoch (seconds)
  const repo = payload.repository as Record<string, unknown> | undefined
  if (repo?.pushed_at && typeof repo.pushed_at === 'number') {
    return now - repo.pushed_at * 1000
  }

  // Installation events: updated_at ISO 8601
  const installation = payload.installation as Record<string, unknown> | undefined
  if (installation?.updated_at && typeof installation.updated_at === 'string') {
    const ts = Date.parse(installation.updated_at)
    if (!Number.isNaN(ts)) return now - ts
  }

  // Fallback: top-level created_at or updated_at
  for (const field of ['created_at', 'updated_at'] as const) {
    const val = payload[field]
    if (typeof val === 'string') {
      const ts = Date.parse(val)
      if (!Number.isNaN(ts)) return now - ts
    }
  }

  return null
}
