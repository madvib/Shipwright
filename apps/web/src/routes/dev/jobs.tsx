import { createFileRoute, redirect, useRouter } from '@tanstack/react-router'
import { createServerFn } from '@tanstack/react-start'
import { useState } from 'react'
import { RefreshCw } from 'lucide-react'

// ── Types ─────────────────────────────────────────────────────────────────

interface Job {
  id: string
  status: 'pending' | 'running'
  description: string
  branch: string
  worktree_path: string
  touched_files: null // aspirational — xzVkQ3Y3
}

interface Capability {
  id: string
  description: string
  target_id: string
}

// ── Target name map (stable — from list_targets) ──────────────────────────

const TARGET_NAMES: Record<string, string> = {
  ShYqMr8e: 'compiler',
  gbJkmuwY: 'cli',
  mXTZ4djg: 'studio',
  MeHin6Fy: 'platform',
  iu86rzHS: 'mcp',
  zU6Leq6v: 'infra',
  JUPHSmmW: 'desktop',
}

// ── ship-mcp JSON-RPC helper ──────────────────────────────────────────────

async function callShipMcpTool(toolName: string, args: Record<string, unknown>): Promise<string> {
  const { spawn } = await import(/* @vite-ignore */ 'node:child_process')

  return new Promise((resolve, reject) => {
    const proc = spawn('ship-mcp', [], { stdio: ['pipe', 'pipe', 'pipe'] })

    const send = (msg: unknown) => {
      proc.stdin.write(JSON.stringify(msg) + '\n')
    }

    let buffer = ''
    let step = 0

    proc.stdout.on('data', (chunk: Buffer) => {
      buffer += chunk.toString()
      const lines = buffer.split('\n')
      buffer = lines.pop() ?? ''

      for (const line of lines) {
        if (!line.trim()) continue
        try {
          const msg = JSON.parse(line) as {
            id?: number
            result?: { content?: Array<{ text?: string }> }
          }
          if (step === 0 && msg.id === 1) {
            send({ jsonrpc: '2.0', method: 'notifications/initialized' })
            send({ jsonrpc: '2.0', id: 2, method: 'tools/call', params: { name: toolName, arguments: args } })
            step = 1
          } else if (step === 1 && msg.id === 2) {
            proc.kill()
            resolve(msg.result?.content?.[0]?.text ?? '')
          }
        } catch { /* ignore parse errors */ }
      }
    })

    send({
      jsonrpc: '2.0',
      id: 1,
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: { name: 'ship-studio-dev', version: '0.1.0' },
      },
    })

    proc.on('error', reject)
    setTimeout(() => { proc.kill(); reject(new Error('ship-mcp timeout')) }, 5000)
  })
}

// ── Server functions ──────────────────────────────────────────────────────

const getJobs = createServerFn({ method: 'GET' }).handler(async (): Promise<Job[]> => {
  try {
    const [runningText, pendingText] = await Promise.all([
      callShipMcpTool('list_jobs', { status: 'running' }),
      callShipMcpTool('list_jobs', { status: 'pending' }),
    ])

    const parseLines = (text: string, status: 'running' | 'pending'): Job[] => {
      const jobs: Job[] = []
      for (const line of text.split('\n')) {
        const m = line.match(/^\s*-\s+(\S+)\s+\[(?:running|pending)\]/)
        if (m) {
          jobs.push({
            id: m[1],
            status,
            description: '',
            branch: `job/${m[1]}`,
            worktree_path: `~/dev/ship-worktrees/${m[1]}`,
            touched_files: null,
          })
        }
      }
      return jobs
    }

    return [...parseLines(runningText, 'running'), ...parseLines(pendingText, 'pending')]
  } catch {
    return []
  }
})

const getCapabilities = createServerFn({ method: 'GET' }).handler(async (): Promise<Capability[]> => {
  try {
    const text = await callShipMcpTool('list_capabilities', { status: 'aspirational' })
    const caps: Capability[] = []
    for (const line of text.split('\n')) {
      const m = line.match(/^- \[ \] (.+) \(id: (\w+), target: (\w+)\)$/)
      if (m) caps.push({ description: m[1].trim(), id: m[2], target_id: m[3] })
    }
    return caps
  } catch {
    return []
  }
})

// ── Route ─────────────────────────────────────────────────────────────────

export const Route = createFileRoute('/dev/jobs')({
  beforeLoad: () => {
    if (import.meta.env.PROD) throw redirect({ to: '/' })
  },
  loader: async () => {
    const [jobs, capabilities] = await Promise.all([getJobs(), getCapabilities()])
    return { jobs, capabilities }
  },
  component: DevJobsPage,
})

// ── Status badge config ───────────────────────────────────────────────────

const STATUS: Record<string, { label: string; cls: string }> = {
  running: { label: '● running', cls: 'bg-blue-500/10 text-blue-600 dark:text-blue-400' },
  pending: { label: '○ pending', cls: 'bg-amber-500/10 text-amber-600 dark:text-amber-400' },
}

// ── Component ─────────────────────────────────────────────────────────────

function DevJobsPage() {
  const { jobs, capabilities } = Route.useLoaderData()
  const [refreshing, setRefreshing] = useState(false)
  const router = useRouter()

  const refresh = async () => {
    setRefreshing(true)
    await router.invalidate()
    setRefreshing(false)
  }

  const byTarget = capabilities.reduce<Record<string, Capability[]>>((acc, cap) => {
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
          <h1 className="font-display text-xl font-semibold text-foreground">Job Queue</h1>
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

      {/* Jobs */}
      <section className="mb-10">
        <h2 className="mb-3 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
          Running &amp; Pending
          <span className="ml-1.5 normal-case tracking-normal font-normal">({jobs.length})</span>
        </h2>
        {jobs.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">No active jobs.</p>
        ) : (
          <div className="rounded-xl border border-border/60 overflow-x-auto">
            <table className="w-full text-[11px]">
              <thead>
                <tr className="border-b border-border/60 bg-muted/30">
                  {['ID', 'Status', 'Description', 'Branch', 'Worktree', 'Touched files'].map((h) => (
                    <th key={h} className="px-3 py-2 text-left font-medium text-muted-foreground whitespace-nowrap">{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {jobs.map((job, i) => {
                  const badge = STATUS[job.status]
                  return (
                    <tr key={job.id} className={`border-b border-border/60 last:border-0 ${i % 2 === 0 ? '' : 'bg-muted/10'}`}>
                      <td className="px-3 py-2 font-mono text-foreground whitespace-nowrap">{job.id}</td>
                      <td className="px-3 py-2 whitespace-nowrap">
                        <span className={`rounded-full px-2 py-0.5 text-[10px] font-semibold ${badge.cls}`}>
                          {badge.label}
                        </span>
                      </td>
                      <td className="px-3 py-2 text-foreground/80 max-w-xs">
                        <span className="line-clamp-2">{job.description}</span>
                      </td>
                      <td className="px-3 py-2 font-mono text-muted-foreground whitespace-nowrap">{job.branch}</td>
                      <td className="px-3 py-2 font-mono text-muted-foreground whitespace-nowrap">{job.worktree_path}</td>
                      <td className="px-3 py-2 text-muted-foreground/40" title="aspirational — capability xzVkQ3Y3 not yet implemented">—</td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {/* Capability delta */}
      <section>
        <h2 className="mb-3 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
          Capability Delta — v0.1.0 aspirational
          <span className="ml-1.5 normal-case tracking-normal font-normal">({capabilities.length} remaining)</span>
        </h2>
        {capabilities.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">No aspirational capabilities found.</p>
        ) : (
          <div className="space-y-5">
            {Object.entries(byTarget).map(([targetId, caps]) => (
              <div key={targetId}>
                <h3 className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">
                  {TARGET_NAMES[targetId] ?? targetId}
                  <span className="ml-1.5 normal-case tracking-normal font-normal opacity-60">({caps.length})</span>
                </h3>
                <div className="rounded-xl border border-border/60 overflow-hidden">
                  {caps.map((cap, i) => (
                    <div
                      key={cap.id}
                      className={`flex items-start gap-2.5 px-3 py-2 text-[11px] ${i < caps.length - 1 ? 'border-b border-border/60' : ''}`}
                    >
                      <span className="mt-0.5 shrink-0 font-mono text-muted-foreground/30">[ ]</span>
                      <span className="flex-1 text-foreground/80">{cap.description}</span>
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
