// Per-IP rate limiting via Cloudflare's native Rate Limiting binding.
// Binding names correspond to rate_limits entries in wrangler.jsonc.
// Falls back to allowed=true in local dev when bindings are unavailable.

import { env as cloudflareEnv } from 'cloudflare:workers'

type RateLimitBinding = 'RATE_LIMITER_PUBLISH' | 'RATE_LIMITER_CLAIM' | 'RATE_LIMITER_INSTALL' | 'RATE_LIMITER_SEARCH'

export async function checkRateLimit(
  request: Request,
  binding: RateLimitBinding,
  retryAfterSeconds: number,
): Promise<{ allowed: boolean; retryAfter: number }> {
  const limiter = (cloudflareEnv as Partial<Env>)[binding] as RateLimit | undefined

  // Local dev: binding not wired, allow all requests
  if (!limiter) return { allowed: true, retryAfter: 0 }

  const ip =
    request.headers.get('CF-Connecting-IP') ??
    request.headers.get('X-Forwarded-For')?.split(',')[0]?.trim() ??
    'unknown'

  const { success } = await limiter.limit({ key: ip })
  return { allowed: success, retryAfter: success ? 0 : retryAfterSeconds }
}

export function rateLimitResponse(retryAfter: number): Response {
  return Response.json(
    { error: 'Rate limit exceeded', retryAfter },
    { status: 429, headers: { 'Retry-After': String(retryAfter) } },
  )
}
