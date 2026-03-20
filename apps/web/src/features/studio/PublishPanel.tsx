import { Github, Terminal, Upload, ChevronRight, ExternalLink, Check, X } from 'lucide-react'
import { authClient } from '#/lib/auth-client'
import type { CompileState } from '#/features/compiler/useCompiler'

interface PublishPanelProps {
  auth: { isAuthenticated: boolean; user: { name: string } | null }
  library: any
  compileState: CompileState
  onCompile: () => void
  onClose: () => void
}

export function PublishPanel({ auth, library, compileState, onCompile, onClose }: PublishPanelProps) {
  const hasAgents = (library?.modes?.length ?? 0) > 0 || (library?.agent_profiles?.length ?? 0) > 0
  const hasSkills = (library?.skills?.length ?? 0) > 0
  const hasMcp = (library?.mcp_servers?.length ?? 0) > 0
  const hasContent = hasAgents || hasSkills || hasMcp

  return (
    <aside className="w-80 border-l border-border/60 bg-card/30 flex flex-col overflow-hidden shrink-0">
      <div className="px-4 py-3 border-b border-border/40 flex items-center justify-between">
        <h3 className="text-xs font-semibold text-foreground">Publish</h3>
        <button onClick={onClose} className="text-muted-foreground/50 hover:text-muted-foreground">
          <X className="size-3.5" />
        </button>
      </div>

      <div className="flex-1 overflow-auto">
        {/* Auth state */}
        {!auth.isAuthenticated ? (
          <SignInSection />
        ) : (
          <>
            <ConnectionStatus user={auth.user} />
            <GitHubAppSection />
            <CLISection />
            {hasContent && (
              <CompileSection compileState={compileState} onCompile={onCompile} />
            )}
            <PublishSection hasContent={hasContent} />
          </>
        )}
      </div>
    </aside>
  )
}

function SignInSection() {
  return (
    <div className="p-4">
      <div className="rounded-xl border border-border/60 bg-card p-5 text-center">
        <div className="size-10 mx-auto mb-3 rounded-xl bg-primary/10 flex items-center justify-center">
          <Upload className="size-5 text-primary" />
        </div>
        <h4 className="text-sm font-semibold text-foreground mb-1">Sign in to publish</h4>
        <p className="text-xs text-muted-foreground mb-4">
          Connect your GitHub account to sync configs, push to repos, and publish to the registry.
        </p>
        <button
          onClick={() => void authClient.signIn.social({ provider: 'github', callbackURL: window.location.href })}
          className="w-full inline-flex items-center justify-center gap-2 rounded-lg bg-foreground px-4 py-2 text-sm font-medium text-background transition hover:opacity-90"
        >
          <svg className="size-4" viewBox="0 0 16 16" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
          Sign in with GitHub
        </button>
      </div>

      <div className="mt-4 space-y-3">
        <StepItem
          icon={<Github className="size-3.5" />}
          title="Sync to GitHub"
          desc="Push agent configs to any repo with a PR"
          done={false}
        />
        <StepItem
          icon={<Upload className="size-3.5" />}
          title="Publish to registry"
          desc="Share skills and agents with the community"
          done={false}
        />
        <StepItem
          icon={<Terminal className="size-3.5" />}
          title="Use locally with CLI"
          desc="ship use <agent> to apply configs in your project"
          done={false}
        />
      </div>
    </div>
  )
}

function ConnectionStatus({ user }: { user: { name: string } | null }) {
  return (
    <div className="px-4 py-3 border-b border-border/40">
      <div className="flex items-center gap-2 text-xs">
        <span className="size-1.5 rounded-full bg-emerald-500" />
        <span className="text-muted-foreground">Signed in as</span>
        <span className="font-medium text-foreground">{user?.name ?? 'user'}</span>
      </div>
    </div>
  )
}

function GitHubAppSection() {
  // TODO: check actual GitHub App installation status
  const isInstalled = false

  return (
    <div className="p-4 border-b border-border/40">
      <StepItem
        icon={<Github className="size-3.5" />}
        title="GitHub App"
        desc={isInstalled ? 'Connected — can push PRs to repos' : 'Install to push configs to your repos'}
        done={isInstalled}
        action={!isInstalled ? (
          <a
            href="https://github.com/apps/ship-dev"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-[10px] font-medium text-primary hover:underline"
          >
            Install <ExternalLink className="size-2.5" />
          </a>
        ) : undefined}
      />
    </div>
  )
}

function CLISection() {
  return (
    <div className="p-4 border-b border-border/40">
      <StepItem
        icon={<Terminal className="size-3.5" />}
        title="Ship CLI"
        desc="Apply configs locally with ship use"
        done={false}
        action={
          <div className="mt-1.5">
            <code className="block text-[10px] font-mono text-emerald-500 bg-emerald-500/5 border border-emerald-500/10 rounded px-2 py-1">
              curl -fsSL https://getship.dev/install | sh
            </code>
          </div>
        }
      />
    </div>
  )
}

function CompileSection({ compileState, onCompile }: { compileState: CompileState; onCompile: () => void }) {
  const isOk = compileState.status === 'ok'
  const isCompiling = compileState.status === 'compiling'

  return (
    <div className="p-4 border-b border-border/40">
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs font-medium text-foreground">Output preview</span>
        <button
          onClick={onCompile}
          disabled={isCompiling}
          className="text-[10px] font-medium text-primary hover:underline disabled:opacity-50"
        >
          {isCompiling ? 'Compiling...' : isOk ? 'Recompile' : 'Compile now'}
        </button>
      </div>
      {isOk && (
        <div className="space-y-2">
          {Object.entries(compileState.output).map(([provider, result]: [string, any]) => (
            <div key={provider} className="rounded-lg border border-border/40 bg-muted/20 p-2.5">
              <div className="text-[10px] font-semibold text-primary uppercase tracking-wider mb-1">{provider}</div>
              {result.context_content && (
                <pre className="text-[9px] font-mono text-muted-foreground leading-relaxed max-h-24 overflow-hidden">
                  {result.context_content.slice(0, 200)}...
                </pre>
              )}
              {result.mcp_config_path && (
                <div className="text-[9px] font-mono text-muted-foreground/50 mt-1">{result.mcp_config_path}</div>
              )}
            </div>
          ))}
        </div>
      )}
      {compileState.status === 'error' && (
        <div className="text-xs text-destructive bg-destructive/5 rounded-lg p-2">{compileState.message}</div>
      )}
    </div>
  )
}

function PublishSection({ hasContent }: { hasContent: boolean }) {
  return (
    <div className="p-4">
      <div className="text-xs font-medium text-foreground mb-2">Distribute</div>
      <div className="space-y-1.5">
        <ActionRow
          icon={<Github className="size-3.5" />}
          label="Push to repo"
          desc="Create a PR with your .ship/ config"
          disabled={!hasContent}
        />
        <ActionRow
          icon={<Upload className="size-3.5" />}
          label="Publish to registry"
          desc="Share with the Ship community"
          disabled={!hasContent}
        />
        <ActionRow
          icon={<Terminal className="size-3.5" />}
          label="Download files"
          desc="Export compiled config files"
          disabled={!hasContent}
        />
      </div>
    </div>
  )
}

function StepItem({ icon, title, desc, done, action }: {
  icon: React.ReactNode; title: string; desc: string; done: boolean; action?: React.ReactNode
}) {
  return (
    <div className="flex items-start gap-2.5">
      <div className={`mt-0.5 size-6 rounded-lg flex items-center justify-center shrink-0 ${
        done ? 'bg-emerald-500/10 text-emerald-500' : 'bg-muted/50 text-muted-foreground'
      }`}>
        {done ? <Check className="size-3" /> : icon}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5">
          <span className="text-xs font-medium text-foreground">{title}</span>
          {done && <Check className="size-3 text-emerald-500" />}
        </div>
        <p className="text-[10px] text-muted-foreground/70 mt-0.5">{desc}</p>
        {action}
      </div>
    </div>
  )
}

function ActionRow({ icon, label, desc, disabled }: {
  icon: React.ReactNode; label: string; desc: string; disabled?: boolean
}) {
  return (
    <button
      disabled={disabled}
      className={`w-full flex items-center gap-2.5 rounded-lg border border-border/40 px-3 py-2.5 text-left transition ${
        disabled
          ? 'opacity-40 cursor-not-allowed'
          : 'hover:border-primary/30 hover:bg-primary/5 cursor-pointer'
      }`}
    >
      <span className="text-muted-foreground">{icon}</span>
      <div className="flex-1 min-w-0">
        <span className="text-xs font-medium text-foreground">{label}</span>
        <p className="text-[10px] text-muted-foreground/60">{desc}</p>
      </div>
      <ChevronRight className="size-3 text-muted-foreground/30" />
    </button>
  )
}
