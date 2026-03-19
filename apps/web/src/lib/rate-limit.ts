// Per-IP sliding window rate limiter backed by Cloudflare KV.
// Falls back to allowed=true in local dev when KV is unavailable.
//
// Key format: "rl:{key}:{ip}"
// Value: JSON array of timestamps (Unix ms) within the current window.

import { env as cloudflareEnv } from 'cloudflare:workers'

/**
 * Check whether a request is within the allowed rate limit.
 *
 * @param request - Incoming request (used to extract CF-Connecting-IP)
 * @param key     - Logical operation key, e.g. "publish", "claim", "install"
 * @param limit   - Maximum number of requests allowed in the window
 * @param windowSeconds - Sliding window duration in seconds
 * @returns { allowed: boolean, retryAfter: number } — retryAfter is 0 when allowed
 */
export async function checkRateLimit(
  request: Request,
  key: string,
  limit: number,
  windowSeconds: number,
): Promise<{ allowed: boolean; retryAfter: number }> {
  const kv: KVNamespace | undefined = (cloudflareEnv as Partial<Env>).RATE_LIMIT_KV

  // Local dev fallback — KV not wired up
  if (!kv) {
    return { allowed: true, retryAfter: 0 }
  }

  const ip =
    request.headers.get('CF-Connecting-IP') ??
    request.headers.get('X-Forwarded-For')?.split(',')[0]?.trim() ??
    'unknown'

  const kvKey = `rl:${key}:${ip}`
  const now = Date.now()
  const windowMs = windowSeconds * 1000
  const cutoff = now - windowMs

  // Read existing timestamps
  const raw = await kv.get(kvKey)
  let timestamps: number[] = []
  if (raw) {
    try {
      const parsed = JSON.parse(raw) as unknown
      if (Array.isArray(parsed)) {
        timestamps = (parsed as unknown[]).filter(
          (t): t is number => typeof t === 'number',
        )
      }
    } catch {
      // corrupt entry — treat as empty
    }
  }

  // Prune timestamps outside the window
  const active = timestamps.filter((t) => t > cutoff)

  if (active.length >= limit) {
    // Oldest timestamp in window tells us when a slot frees up
    const oldest = active[0] ?? now
    const retryAfter = Math.ceil((oldest + windowMs - now) / 1000)
    return { allowed: false, retryAfter: Math.max(1, retryAfter) }
  }

  // Append current timestamp and persist
  active.push(now)
  await kv.put(kvKey, JSON.stringify(active), { expirationTtl: windowSeconds })

  return { allowed: true, retryAfter: 0 }
}

/**
 * Build a 429 rate-limit Response with the correct Retry-After header.
 */
export function rateLimitResponse(retryAfter: number): Response {
  return Response.json(
    { error: 'Rate limit exceeded', retryAfter },
    {
      status: 429,
      headers: { 'Retry-After': String(retryAfter) },
    },
  )
}
