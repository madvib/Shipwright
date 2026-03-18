// Centralized API error handling for Ship Studio.
// Maps HTTP status codes and network errors to actionable UI states.

export interface ApiError {
  status: number
  message: string
  code?: string
}

export type ApiErrorAction =
  | { type: 'auth_expired' }
  | { type: 'server_error'; message: string }
  | { type: 'offline' }
  | { type: 'validation'; message: string }
  | { type: 'not_found'; message: string }

/**
 * Classify a fetch error or HTTP response into an actionable error type.
 * Components can switch on `action.type` to decide what to show.
 */
export function classifyError(error: unknown): ApiErrorAction {
  // Network error — no response at all
  if (error instanceof TypeError && error.message.includes('fetch')) {
    return { type: 'offline' }
  }

  if (isApiError(error)) {
    if (error.status === 401) {
      return { type: 'auth_expired' }
    }
    if (error.status === 404) {
      return { type: 'not_found', message: error.message }
    }
    if (error.status === 400 || error.status === 422) {
      return { type: 'validation', message: error.message }
    }
    return { type: 'server_error', message: error.message }
  }

  if (error instanceof Error) {
    // navigator.onLine can supplement detection
    if (!navigator.onLine) {
      return { type: 'offline' }
    }
    return { type: 'server_error', message: error.message }
  }

  return { type: 'server_error', message: 'An unexpected error occurred.' }
}

function isApiError(error: unknown): error is ApiError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'status' in error &&
    typeof (error as ApiError).status === 'number' &&
    'message' in error &&
    typeof (error as ApiError).message === 'string'
  )
}

/**
 * Wraps a fetch call with structured error handling.
 * Throws an ApiError for non-2xx responses. Callers handle via classifyError.
 */
export async function fetchApi<T>(
  url: string,
  init?: RequestInit,
): Promise<T> {
  let response: Response
  try {
    response = await fetch(url, init)
  } catch (err) {
    // Network-level failure
    throw Object.assign(new Error('Network request failed'), {
      status: 0,
      code: 'NETWORK_ERROR',
    })
  }

  if (!response.ok) {
    let body: { error?: string } = {}
    try {
      body = (await response.json()) as { error?: string }
    } catch {
      // Response body was not JSON
    }

    const apiError: ApiError = {
      status: response.status,
      message: body.error ?? response.statusText ?? 'Request failed',
      code: String(response.status),
    }
    throw apiError
  }

  return (await response.json()) as T
}

/**
 * Whether TanStack Query should retry on this error.
 * Don't retry 401 (auth) or 4xx client errors.
 */
export function shouldRetry(failureCount: number, error: unknown): boolean {
  if (isApiError(error)) {
    if (error.status === 401 || (error.status >= 400 && error.status < 500)) {
      return false
    }
  }
  return failureCount < 2
}
