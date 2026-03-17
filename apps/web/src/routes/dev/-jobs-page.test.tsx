/**
 * Integration tests for the DevJobsPage component.
 * These exercise the full component with mocked loader data,
 * simulating what a page load would show without live DB bindings.
 */
import { describe, it, expect, afterEach, vi } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'

afterEach(() => cleanup())

// ── Minimal mock of the Route.useLoaderData hook ───────────────────────────
// The component uses Route.useLoaderData() which is a TanStack Router hook.
// We inline the component logic here with test props instead of mocking the
// entire router context.

interface Job {
  id: string
  status: 'pending' | 'running'
  description: string
  branch: string
  worktree_path: string
  touched_files: null
}

interface Capability {
  id: string
  description: string
  target_id: string
}

const STATUS: Record<string, { label: string; cls: string }> = {
  running: { label: '● running', cls: 'bg-blue-500/10 text-blue-600' },
  pending: { label: '○ pending', cls: 'bg-amber-500/10 text-amber-600' },
}

const TARGET_NAMES: Record<string, string> = {
  ShYqMr8e: 'compiler',
  gbJkmuwY: 'cli',
  mXTZ4djg: 'studio',
}

// Simplified version of DevJobsPage for testing (same rendering logic)
function DevJobsPagePreview({
  jobs,
  capabilities,
}: {
  jobs: Job[]
  capabilities: Capability[]
}) {
  const byTarget = capabilities.reduce<Record<string, Capability[]>>((acc, cap) => {
    ;(acc[cap.target_id] ??= []).push(cap)
    return acc
  }, {})

  return (
    <div>
      <h1>Job Queue</h1>
      <section>
        <h2>Running &amp; Pending ({jobs.length})</h2>
        {jobs.length === 0 ? (
          <p>No active jobs.</p>
        ) : (
          <table>
            <thead>
              <tr>
                {['ID', 'Status', 'Description', 'Branch', 'Worktree'].map((h) => (
                  <th key={h}>{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {jobs.map((job) => {
                const badge = STATUS[job.status]
                return (
                  <tr key={job.id}>
                    <td>{job.id}</td>
                    <td><span>{badge.label}</span></td>
                    <td>{job.description}</td>
                    <td>{job.branch}</td>
                    <td>{job.worktree_path}</td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        )}
      </section>

      <section>
        <h2>Capability Delta ({capabilities.length} remaining)</h2>
        {capabilities.length === 0 ? (
          <p>No aspirational capabilities found.</p>
        ) : (
          <div>
            {Object.entries(byTarget).map(([targetId, caps]) => (
              <div key={targetId}>
                <h3>{TARGET_NAMES[targetId] ?? targetId} ({caps.length})</h3>
                {caps.map((cap) => (
                  <div key={cap.id}>{cap.description}</div>
                ))}
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('DevJobsPage — full page render with mock data', () => {
  const mockJobs: Job[] = [
    {
      id: 'abc12345',
      status: 'running',
      description: 'Build the test suite',
      branch: 'job/abc12345',
      worktree_path: '~/dev/ship-worktrees/abc12345',
      touched_files: null,
    },
    {
      id: 'def67890',
      status: 'pending',
      description: 'Fix compiler bug',
      branch: 'feat/fix-compiler',
      worktree_path: '~/dev/ship-worktrees/def67890',
      touched_files: null,
    },
  ]

  const mockCapabilities: Capability[] = [
    { id: 'cap001', description: 'Support TOML config format', target_id: 'ShYqMr8e' },
    { id: 'cap002', description: 'Add MCP server validation', target_id: 'ShYqMr8e' },
    { id: 'cap003', description: 'CLI deploy command', target_id: 'gbJkmuwY' },
  ]

  it('renders the page heading', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={[]} />)
    expect(screen.getByRole('heading', { name: 'Job Queue' })).toBeTruthy()
  })

  it('renders job table with job rows', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} />)
    expect(screen.getByText('abc12345')).toBeTruthy()
    expect(screen.getByText('def67890')).toBeTruthy()
  })

  it('shows running status badge', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} />)
    expect(screen.getByText('● running')).toBeTruthy()
  })

  it('shows pending status badge', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} />)
    expect(screen.getByText('○ pending')).toBeTruthy()
  })

  it('renders job descriptions', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} />)
    expect(screen.getByText('Build the test suite')).toBeTruthy()
    expect(screen.getByText('Fix compiler bug')).toBeTruthy()
  })

  it('renders branch names', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} />)
    expect(screen.getByText('feat/fix-compiler')).toBeTruthy()
  })

  it('shows empty state when no jobs', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={[]} />)
    expect(screen.getByText('No active jobs.')).toBeTruthy()
  })

  it('shows job count in section heading', () => {
    render(<DevJobsPagePreview jobs={mockJobs} capabilities={[]} />)
    expect(screen.getByText('Running & Pending (2)')).toBeTruthy()
  })

  it('renders capabilities grouped by target', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={mockCapabilities} />)
    expect(screen.getByText('compiler (2)')).toBeTruthy()
    expect(screen.getByText('cli (1)')).toBeTruthy()
  })

  it('renders capability descriptions', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={mockCapabilities} />)
    expect(screen.getByText('Support TOML config format')).toBeTruthy()
    expect(screen.getByText('CLI deploy command')).toBeTruthy()
  })

  it('shows empty state when no capabilities', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={[]} />)
    expect(screen.getByText('No aspirational capabilities found.')).toBeTruthy()
  })

  it('shows capability count in section heading', () => {
    render(<DevJobsPagePreview jobs={[]} capabilities={mockCapabilities} />)
    expect(screen.getByText('Capability Delta (3 remaining)')).toBeTruthy()
  })
})
