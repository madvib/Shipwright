import { createFileRoute, Link } from '@tanstack/react-router'
import { Download, Copy, CheckCheck, Terminal, ArrowRight } from 'lucide-react'
import { useState } from 'react'
import { useLibrary } from '#/features/compiler/useLibrary'
import { getInspectorTabs } from '#/features/compiler/components/InspectorPanel'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { PROVIDER_SHORT, triggerDownload } from '#/features/compiler/components/ModeHeader'

export const Route = createFileRoute('/studio/export')({ component: ExportPage })

// CLI detection is not possible from the browser — default to no-CLI state.
// When auth is wired, we can check session.user.hasCli and switch states.
type CliState = 'cli-and-account' | 'cli-no-account' | 'no-cli'
const cliState: CliState = 'no-cli'

function ExportPage() {
  const { modeName, selectedProviders, compileState } = useLibrary()
  const output = compileState.status === 'ok' ? compileState.output : null
  const hasOutput = Boolean(output)

  const downloadAll = () => {
    if (!output) return
    selectedProviders.forEach((p) => {
      const result = output[p]
      if (result) getInspectorTabs(p, result).forEach((tab) => triggerDownload(tab.content, tab.filename))
    })
  }

  return (
    <div className="h-full flex flex-col">
      {/* View header */}
      <div className="flex items-center px-4 h-11 border-b border-border/60 bg-card/30 shrink-0">
        <span className="text-sm font-semibold text-foreground mr-2">Export</span>
        <span className="text-[11px] text-muted-foreground/50">Get your config into your AI tools</span>
      </div>

      <div className="flex-1 overflow-auto p-6">
        <div className="mx-auto max-w-3xl">

      {/* State: no CLI */}
      {cliState === 'no-cli' && <NoCLIState hasOutput={hasOutput} onDownloadAll={downloadAll} />}
      {cliState === 'cli-no-account' && <CliNoAccountState modeName={modeName} />}
      {cliState === 'cli-and-account' && <CliAndAccountState output={output} selectedProviders={selectedProviders} />}

      {/* Escape hatch: per-file downloads (all states) */}
      {hasOutput && output && (
        <div className="mt-8">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Files</h2>
            <button
              onClick={downloadAll}
              className="inline-flex items-center gap-1.5 text-xs text-primary hover:underline"
            >
              <Download className="size-3" />
              Download all
            </button>
          </div>
          <div className="space-y-2">
            {selectedProviders.map((p) => {
              const result = output[p]
              if (!result) return null
              const tabs = getInspectorTabs(p, result)
              return (
                <div key={p} className="rounded-xl border border-border/60 bg-card overflow-hidden">
                  <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/60 bg-muted/20">
                    <div className="flex items-center gap-2">
                      <ProviderLogo provider={p} size="md" />
                      <span className="text-xs font-semibold">{PROVIDER_SHORT[p] ?? p}</span>
                    </div>
                    <button
                      onClick={() => tabs.forEach((tab) => triggerDownload(tab.content, tab.filename))}
                      className="text-xs text-muted-foreground transition hover:text-foreground"
                    >
                      Download {tabs.length} file{tabs.length !== 1 ? 's' : ''}
                    </button>
                  </div>
                  <div className="divide-y divide-border/40">
                    {tabs.map((tab) => (
                      <div key={tab.id} className="flex items-center justify-between px-4 py-2">
                        <span className="font-mono text-[11px] text-muted-foreground">{tab.filename}</span>
                        <button
                          onClick={() => triggerDownload(tab.content, tab.filename)}
                          className="rounded p-1 text-muted-foreground transition hover:bg-muted hover:text-foreground"
                        >
                          <Download className="size-3" />
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}

        </div>
      </div>
    </div>
  )
}

// ── State 3: No CLI ───────────────────────────────────────────────────────────

function NoCLIState({ hasOutput, onDownloadAll }: { hasOutput: boolean; onDownloadAll: () => void }) {
  const [copiedInstall, setCopiedInstall] = useState(false)
  const installCmd = 'curl -fsSL https://getship.dev/install | sh'

  const copy = (text: string, setCopied: (v: boolean) => void) => {
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  return (
    <div className="space-y-4">
      {/* Install prompt */}
      <div className="rounded-xl border border-border/60 bg-card p-5">
        <div className="mb-4 flex items-center gap-3">
          <div className="flex size-8 items-center justify-center rounded-lg bg-foreground/5">
            <Terminal className="size-4 text-foreground" />
          </div>
          <div>
            <p className="text-sm font-semibold text-foreground">Install Ship CLI</p>
            <p className="text-[11px] text-muted-foreground">Sync config automatically — no copy-paste</p>
          </div>
        </div>

        <ol className="space-y-3">
          <li className="flex items-start gap-3">
            <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary/10 text-[10px] font-bold text-primary mt-0.5">1</span>
            <div className="flex-1">
              <p className="text-xs text-muted-foreground mb-1.5">Install the CLI</p>
              <div className="flex items-center gap-2 rounded-lg border border-border/60 bg-muted/30 pl-3 pr-1 py-1.5">
                <code className="flex-1 font-mono text-[11px] text-foreground">{installCmd}</code>
                <button
                  onClick={() => copy(installCmd, setCopiedInstall)}
                  className="flex size-6 items-center justify-center rounded text-muted-foreground transition hover:bg-muted hover:text-foreground"
                >
                  {copiedInstall ? <CheckCheck className="size-3.5 text-emerald-500" /> : <Copy className="size-3.5" />}
                </button>
              </div>
            </div>
          </li>
          <li className="flex items-start gap-3">
            <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary/10 text-[10px] font-bold text-primary mt-0.5">2</span>
            <div className="flex-1">
              <p className="text-xs text-muted-foreground mb-1.5">Link your account</p>
              <code className="block rounded-lg border border-border/60 bg-muted/30 px-3 py-1.5 font-mono text-[11px] text-foreground">ship login</code>
            </div>
          </li>
          <li className="flex items-start gap-3">
            <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary/10 text-[10px] font-bold text-primary mt-0.5">3</span>
            <div className="flex-1">
              <p className="text-xs text-muted-foreground">Config syncs automatically when you save</p>
            </div>
          </li>
        </ol>
      </div>

      {/* Escape hatch */}
      {hasOutput && (
        <div className="flex items-center justify-between rounded-xl border border-border/40 bg-muted/20 px-4 py-3">
          <p className="text-xs text-muted-foreground">Not using the CLI? Download files manually.</p>
          <button
            onClick={onDownloadAll}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 px-3 py-1.5 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
          >
            <Download className="size-3" />
            Download .zip
          </button>
        </div>
      )}

      {!hasOutput && (
        <div className="flex items-center gap-3 rounded-xl border border-border/40 bg-muted/20 px-4 py-3">
          <p className="text-xs text-muted-foreground">Configure a profile to generate files.</p>
          <Link
            to="/studio/profiles"
            className="ml-auto inline-flex items-center gap-1 text-xs text-primary no-underline hover:underline"
          >
            Go to profiles <ArrowRight className="size-3" />
          </Link>
        </div>
      )}
    </div>
  )
}

// ── State 2: CLI, no account ──────────────────────────────────────────────────

function CliNoAccountState({ modeName }: { modeName: string }) {
  const [copied, setCopied] = useState(false)
  const cmd = `ship use @me/${modeName || 'my-profile'}`

  const copy = () => {
    void navigator.clipboard.writeText(cmd).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  return (
    <div className="rounded-xl border border-border/60 bg-card p-5 space-y-4">
      <p className="text-sm text-foreground">Run this command to activate your profile:</p>
      <div className="flex items-center gap-2 rounded-lg border border-border/60 bg-muted/30 pl-3 pr-1 py-1.5">
        <code className="flex-1 font-mono text-[11px] text-foreground">{cmd}</code>
        <button
          onClick={copy}
          className="flex size-6 items-center justify-center rounded text-muted-foreground transition hover:bg-muted hover:text-foreground"
        >
          {copied ? <CheckCheck className="size-3.5 text-emerald-500" /> : <Copy className="size-3.5" />}
        </button>
      </div>
      <p className="text-[11px] text-muted-foreground">
        Link expires in 7 days · one-time use
      </p>
      <div className="rounded-lg border border-border/40 bg-primary/5 px-3 py-2.5">
        <p className="text-[11px] text-muted-foreground">
          Auto-sync with a Ship account — no more copy-paste.{' '}
          <Link to="/studio" className="text-primary no-underline hover:underline">Sign in →</Link>
        </p>
      </div>
    </div>
  )
}

// ── State 1: CLI + account ────────────────────────────────────────────────────

function CliAndAccountState({
  output,
  selectedProviders,
}: {
  output: Record<string, unknown> | null
  selectedProviders: string[]
}) {
  return (
    <div className="rounded-xl border border-border/60 bg-card p-5">
      <div className="flex items-center gap-2 mb-4">
        <span className="size-2 rounded-full bg-emerald-500 animate-pulse" />
        <p className="text-sm font-medium text-foreground">Auto-syncing</p>
      </div>
      <p className="text-xs text-muted-foreground">
        Changes sync automatically when saved.{' '}
        {output && selectedProviders.length > 0 && (
          <span>{selectedProviders.length} provider{selectedProviders.length !== 1 ? 's' : ''} active.</span>
        )}
      </p>
    </div>
  )
}
