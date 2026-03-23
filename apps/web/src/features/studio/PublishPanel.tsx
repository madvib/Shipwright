import { useState, useEffect } from 'react'
import { Github, Terminal, Upload, ChevronRight, X, Copy, CheckCheck, Loader2 } from 'lucide-react'
import { toast } from 'sonner'
import { authClient } from '#/lib/auth-client'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { PROVIDER_SHORT } from '#/features/compiler/components/ModeHeader'
import { PublishDialog } from '#/features/studio/PublishDialog'
import { PushToGitHubDialog } from '#/features/studio/PushToGitHubDialog'
import { useAgentStore } from '#/features/agents/useAgentStore'
import type { CompileState } from '#/features/compiler/useCompiler'
import type { CompileResult } from '#/features/compiler/types'

interface PublishPanelProps {
  auth: { isAuthenticated: boolean; user: { name: string } | null }
  library: any
  compileState: CompileState
  selectedProviders: string[]
  onCompile: () => void
  onClose: () => void
}

export function PublishPanel({ auth, library, compileState, selectedProviders, onCompile, onClose }: PublishPanelProps) {
  const hasContent = (library?.skills?.length ?? 0) > 0 || (library?.mcp_servers?.length ?? 0) > 0
  const [publishOpen, setPublishOpen] = useState(false)
  const [pushOpen, setPushOpen] = useState(false)

  const compileOutput = compileState.status === 'ok' ? compileState.output : null

  return (
    <aside className="w-96 border-l border-border/60 bg-card/30 flex flex-col overflow-hidden shrink-0">
      {/* Header */}
      <div className="px-4 py-2.5 border-b border-border/40 flex items-center justify-between shrink-0">
        <h3 className="text-xs font-semibold text-foreground flex items-center gap-2">
          Live Preview
          {compileState.status === 'ok' && (
            <span className="flex items-center gap-1 text-[10px] font-normal text-emerald-500">
              <span className="size-1.5 rounded-full bg-emerald-500 animate-pulse" />
              {compileState.elapsed}ms
            </span>
          )}
        </h3>
        <button onClick={onClose} className="text-muted-foreground/50 hover:text-muted-foreground">
          <X className="size-3.5" />
        </button>
      </div>

      {/* Output preview section */}
      <OutputSection
        compileState={compileState}
        onCompile={onCompile}
        hasContent={hasContent}
      />

      {/* CLI actions — always visible */}
      <CliSection />

      {/* Auth-gated distribution */}
      <div className="border-t border-border/40 shrink-0">
        {!auth.isAuthenticated ? (
          <SignInCTA />
        ) : (
          <DistributeSection
            hasContent={hasContent}
            isCompiled={compileState.status === 'ok'}
            onPublish={() => setPublishOpen(true)}
            onPush={() => setPushOpen(true)}
          />
        )}
      </div>

      <PublishDialog open={publishOpen} onOpenChange={setPublishOpen} />
      <PushToGitHubDialog
        open={pushOpen}
        onOpenChange={setPushOpen}
        compileOutput={compileOutput}
        selectedProviders={selectedProviders}
      />
    </aside>
  )
}

/** Live compiler output with provider tabs + file tabs */
const ALL_PREVIEW_PROVIDERS = ['claude', 'gemini', 'codex', 'cursor']

function OutputSection({ compileState, hasContent }: {
  compileState: CompileState; onCompile?: () => void; hasContent: boolean
}) {
  const [activeProvider, setActiveProvider] = useState('claude')
  const [activeFile, setActiveFile] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const output = compileState.status === 'ok' ? compileState.output : null
  const current = output?.[activeProvider] ?? null
  const tabs = current ? getFileTabs(activeProvider, current) : []

  useEffect(() => {
    if (tabs.length > 0 && !tabs.find((t) => t.id === activeFile)) {
      setActiveFile(tabs[0].id)
    }
  }, [activeProvider, compileState.status, tabs, activeFile])

  const displayTab = tabs.find((t) => t.id === activeFile) ?? tabs[0] ?? null
  const text = displayTab?.content ?? null

  const copy = () => {
    if (!text) return
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
      {/* Provider tabs */}
      {ALL_PREVIEW_PROVIDERS.length > 0 && (
        <div className="flex items-center gap-0.5 border-b border-border/40 px-2 py-1.5 shrink-0 overflow-x-auto [scrollbar-width:none]">
          {ALL_PREVIEW_PROVIDERS.map((p) => (
            <button
              key={p}
              onClick={() => setActiveProvider(p)}
              className={`flex shrink-0 items-center gap-1.5 rounded-md px-2 py-1 text-[11px] font-medium transition ${
                activeProvider === p
                  ? 'bg-muted text-foreground'
                  : 'text-muted-foreground/60 hover:text-foreground'
              }`}
            >
              <ProviderLogo provider={p} />
              {PROVIDER_SHORT[p] ?? p}
            </button>
          ))}
        </div>
      )}

      {/* File tabs + actions */}
      {tabs.length > 0 && (
        <div className="flex items-center justify-between border-b border-border/40 px-2 py-1 shrink-0">
          <div className="flex items-center gap-0.5 overflow-x-auto [scrollbar-width:none]">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveFile(tab.id)}
                className={`shrink-0 rounded px-1.5 py-0.5 text-[9px] font-mono font-medium transition ${
                  activeFile === tab.id ? 'bg-muted text-foreground' : 'text-muted-foreground/50 hover:text-foreground'
                }`}
              >
                {tab.filename}
              </button>
            ))}
          </div>
          <button onClick={copy} disabled={!text} className="rounded p-1 text-muted-foreground hover:text-foreground disabled:opacity-30 transition">
            {copied ? <CheckCheck className="size-3 text-emerald-500" /> : <Copy className="size-3" />}
          </button>
        </div>
      )}

      {/* Content */}
      <div className="flex-1 min-h-0 overflow-hidden">
        {compileState.status === 'idle' && (
          <div className="flex flex-col items-center justify-center h-full p-6 text-center">
            {hasContent ? (
              <p className="text-xs text-muted-foreground">Config ready to preview</p>
            ) : (
              <p className="text-xs text-muted-foreground">Select an agent to see compiled output</p>
            )}
          </div>
        )}
        {compileState.status === 'compiling' && (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="size-4 animate-spin text-muted-foreground" />
          </div>
        )}
        {compileState.status === 'error' && (
          <div className="p-3 text-xs text-destructive">{compileState.message}</div>
        )}
        {compileState.status === 'ok' && text && (
          <pre className="h-full overflow-auto p-3 font-mono text-[10px] leading-relaxed text-foreground/70 whitespace-pre-wrap break-all">
            {text}
          </pre>
        )}
      </div>
    </div>
  )
}

function SignInCTA() {
  return (
    <div className="p-4">
      <p className="text-xs font-medium text-foreground mb-1">Sign in to publish</p>
      <p className="text-[10px] text-muted-foreground mb-3">Push to GitHub, publish to the registry, sync across devices.</p>
      <button
        onClick={() => void authClient.signIn.social({ provider: 'github', callbackURL: window.location.href })}
        className="w-full inline-flex items-center justify-center gap-2 rounded-lg border border-border/60 bg-card px-3 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
      >
        <svg className="size-3.5" viewBox="0 0 16 16" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
        Sign in with GitHub
      </button>
    </div>
  )
}

function CliSection() {
  const { agents, activeId } = useAgentStore()
  const activeAgent = activeId ? agents.find((a) => a.profile.id === activeId) : undefined
  const agentId = activeAgent?.profile.id
  const importCmd = agentId ? `ship import --from ${window.location.origin}/studio/agents/${agentId}` : 'ship import --from <url>'

  return (
    <div className="border-t border-border/40 p-3 space-y-2">
      {/* CLI import command */}
      <div className="rounded-lg border border-border/40 bg-background/60 px-3 py-2">
        <div className="text-[10px] font-medium text-muted-foreground/60 mb-1">Import via CLI</div>
        <div className="flex items-center gap-1.5">
          <code className="flex-1 text-[10px] font-mono text-emerald-400 truncate">{importCmd}</code>
          <button
            onClick={() => {
              void navigator.clipboard.writeText(importCmd)
              toast.success('Copied to clipboard')
            }}
            className="shrink-0 rounded p-1 text-muted-foreground/40 hover:text-foreground transition"
          >
            <Copy className="size-3" />
          </button>
        </div>
      </div>
      {/* CLI download */}
      <a
        href="https://github.com/madvib/Ship#installation"
        target="_blank"
        rel="noopener noreferrer"
        className="flex items-center gap-2 rounded-lg border border-border/40 px-3 py-2 text-left transition hover:border-primary/30 hover:bg-primary/5 no-underline"
      >
        <Terminal className="size-3.5 text-muted-foreground" />
        <div className="flex-1 min-w-0">
          <span className="text-[11px] font-medium text-foreground">Get the CLI</span>
          <p className="text-[9px] text-muted-foreground/60">Install Ship to use agents locally</p>
        </div>
        <ChevronRight className="size-3 text-muted-foreground/20" />
      </a>
    </div>
  )
}

function DistributeSection({ hasContent, isCompiled, onPublish, onPush }: {
  hasContent: boolean; isCompiled: boolean; onPublish: () => void; onPush: () => void
}) {
  return (
    <div className="p-3 space-y-1.5">
      <DistAction icon={<Github className="size-3.5" />} label="Push to repo" desc="Create a PR with .ship/ config" disabled={!isCompiled} onClick={onPush} />
      <DistAction icon={<Upload className="size-3.5" />} label="Publish to registry" desc="Share with the community" disabled={!hasContent} onClick={onPublish} />
    </div>
  )
}

function DistAction({ icon, label, desc, disabled, onClick }: {
  icon: React.ReactNode; label: string; desc: string; disabled?: boolean; onClick?: () => void
}) {
  return (
    <button
      disabled={disabled}
      onClick={onClick}
      className={`w-full flex items-center gap-2.5 rounded-lg border border-border/40 px-3 py-2 text-left transition ${
        disabled ? 'opacity-30 cursor-not-allowed' : 'hover:border-primary/30 hover:bg-primary/5'
      }`}
    >
      <span className="text-muted-foreground">{icon}</span>
      <div className="flex-1 min-w-0">
        <span className="text-[11px] font-medium text-foreground">{label}</span>
        <p className="text-[9px] text-muted-foreground/60">{desc}</p>
      </div>
      <ChevronRight className="size-3 text-muted-foreground/20" />
    </button>
  )
}

/** Extract viewable file tabs from a compile result */
function getFileTabs(provider: string, result: CompileResult) {
  const tabs: { id: string; filename: string; content: string }[] = []
  const ctx = result.context_content
  if (ctx) {
    const name = provider === 'claude' ? 'CLAUDE.md' : provider === 'gemini' ? 'GEMINI.md' : 'AGENTS.md'
    tabs.push({ id: 'context', filename: name, content: ctx })
  }
  if (result.mcp_servers) {
    const path = provider === 'gemini' ? '.gemini/settings.json' : provider === 'cursor' ? '.cursor/mcp.json' : '.mcp.json'
    tabs.push({ id: 'mcp', filename: path, content: JSON.stringify(result.mcp_servers, null, 2) })
  }
  if (result.claude_settings_patch) {
    tabs.push({ id: 'claude-settings', filename: '.claude/settings.json', content: JSON.stringify(result.claude_settings_patch, null, 2) })
  }
  if (result.codex_config_patch) {
    tabs.push({ id: 'codex-config', filename: '.codex/config.toml', content: result.codex_config_patch })
  }
  if (result.gemini_settings_patch && provider === 'gemini') {
    tabs.push({ id: 'gemini-settings', filename: '.gemini/settings.json', content: JSON.stringify(result.gemini_settings_patch, null, 2) })
  }
  if (result.gemini_policy_patch) {
    tabs.push({ id: 'gemini-policy', filename: '.gemini/policies/ship.toml', content: result.gemini_policy_patch })
  }
  if (result.cursor_hooks_patch) {
    tabs.push({ id: 'cursor-hooks', filename: '.cursor/hooks.json', content: JSON.stringify(result.cursor_hooks_patch, null, 2) })
  }
  const ruleFiles = result.rule_files ?? {}
  Object.entries(ruleFiles).forEach(([path, content]) => {
    tabs.push({ id: `rule-${path}`, filename: path, content: content ?? '' })
  })
  return tabs
}
