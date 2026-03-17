import { describe, it, expect } from 'vitest'

// ── Extracted pure logic from jobs.tsx for unit testing ────────────────────
// These mirror the transformations in the route's server functions.

interface RawRow {
  id: string
  status: string
  branch: string | null
  payload_json: string
}

function mapJobRow(row: RawRow) {
  let description = ''
  try {
    const payload = JSON.parse(row.payload_json) as Record<string, unknown>
    if (typeof payload.description === 'string') description = payload.description
  } catch { /* ignore */ }
  return {
    id: row.id,
    status: row.status as 'pending' | 'running',
    description,
    branch: row.branch ?? `job/${row.id}`,
    worktree_path: `~/dev/ship-worktrees/${row.id}`,
    touched_files: null,
  }
}

const TARGET_NAMES: Record<string, string> = {
  ShYqMr8e: 'compiler',
  gbJkmuwY: 'cli',
  mXTZ4djg: 'studio',
  MeHin6Fy: 'platform',
  iu86rzHS: 'mcp',
  zU6Leq6v: 'infra',
  JUPHSmmW: 'desktop',
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('mapJobRow', () => {
  it('extracts description from payload_json', () => {
    const row: RawRow = {
      id: 'abc123',
      status: 'running',
      branch: 'job/abc123',
      payload_json: JSON.stringify({ description: 'Build feature X' }),
    }
    expect(mapJobRow(row).description).toBe('Build feature X')
  })

  it('returns empty description when payload has no description field', () => {
    const row: RawRow = {
      id: 'abc123',
      status: 'pending',
      branch: 'job/abc123',
      payload_json: JSON.stringify({ other: 'data' }),
    }
    expect(mapJobRow(row).description).toBe('')
  })

  it('returns empty description when payload_json is malformed', () => {
    const row: RawRow = {
      id: 'abc123',
      status: 'pending',
      branch: null,
      payload_json: 'not-json',
    }
    expect(mapJobRow(row).description).toBe('')
  })

  it('falls back to job/id when branch is null', () => {
    const row: RawRow = {
      id: 'xyz999',
      status: 'pending',
      branch: null,
      payload_json: '{}',
    }
    expect(mapJobRow(row).branch).toBe('job/xyz999')
  })

  it('uses provided branch when present', () => {
    const row: RawRow = {
      id: 'xyz999',
      status: 'running',
      branch: 'feat/my-feature',
      payload_json: '{}',
    }
    expect(mapJobRow(row).branch).toBe('feat/my-feature')
  })

  it('computes worktree_path from id', () => {
    const row: RawRow = { id: 'CDPh4Tc2', status: 'running', branch: null, payload_json: '{}' }
    expect(mapJobRow(row).worktree_path).toBe('~/dev/ship-worktrees/CDPh4Tc2')
  })

  it('always sets touched_files to null', () => {
    const row: RawRow = { id: 'a', status: 'pending', branch: null, payload_json: '{}' }
    expect(mapJobRow(row).touched_files).toBeNull()
  })

  it('preserves status field', () => {
    const pending: RawRow = { id: 'a', status: 'pending', branch: null, payload_json: '{}' }
    const running: RawRow = { id: 'b', status: 'running', branch: null, payload_json: '{}' }
    expect(mapJobRow(pending).status).toBe('pending')
    expect(mapJobRow(running).status).toBe('running')
  })
})

describe('TARGET_NAMES', () => {
  it('maps known target ids to names', () => {
    expect(TARGET_NAMES['ShYqMr8e']).toBe('compiler')
    expect(TARGET_NAMES['gbJkmuwY']).toBe('cli')
    expect(TARGET_NAMES['mXTZ4djg']).toBe('studio')
    expect(TARGET_NAMES['MeHin6Fy']).toBe('platform')
    expect(TARGET_NAMES['iu86rzHS']).toBe('mcp')
    expect(TARGET_NAMES['zU6Leq6v']).toBe('infra')
    expect(TARGET_NAMES['JUPHSmmW']).toBe('desktop')
  })

  it('returns undefined for unknown target id', () => {
    expect(TARGET_NAMES['unknown-id']).toBeUndefined()
  })
})

describe('capability grouping by target', () => {
  interface Capability {
    id: string
    description: string
    target_id: string
  }

  function groupByTarget(caps: Capability[]): Record<string, Capability[]> {
    return caps.reduce<Record<string, Capability[]>>((acc, cap) => {
      ;(acc[cap.target_id] ??= []).push(cap)
      return acc
    }, {})
  }

  it('groups capabilities by target_id', () => {
    const caps: Capability[] = [
      { id: '1', description: 'Cap A', target_id: 'ShYqMr8e' },
      { id: '2', description: 'Cap B', target_id: 'ShYqMr8e' },
      { id: '3', description: 'Cap C', target_id: 'gbJkmuwY' },
    ]
    const grouped = groupByTarget(caps)
    expect(grouped['ShYqMr8e']).toHaveLength(2)
    expect(grouped['gbJkmuwY']).toHaveLength(1)
  })

  it('returns empty object for empty capabilities array', () => {
    expect(groupByTarget([])).toEqual({})
  })
})
