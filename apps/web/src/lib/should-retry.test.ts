import { describe, it, expect } from 'vitest'
import { shouldRetry } from './should-retry'
import type { ApiError } from './api-errors'

describe('shouldRetry', () => {
  it('does not retry on 401', () => {
    const err: ApiError = { status: 401, message: 'Unauthorized' }
    expect(shouldRetry(0, err)).toBe(false)
  })

  it('does not retry on 4xx client errors', () => {
    const err: ApiError = { status: 400, message: 'Bad request' }
    expect(shouldRetry(0, err)).toBe(false)
  })

  it('retries on 500 up to 2 times', () => {
    const err: ApiError = { status: 500, message: 'Server error' }
    expect(shouldRetry(0, err)).toBe(true)
    expect(shouldRetry(1, err)).toBe(true)
    expect(shouldRetry(2, err)).toBe(false)
  })

  it('retries unknown errors up to 2 times', () => {
    expect(shouldRetry(0, new Error('oops'))).toBe(true)
    expect(shouldRetry(2, new Error('oops'))).toBe(false)
  })
})
