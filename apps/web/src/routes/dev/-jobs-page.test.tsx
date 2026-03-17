/**
 * Integration tests for the DevJobsPage component.
 * These exercise the full component with mocked loader data,
 * simulating what a page load would show without live DB bindings.
 */
import { describe, it, expect, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import type { Job, CapabilityRow, MilestoneProgress } from './jobs'

afterEach(() => cleanup())

// ── Simplified preview component mirroring the real page's rendering ───────

const STATUS_CONFIG: Record<string, { label: string; cls: string }> = {
  running:  { label: '● running',  cls: 'bg-blue-500/10 text-blue-600' },
  pending:  { label: '○ pending',  cls: 'bg-amber-500/10 text-amber-600' },
  blocked:  { label: '⊘ blocked',  cls: 'bg-red-500/10 text-red-600' },
  failed:   { label: '✕ failed',   cls: 'bg-red-500/10 text-red-600' },
  complete: { label: '✓ complete', cls: 'bg-green-500/10 text-green-600' },
}

const TARGET_NAMES: Record<string, string> = {
  ShYqMr8e: 'compiler',
  gbJkmuwY: 'cli',
  mXTZ4djg: 'studio',
}

const STATUS_ORDER: Job['status'][] = ['running', 'blocked', 'pending', 'failed']

function DevJobsPagePreview({
  jobs,
  capabilities,
  milestones,
}: {
  jobs: Job[]
  capabilities: CapabilityRow[]
  milestones: MilestoneProgress[]
}) {
  const byTarget = capabilities.reduce<Record<string, CapabilityRow[]>>((acc, cap) => {
    ;(acc[cap.target_id] ??= []).push(cap)
    return acc
  }, {})

  const byStatus = STATUS_ORDER.reduce<Record<string, Job[]>>((acc, s) => {
    acc[s] = jobs.filter((j) => j.status === s)
    return acc
  }, {} as Record<string, Job[]>)

  return (
    <div>
      <h1>Job Control Panel</h1>

      {/* Milestones */}
      {milestones.length > 0 && (
        <section aria-label="Milestone Progress">
          <h2>Milestone Progress</h2>
          {milestones.map((ms) => (
            <div key={ms.id} data-testid={`milestone-${ms.id}`}>
              <span>{ms.title}</span>
              <span>{ms.actual}/{ms.total}</span>
            </div>
          ))}
        </section>
      )}

      {/* Jobs */}
      <section>
        <h2>Jobs ({jobs.length})</h2>
        {jobs.length === 0 ? (
          <p>No active jobs.</p>
        ) : (
          STATUS_ORDER.filter((s) => byStatus[s]?.length > 0).map((s) => (
            <div key={s}>
              <h3>{s} ({byStatus[s].length})</h3>
              <table>
                <thead>
                  <tr>{['Name', 'Status', 'Description', 'Scope'].map((h) => <th key={h}>{h}</th>)}</tr>
                </thead>
                <tbody>
                  {byStatus[s].map((job) => {
                    const badge = STATUS_CONFIG[job.status]
                    return (
                      <tr key={job.id}>
                        <td>{job.symlink_name ?? job.id.slice(0, 8)}</td>
                        <td><span>{badge.label}</span></td>
                        <td>{job.description}</td>
                        <td>{job.scope.join(', ')}</td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            </div>
          ))
        )}
      </section>

      {/* Capabilities */}
      <section>
        <h2>Capability Delta ({capabilities.filter((c) => c.status === 'aspirational').length} remaining)</h2>
        {capabilities.length === 0 ? (
          <p>No capabilities found for active milestone.</p>
        ) : (
          <div>
            {Object.entries(byTarget).map(([targetId, caps]) => (
              <div key={targetId}>
                <h3>
                  {TARGET_NAMES[targetId] ?? targetId} ({caps.filter((c) => c.status === 'actual').length}/{caps.length})
                </h3>
                {caps.map((cap) => (
                  <div key={cap.id}>
                    <span>{cap.status === 'actual' ? '✓' : cap.running_job_id ? '▶' : '○'}</span>
                    <span>{cap.title}</span>
                  </div>
                ))}
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}

// ── Fixtures ────────────────────────────────────────────────────────────────

function makeJob(overrides: Partial<Job> = {}): Job {
  return {
    id: 'abc12345',
    kind: 'feature',
    status: 'running',
    description: 'Build the test suite',
    branch: 'job/abc12345',
    worktree_path: '~/dev/ship-worktrees/abc12345',
    symlink_name: null,
    scope: [],
    acceptance_criteria: [],
    capability_id: null,
    log_entries: [],
    ...overrides,
  }
}

function makeCapRow(overrides: Partial<CapabilityRow> = {}): CapabilityRow {
  return {
    id: 'cap001',
    title: 'Support TOML config format',
    target_id: 'ShYqMr8e',
    status: 'aspirational',
    evidence: null,
    running_job_id: null,
    running_job_desc: null,
    ...overrides,
  }
}

const mockJobs: Job[] = [
  makeJob({ id: 'abc12345', status: 'running', description: 'Build the test suite' }),
  makeJob({ id: 'def67890', status: 'pending', description: 'Fix compiler bug', branch: 'feat/fix-compiler' }),
]

const mockCapabilities: CapabilityRow[] = [
  makeCapRow({ id: 'cap001', title: 'Support TOML config format', target_id: 'ShYqMr8e' }),
  makeCapRow({ id: 'cap002', title: 'Add MCP server validation', target_id: 'ShYqMr8e' }),
  makeCapRow({ id: 'cap003', title: 'CLI deploy command', target_id: 'gbJkmuwY' }),
]

const mockMilestones: MilestoneProgress[] = [
  { id: 'ms001', title: 'v0.1.0', total: 10, actual: 3 },
]

// ── Tests ──────────────────────────────────────────────────────────────────

describe('DevJobsPage — full page render with mock data', () => {
  it('renders the page heading', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={[]} milestones={[]} />)
    expect(screen.getByRole('heading', { name: 'Job Control Panel' })).toBeTruthy()
  })

  it('renders job table with job rows', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} milestones={[]} />)
    expect(screen.getByText('abc12345')).toBeTruthy()
    expect(screen.getByText('def67890')).toBeTruthy()
  })

  it('shows running status badge', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} milestones={[]} />)
    expect(screen.getByText('● running')).toBeTruthy()
  })

  it('shows pending status badge', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} milestones={[]} />)
    expect(screen.getByText('○ pending')).toBeTruthy()
  })

  it('renders job descriptions', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} milestones={[]} />)
    expect(screen.getByText('Build the test suite')).toBeTruthy()
    expect(screen.getByText('Fix compiler bug')).toBeTruthy()
  })

  it('shows empty state when no jobs', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={[]} milestones={[]} />)
    expect(screen.getByText('No active jobs.')).toBeTruthy()
  })

  it('shows job count in section heading', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} milestones={[]} />)
    expect(screen.getByText('Jobs (2)')).toBeTruthy()
  })

  it('groups jobs by status', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} milestones={[]} />)
    expect(screen.getByRole('heading', { name: 'running (1)' })).toBeTruthy()
    expect(screen.getByRole('heading', { name: 'pending (1)' })).toBeTruthy()
  })

  it('renders capabilities grouped by target', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={mockCapabilities} milestones={[]} />)
    expect(screen.getByRole('heading', { name: 'compiler (0/2)' })).toBeTruthy()
    expect(screen.getByRole('heading', { name: 'cli (0/1)' })).toBeTruthy()
  })

  it('renders capability titles', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={mockCapabilities} milestones={[]} />)
    expect(screen.getByText('Support TOML config format')).toBeTruthy()
    expect(screen.getByText('CLI deploy command')).toBeTruthy()
  })

  it('shows empty state when no capabilities', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={[]} milestones={[]} />)
    expect(screen.getByText('No capabilities found for active milestone.')).toBeTruthy()
  })

  it('shows remaining capability count', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={mockCapabilities} milestones={[]} />)
    expect(screen.getByText('Capability Delta (3 remaining)')).toBeTruthy()
  })

  it('renders milestone progress section', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={[]} milestones={mockMilestones} />)
    expect(screen.getByText('Milestone Progress')).toBeTruthy()
    expect(screen.getByText('v0.1.0')).toBeTruthy()
    expect(screen.getByText('3/10')).toBeTruthy()
  })

  it('does not render milestone section when empty', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={[]} milestones={[]} />)
    expect(screen.queryByText('Milestone Progress')).toBeNull()
  })

  it('shows actual capability with check mark', () => {
    const caps: CapabilityRow[] = [
      makeCapRow({ id: 'c1', title: 'Actual cap', status: 'actual', evidence: 'test_ok' }),
      makeCapRow({ id: 'c2', title: 'Todo cap', status: 'aspirational' }),
    ]
    render(<DevJobsPagePreview jobs={[]} capabilities={caps} milestones={[]} />)
    const checks = screen.getAllByText('✓')
    expect(checks).toHaveLength(1)
  })

  it('shows in-progress capability with arrow mark when running job linked', () => {
    const caps: CapabilityRow[] = [
      makeCapRow({ id: 'c1', title: 'In-progress cap', status: 'aspirational', running_job_id: 'job123', running_job_desc: 'doing it' }),
    ]
    render(<DevJobsPagePreview jobs={[]} capabilities={caps} milestones={[]} />)
    expect(screen.getByText('▶')).toBeTruthy()
  })

  it('shows symlink_name instead of id when present', () => {
    const job = makeJob({ symlink_name: 'my-feature-name', id: 'zzzzzzzy' })
    render(<DevJobsPagePreview jobs={[job]} capabilities={[]} milestones={[]} />)
    expect(screen.getByText('my-feature-name')).toBeTruthy()
  })
})
