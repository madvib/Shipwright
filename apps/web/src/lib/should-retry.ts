import type { ApiError } from './api-errors'

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
