import { createFileRoute, redirect, useRouter } from '@tanstack/react-router'
import { createServerFn } from '@tanstack/react-start'
import { useState } from 'react'
import { RefreshCw, ChevronDown, ChevronRight } from 'lucide-react'

// ── Types ─────────────────────────────────────────────────────────────────

export interface JobPayload {
  description?: string
  capability_id?: string
  scope?: string[]
  acceptance_criteria?: string[]
  preset_hint?: string
  symlink_name?: string
  [key: string]: unknown
}

export interface Job {
  id: string
  kind: string
  status: 'pending' | 'running' | 'blocked' | 'complete' | 'failed'
  description: string
  branch: string
  worktree_path: string
  symlink_name: string | null
  scope: string[]
  acceptance_criteria: string[]
  capability_id: string | null
  log_entries: LogEntry[]
}

export interface LogEntry {
  id: number
  message: string
  created_at: string
}

export interface CapabilityRow {
  id: string
  title: string
  target_id: string
  status: 'aspirational' | 'actual'
  evidence: string | null
  running_job_id: string | null
  running_job_desc: string | null
}

export interface MilestoneProgress {
  id: string
  title: string
  total: number
  actual: number
}

// ── Target name map (stable — from list_targets) ──────────────────────────

export const TARGET_NAMES: Record<string, string> = {
  ShYqMr8e: 'compiler',
  gbJkmuwY: 'cli',
  mXTZ4djg: 'studio',
  MeHin6Fy: 'platform',
  iu86rzHS: 'mcp',
  zU6Leq6v: 'infra',
  JUPHSmmW: 'desktop',
}

// ── DB path ───────────────────────────────────────────────────────────────

function getDbPath(): string {
  const { homedir } = require('os') as typeof import('os')
  const home = process.env.HOME ?? homedir()
  return `${home}/.ship/state/ship-hrvmuz4p/platform.db`
}

async function openDb() {
  const fs = require('fs') as typeof import('fs')
  const initSqlJs = (await import('sql.js')).default
  const SQL = await initSqlJs()
  const buffer = fs.readFileSync(getDbPath())
  return new SQL.Database(buffer)
}

// ── Data loading helpers ──────────────────────────────────────────────────

/** Map a raw job DB row into a typed Job with all payload fields extracted. */
export function mapJobRowFull(row: {
  id: string
  kind: string
  status: string
  branch: string | null
  payload_json: string
  log_entries: LogEntry[]
}): Job {
  let payload: JobPayload = {}
  try {
    payload = JSON.parse(row.payload_json) as JobPayload
  } catch { /* ignore */ }

  return {
    id: row.id,
    kind: row.kind,
    status: row.status as Job['status'],
    description: typeof payload.description === 'string' ? payload.description : '',
    branch: row.branch ?? `job/${row.id}`,
    worktree_path: `~/dev/ship-worktrees/${row.id}`,
    symlink_name: typeof payload.symlink_name === 'string' ? payload.symlink_name : null,
    scope: Array.isArray(payload.scope) ? (payload.scope as string[]) : [],
    acceptance_criteria: Array.isArray(payload.acceptance_criteria)
      ? (payload.acceptance_criteria as string[])
      : [],
    capability_id: typeof payload.capability_id === 'string' ? payload.capability_id : null,
    log_entries: row.log_entries,
  }
}

// ── Server functions ──────────────────────────────────────────────────────

const loadDashboard = createServerFn({ method: 'GET' }).handler(async (): Promise<{
  jobs: Job[]
  milestones: MilestoneProgress[]
  capabilities: CapabilityRow[]
}> => {
  try {
    const db = await openDb()

    // Load jobs — all non-archived statuses
    const jobStmt = db.prepare(
      "SELECT id, kind, status, branch, payload_json FROM job WHERE status IN ('running', 'pending', 'blocked', 'failed') ORDER BY CASE status WHEN 'running' THEN 0 WHEN 'blocked' THEN 1 WHEN 'pending' THEN 2 ELSE 3 END, created_at DESC"
    )
    const jobRows: { id: string; kind: string; status: string; branch: string | null; payload_json: string }[] = []
    while (jobStmt.step()) {
      const r = jobStmt.getAsObject() as typeof jobRows[0]
      jobRows.push(r)
    }
    jobStmt.free()

    // Load last 5 log entries per job
    const logStmt = db.prepare(
      'SELECT job_id, id, message, created_at FROM job_log WHERE job_id IS NOT NULL ORDER BY created_at DESC'
    )
    const allLogs: { job_id: string; id: number; message: string; created_at: string }[] = []
    while (logStmt.step()) {
      const r = logStmt.getAsObject() as typeof allLogs[0]
      allLogs.push(r)
    }
    logStmt.free()

    // Group logs by job_id, keep last 5
    const logsByJob: Record<string, LogEntry[]> = {}
    for (const log of allLogs) {
      const jid = String(log.job_id)
      if (!logsByJob[jid]) logsByJob[jid] = []
      if (logsByJob[jid].length < 5) {
        logsByJob[jid].push({ id: Number(log.id), message: log.message, created_at: log.created_at })
      }
    }

    const jobs = jobRows.map((row) =>
      mapJobRowFull({ ...row, log_entries: logsByJob[row.id] ?? [] })
    )

    // Load milestones + capabilities
    const msStmt = db.prepare("SELECT id, title FROM target WHERE kind = 'milestone' AND status = 'active' ORDER BY created_at")
    const milestoneRows: { id: string; title: string }[] = []
    while (msStmt.step()) {
      const r = msStmt.getAsObject() as { id: string; title: string }
      milestoneRows.push(r)
    }
    msStmt.free()

    const capStmt = db.prepare(
      'SELECT id, target_id, title, status, evidence, milestone_id FROM capability ORDER BY target_id, created_at'
    )
    const capRows: { id: string; target_id: string; title: string; status: string; evidence: string | null; milestone_id: string | null }[] = []
    while (capStmt.step()) {
      const r = capStmt.getAsObject() as typeof capRows[0]
      capRows.push(r)
    }
    capStmt.free()
    db.close()

    // Build running-job lookup by capability_id
    const runningByCapId: Record<string, Job> = {}
    for (const job of jobs) {
      if (job.capability_id && job.status === 'running') {
        runningByCapId[job.capability_id] = job
      }
    }

    // Use first active milestone (hardcoded v0.1.0 id Gext6Bgu or first found)
    const activeMilestone = milestoneRows[0]

    const milestones: MilestoneProgress[] = milestoneRows.map((ms) => {
      const linked = capRows.filter((c) => c.milestone_id === ms.id)
      return {
        id: ms.id,
        title: ms.title,
        total: linked.length,
        actual: linked.filter((c) => c.status === 'actual').length,
      }
    })

    const capabilities: CapabilityRow[] = activeMilestone
      ? capRows
          .filter((c) => c.milestone_id === activeMilestone.id)
          .map((c) => {
            const rj = runningByCapId[c.id]
            return {
              id: c.id,
              title: c.title,
              target_id: c.target_id,
              status: c.status as CapabilityRow['status'],
              evidence: c.evidence,
              running_job_id: rj?.id ?? null,
              running_job_desc: rj?.description ?? null,
            }
          })
      : []

    return { jobs, milestones, capabilities }
  } catch (e) {
    console.error('[loadDashboard]', e)
    return { jobs: [], milestones: [], capabilities: [] }
  }
})

// ── Route ─────────────────────────────────────────────────────────────────

export const Route = createFileRoute('/dev/jobs')({
  beforeLoad: () => {
    if (import.meta.env.PROD) throw redirect({ to: '/' })
  },
  loader: () => loadDashboard(),
  component: DevJobsPage,
})

// ── Status config ─────────────────────────────────────────────────────────

const STATUS_CONFIG: Record<string, { label: string; cls: string }> = {
  running:  { label: '● running',  cls: 'bg-blue-500/10 text-blue-600 dark:text-blue-400' },
  pending:  { label: '○ pending',  cls: 'bg-amber-500/10 text-amber-600 dark:text-amber-400' },
  blocked:  { label: '⊘ blocked',  cls: 'bg-red-500/10 text-red-600 dark:text-red-400' },
  failed:   { label: '✕ failed',   cls: 'bg-red-500/10 text-red-600 dark:text-red-400' },
  complete: { label: '✓ complete', cls: 'bg-green-500/10 text-green-600 dark:text-green-400' },
}

const STATUS_ORDER: Job['status'][] = ['running', 'blocked', 'pending', 'failed']

// ── Sub-components ────────────────────────────────────────────────────────

function ProgressBar({ value, max }: { value: number; max: number }) {
  const pct = max === 0 ? 0 : Math.round((value / max) * 100)
  return (
    <div className="mt-1 h-1.5 w-full overflow-hidden rounded-full bg-muted/40">
      <div
        className="h-full rounded-full bg-green-500 transition-all"
        style={{ width: `${pct}%` }}
      />
    </div>
  )
}

function ScopeBadge({ path }: { path: string }) {
  const short = path.replace(/^(apps|crates|packages)\//, '').slice(0, 24)
  return (
    <span className="rounded border border-border/50 bg-muted/30 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
      {short}
    </span>
  )
}

function JobRow({ job }: { job: Job }) {
  const [expanded, setExpanded] = useState(false)
  const badge = STATUS_CONFIG[job.status] ?? STATUS_CONFIG.pending
  const label = job.symlink_name ?? job.id.slice(0, 8)

  return (
    <>
      <tr
        className="cursor-pointer border-b border-border/60 hover:bg-muted/10 transition-colors"
        onClick={() => setExpanded((v) => !v)}
      >
        <td className="px-3 py-2 font-mono text-xs text-foreground/80 whitespace-nowrap">
          <span className="flex items-center gap-1">
            {expanded ? <ChevronDown className="size-3 shrink-0" /> : <ChevronRight className="size-3 shrink-0" />}
            {label}
          </span>
        </td>
        <td className="px-3 py-2 whitespace-nowrap">
          <span className={`rounded-full px-2 py-0.5 text-[10px] font-semibold ${badge.cls}`}>
            {badge.label}
          </span>
        </td>
        <td className="px-3 py-2 text-[11px] text-foreground/80 max-w-xs">
          <span className="line-clamp-1">{job.description}</span>
        </td>
        <td className="px-3 py-2">
          <div className="flex flex-wrap gap-1">
            {job.scope.slice(0, 3).map((s) => <ScopeBadge key={s} path={s} />)}
            {job.scope.length > 3 && (
              <span className="text-[10px] text-muted-foreground/50">+{job.scope.length - 3}</span>
            )}
          </div>
        </td>
      </tr>
      {expanded && (
        <tr className="border-b border-border/60 bg-muted/5">
          <td colSpan={4} className="px-4 py-3">
            <div className="grid gap-3 text-[11px]">
              {/* Acceptance criteria */}
              {job.acceptance_criteria.length > 0 && (
                <div>
                  <div className="mb-1 font-medium text-muted-foreground uppercase tracking-wide text-[10px]">
                    Acceptance criteria
                  </div>
                  <ul className="space-y-0.5">
                    {job.acceptance_criteria.map((c, i) => (
                      <li key={i} className="flex items-start gap-1.5 text-foreground/80">
                        <span className="mt-0.5 font-mono text-muted-foreground/40">[ ]</span>
                        {c}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
              {/* Log entries */}
              {job.log_entries.length > 0 && (
                <div>
                  <div className="mb-1 font-medium text-muted-foreground uppercase tracking-wide text-[10px]">
                    Recent log
                  </div>
                  <ul className="space-y-0.5 font-mono">
                    {job.log_entries.map((entry) => (
                      <li key={entry.id} className="text-[10px] text-muted-foreground">
                        <span className="mr-1.5 opacity-40">{entry.created_at.slice(11, 19)}</span>
                        {entry.message}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
              {/* Full scope */}
              {job.scope.length > 0 && (
                <div>
                  <div className="mb-1 font-medium text-muted-foreground uppercase tracking-wide text-[10px]">
                    Scope
                  </div>
                  <div className="flex flex-wrap gap-1">
                    {job.scope.map((s) => <ScopeBadge key={s} path={s} />)}
                  </div>
                </div>
              )}
              {/* Capability link */}
              {job.capability_id && (
                <div className="text-muted-foreground">
                  capability: <span className="font-mono">{job.capability_id}</span>
                </div>
              )}
            </div>
          </td>
        </tr>
      )}
    </>
  )
}

// ── Main page ─────────────────────────────────────────────────────────────

function DevJobsPage() {
  const { jobs, milestones, capabilities } = Route.useLoaderData()
  const [refreshing, setRefreshing] = useState(false)
  const router = useRouter()

  const refresh = async () => {
    setRefreshing(true)
    await router.invalidate()
    setRefreshing(false)
  }

  // Group jobs by status
  const byStatus = STATUS_ORDER.reduce<Record<string, Job[]>>((acc, s) => {
    acc[s] = jobs.filter((j) => j.status === s)
    return acc
  }, {} as Record<string, Job[]>)

  // Group capabilities by target
  const byTarget = capabilities.reduce<Record<string, CapabilityRow[]>>((acc, cap) => {
    ;(acc[cap.target_id] ??= []).push(cap)
    return acc
  }, {})

  return (
    <div className="min-h-screen bg-background px-6 py-8 font-sans">
      {/* Header */}
      <div className="mb-8 flex items-start justify-between">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className="rounded-md border border-amber-500/30 bg-amber-500/10 px-2 py-0.5 text-[10px] font-semibold text-amber-600 dark:text-amber-400 uppercase tracking-wide">
              dev only
            </span>
          </div>
          <h1 className="font-display text-xl font-semibold text-foreground">Job Control Panel</h1>
          <p className="mt-0.5 font-mono text-[11px] text-muted-foreground">/dev/jobs</p>
        </div>
        <div className="flex items-center gap-3">
          <a href="/studio" className="text-[11px] text-muted-foreground hover:text-foreground transition">
            ← Studio
          </a>
          <button
            onClick={refresh}
            disabled={refreshing}
            className="flex items-center gap-1.5 rounded-lg border border-border/60 px-3 py-1.5 text-xs text-muted-foreground transition hover:border-border hover:text-foreground disabled:opacity-50"
          >
            <RefreshCw className={`size-3 ${refreshing ? 'animate-spin' : ''}`} />
            Refresh
          </button>
        </div>
      </div>

      {/* Milestone progress */}
      {milestones.length > 0 && (
        <section className="mb-8">
          <h2 className="mb-3 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
            Milestone Progress
          </h2>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {milestones.map((ms) => (
              <div key={ms.id} className="rounded-xl border border-border/60 px-4 py-3">
                <div className="flex items-baseline justify-between">
                  <span className="font-medium text-[13px] text-foreground">{ms.title}</span>
                  <span className="font-mono text-[11px] text-muted-foreground">
                    {ms.actual}/{ms.total}
                  </span>
                </div>
                <ProgressBar value={ms.actual} max={ms.total} />
                <p className="mt-1 text-[10px] text-muted-foreground/60">
                  {ms.total === 0 ? 'No capabilities' : `${Math.round((ms.actual / ms.total) * 100)}% actual`}
                </p>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Job table grouped by status */}
      <section className="mb-10">
        <h2 className="mb-3 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
          Jobs
          <span className="ml-1.5 normal-case tracking-normal font-normal">({jobs.length})</span>
        </h2>
        {jobs.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">No active jobs.</p>
        ) : (
          <div className="space-y-6">
            {STATUS_ORDER.filter((s) => byStatus[s]?.length > 0).map((statusKey) => (
              <div key={statusKey}>
                <h3 className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">
                  {statusKey}
                  <span className="ml-1.5 normal-case tracking-normal font-normal opacity-60">
                    ({byStatus[statusKey].length})
                  </span>
                </h3>
                <div className="rounded-xl border border-border/60 overflow-x-auto">
                  <table className="w-full text-[11px]">
                    <thead>
                      <tr className="border-b border-border/60 bg-muted/30">
                        {['Name', 'Status', 'Description', 'Scope'].map((h) => (
                          <th key={h} className="px-3 py-2 text-left font-medium text-muted-foreground whitespace-nowrap">
                            {h}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {byStatus[statusKey].map((job) => (
                        <JobRow key={job.id} job={job} />
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Capability delta by surface */}
      <section>
        <h2 className="mb-3 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
          Capability Delta — v0.1.0
          <span className="ml-1.5 normal-case tracking-normal font-normal">
            ({capabilities.filter((c) => c.status === 'aspirational').length} remaining)
          </span>
        </h2>
        {capabilities.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">No capabilities found for active milestone.</p>
        ) : (
          <div className="space-y-5">
            {Object.entries(byTarget).map(([targetId, caps]) => (
              <div key={targetId}>
                <h3 className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">
                  {TARGET_NAMES[targetId] ?? targetId}
                  <span className="ml-1.5 normal-case tracking-normal font-normal opacity-60">
                    ({caps.filter((c) => c.status === 'actual').length}/{caps.length})
                  </span>
                </h3>
                <div className="rounded-xl border border-border/60 overflow-hidden">
                  {caps.map((cap, i) => (
                    <div
                      key={cap.id}
                      className={`flex items-start gap-2.5 px-3 py-2 text-[11px] ${i < caps.length - 1 ? 'border-b border-border/60' : ''}`}
                    >
                      <span className={`mt-0.5 shrink-0 font-mono text-[12px] ${cap.status === 'actual' ? 'text-green-500' : cap.running_job_id ? 'text-blue-500' : 'text-muted-foreground/30'}`}>
                        {cap.status === 'actual' ? '✓' : cap.running_job_id ? '▶' : '○'}
                      </span>
                      <div className="flex-1 min-w-0">
                        <span className={`${cap.status === 'actual' ? 'text-muted-foreground line-through' : 'text-foreground/80'}`}>
                          {cap.title}
                        </span>
                        {cap.status === 'actual' && cap.evidence && (
                          <span className="ml-2 font-mono text-[10px] text-green-500/70">{cap.evidence}</span>
                        )}
                        {cap.running_job_id && (
                          <span className="ml-2 font-mono text-[10px] text-blue-500/70">
                            job/{cap.running_job_id.slice(0, 8)} — {cap.running_job_desc?.slice(0, 40)}
                          </span>
                        )}
                      </div>
                      <span className="shrink-0 font-mono text-[10px] text-muted-foreground/30">{cap.id}</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
