import { describe, it, expect, vi, beforeEach } from 'vitest'
import { classifyError, fetchApi, shouldRetry, type ApiError } from './api-errors'

describe('classifyError', () => {
  it('classifies 401 as auth_expired', () => {
    const err: ApiError = { status: 401, message: 'Unauthorized' }
    expect(classifyError(err)).toEqual({ type: 'auth_expired' })
  })

  it('classifies 404 as not_found', () => {
    const err: ApiError = { status: 404, message: 'Not found' }
    expect(classifyError(err)).toEqual({ type: 'not_found', message: 'Not found' })
  })

  it('classifies 400 as validation', () => {
    const err: ApiError = { status: 400, message: 'Bad input' }
    expect(classifyError(err)).toEqual({ type: 'validation', message: 'Bad input' })
  })

  it('classifies 422 as validation', () => {
    const err: ApiError = { status: 422, message: 'Invalid field' }
    expect(classifyError(err)).toEqual({ type: 'validation', message: 'Invalid field' })
  })

  it('classifies 500 as server_error', () => {
    const err: ApiError = { status: 500, message: 'Internal error' }
    expect(classifyError(err)).toEqual({ type: 'server_error', message: 'Internal error' })
  })

  it('classifies TypeError with "fetch" as offline', () => {
    const err = new TypeError('Failed to fetch')
    expect(classifyError(err)).toEqual({ type: 'offline' })
  })

  it('classifies unknown errors as server_error', () => {
    expect(classifyError('random string')).toEqual({
      type: 'server_error',
      message: 'An unexpected error occurred.',
    })
  })
})

describe('shouldRetry', () => {
  it('does not retry on 401', () => {
    const err: ApiError = { status: 401, message: 'Unauthorized' }
    expect(shouldRetry(0, err)).toBe(false)
  })

  it('does not retry on 400', () => {
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

describe('fetchApi', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('returns parsed JSON on success', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ data: 'test' }),
    }))

    const result = await fetchApi<{ data: string }>('/api/test')
    expect(result).toEqual({ data: 'test' })
  })

  it('throws ApiError on 401', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      status: 401,
      statusText: 'Unauthorized',
      json: () => Promise.resolve({ error: 'Token expired' }),
    }))

    await expect(fetchApi('/api/test')).rejects.toMatchObject({
      status: 401,
      message: 'Token expired',
    })
  })

  it('throws ApiError on 500 with no error body', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      status: 500,
      statusText: 'Internal Server Error',
      json: () => Promise.reject(new Error('not json')),
    }))

    await expect(fetchApi('/api/test')).rejects.toMatchObject({
      status: 500,
      message: 'Internal Server Error',
    })
  })

  it('throws on network failure', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')))

    await expect(fetchApi('/api/test')).rejects.toMatchObject({
      status: 0,
      code: 'NETWORK_ERROR',
    })
  })
})
