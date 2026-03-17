import { describe, it, expect } from 'vitest'
import { mapJobRowFull, TARGET_NAMES } from './jobs'

// ── Tests ──────────────────────────────────────────────────────────────────

describe('mapJobRowFull', () => {
  it('extracts description from payload_json', () => {
    const result = mapJobRowFull({
      id: 'abc123',
      kind: 'feature',
      status: 'running',
      branch: 'job/abc123',
      payload_json: JSON.stringify({ description: 'Build feature X' }),
      log_entries: [],
    })
    expect(result.description).toBe('Build feature X')
  })

  it('returns empty description when payload has no description field', () => {
    const result = mapJobRowFull({
      id: 'abc123',
      kind: 'feature',
      status: 'pending',
      branch: 'job/abc123',
      payload_json: JSON.stringify({ other: 'data' }),
      log_entries: [],
    })
    expect(result.description).toBe('')
  })

  it('returns empty description when payload_json is malformed', () => {
    const result = mapJobRowFull({
      id: 'abc123',
      kind: 'feature',
      status: 'pending',
      branch: null,
      payload_json: 'not-json',
      log_entries: [],
    })
    expect(result.description).toBe('')
  })

  it('falls back to job/id when branch is null', () => {
    const result = mapJobRowFull({
      id: 'xyz999',
      kind: 'feature',
      status: 'pending',
      branch: null,
      payload_json: '{}',
      log_entries: [],
    })
    expect(result.branch).toBe('job/xyz999')
  })

  it('uses provided branch when present', () => {
    const result = mapJobRowFull({
      id: 'xyz999',
      kind: 'feature',
      status: 'running',
      branch: 'feat/my-feature',
      payload_json: '{}',
      log_entries: [],
    })
    expect(result.branch).toBe('feat/my-feature')
  })

  it('computes worktree_path from id', () => {
    const result = mapJobRowFull({
      id: 'CDPh4Tc2',
      kind: 'feature',
      status: 'running',
      branch: null,
      payload_json: '{}',
      log_entries: [],
    })
    expect(result.worktree_path).toBe('~/dev/ship-worktrees/CDPh4Tc2')
  })

  it('preserves status field', () => {
    const pending = mapJobRowFull({ id: 'a', kind: 'f', status: 'pending', branch: null, payload_json: '{}', log_entries: [] })
    const running = mapJobRowFull({ id: 'b', kind: 'f', status: 'running', branch: null, payload_json: '{}', log_entries: [] })
    expect(pending.status).toBe('pending')
    expect(running.status).toBe('running')
  })

  it('extracts capability_id from payload', () => {
    const result = mapJobRowFull({
      id: 'abc',
      kind: 'feature',
      status: 'running',
      branch: null,
      payload_json: JSON.stringify({ capability_id: 'cap001' }),
      log_entries: [],
    })
    expect(result.capability_id).toBe('cap001')
  })

  it('returns null capability_id when absent', () => {
    const result = mapJobRowFull({ id: 'a', kind: 'f', status: 'pending', branch: null, payload_json: '{}', log_entries: [] })
    expect(result.capability_id).toBeNull()
  })

  it('extracts scope array from payload', () => {
    const result = mapJobRowFull({
      id: 'abc',
      kind: 'feature',
      status: 'pending',
      branch: null,
      payload_json: JSON.stringify({ scope: ['src/lib.rs', 'src/main.rs'] }),
      log_entries: [],
    })
    expect(result.scope).toEqual(['src/lib.rs', 'src/main.rs'])
  })

  it('returns empty scope array when absent', () => {
    const result = mapJobRowFull({ id: 'a', kind: 'f', status: 'pending', branch: null, payload_json: '{}', log_entries: [] })
    expect(result.scope).toEqual([])
  })

  it('extracts acceptance_criteria array from payload', () => {
    const result = mapJobRowFull({
      id: 'abc',
      kind: 'feature',
      status: 'pending',
      branch: null,
      payload_json: JSON.stringify({ acceptance_criteria: ['Tests pass', 'Docs updated'] }),
      log_entries: [],
    })
    expect(result.acceptance_criteria).toEqual(['Tests pass', 'Docs updated'])
  })

  it('extracts symlink_name from payload', () => {
    const result = mapJobRowFull({
      id: 'abc',
      kind: 'feature',
      status: 'running',
      branch: null,
      payload_json: JSON.stringify({ symlink_name: 'my-feature' }),
      log_entries: [],
    })
    expect(result.symlink_name).toBe('my-feature')
  })

  it('returns null symlink_name when absent', () => {
    const result = mapJobRowFull({ id: 'a', kind: 'f', status: 'pending', branch: null, payload_json: '{}', log_entries: [] })
    expect(result.symlink_name).toBeNull()
  })

  it('attaches log_entries unchanged', () => {
    const logs = [{ id: 1, message: 'hello', created_at: '2026-03-17T10:00:00Z' }]
    const result = mapJobRowFull({ id: 'a', kind: 'f', status: 'running', branch: null, payload_json: '{}', log_entries: logs })
    expect(result.log_entries).toEqual(logs)
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
  interface CapabilityRow {
    id: string
    title: string
    target_id: string
    status: 'aspirational' | 'actual'
    evidence: string | null
    running_job_id: string | null
    running_job_desc: string | null
  }

  function groupByTarget(caps: CapabilityRow[]): Record<string, CapabilityRow[]> {
    return caps.reduce<Record<string, CapabilityRow[]>>((acc, cap) => {
      ;(acc[cap.target_id] ??= []).push(cap)
      return acc
    }, {})
  }

  function makeCapRow(id: string, target_id: string): CapabilityRow {
    return { id, title: `Cap ${id}`, target_id, status: 'aspirational', evidence: null, running_job_id: null, running_job_desc: null }
  }

  it('groups capabilities by target_id', () => {
    const caps = [
      makeCapRow('1', 'ShYqMr8e'),
      makeCapRow('2', 'ShYqMr8e'),
      makeCapRow('3', 'gbJkmuwY'),
    ]
    const grouped = groupByTarget(caps)
    expect(grouped['ShYqMr8e']).toHaveLength(2)
    expect(grouped['gbJkmuwY']).toHaveLength(1)
  })

  it('returns empty object for empty capabilities array', () => {
    expect(groupByTarget([])).toEqual({})
  })

  it('counts actual vs aspirational correctly', () => {
    const caps: CapabilityRow[] = [
      { ...makeCapRow('1', 'ShYqMr8e'), status: 'actual', evidence: 'test_ok' },
      makeCapRow('2', 'ShYqMr8e'),
    ]
    const grouped = groupByTarget(caps)
    expect(grouped['ShYqMr8e'].filter((c) => c.status === 'actual')).toHaveLength(1)
    expect(grouped['ShYqMr8e'].filter((c) => c.status === 'aspirational')).toHaveLength(1)
  })
})
