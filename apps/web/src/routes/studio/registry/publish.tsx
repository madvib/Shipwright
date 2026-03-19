import { createFileRoute, Link } from '@tanstack/react-router'
import { useState } from 'react'
import { Upload, Loader2, CheckCircle2, AlertTriangle, ArrowLeft, ExternalLink } from 'lucide-react'
import { authClient } from '#/lib/auth-client'

// Route path registered in routeTree.gen.ts on next dev server start.
export const Route = createFileRoute('/studio/registry/publish' as '/studio/registry/')({
  component: PublishPage,
})

interface PublishResult {
  package_id: string
  version: string
  skills_indexed: number
  package_path?: string
}

function PublishPage() {
  const { data: session, isPending } = authClient.useSession()
  const isSignedIn = !!session?.user

  const [repoUrl, setRepoUrl] = useState('')
  const [tag, setTag] = useState('')
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle')
  const [result, setResult] = useState<PublishResult | null>(null)
  const [errorMsg, setErrorMsg] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!repoUrl.trim()) return

    setStatus('loading')
    setErrorMsg('')
    setResult(null)

    try {
      const body: { repo_url: string; tag?: string } = { repo_url: repoUrl.trim() }
      if (tag.trim()) body.tag = tag.trim()

      const res = await fetch('/api/registry/publish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })

      const data = (await res.json()) as PublishResult & { error?: string }

      if (!res.ok) {
        setStatus('error')
        setErrorMsg(data.error ?? `Request failed (${res.status})`)
        return
      }

      // Derive registry path from repo URL
      let pkgPath: string | undefined
      try {
        const parsed = new URL(repoUrl.trim())
        if (parsed.hostname === 'github.com') {
          pkgPath = `github.com${parsed.pathname.replace(/\.git$/, '')}`
        }
      } catch { /* ignore */ }

      setResult({ ...data, package_path: pkgPath })
      setStatus('success')
    } catch {
      setStatus('error')
      setErrorMsg('Network error — please try again.')
    }
  }

  function handleReset() {
    setStatus('idle')
    setResult(null)
    setErrorMsg('')
    setRepoUrl('')
    setTag('')
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-auto p-5 pb-20">

        {/* Back link */}
        <Link
          to="/studio/registry"
          className="inline-flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors mb-4 no-underline"
        >
          <ArrowLeft className="size-3" />
          Registry
        </Link>

        {/* Header */}
        <div className="mb-6">
          <h1 className="text-base font-semibold text-foreground mb-1">Publish a package</h1>
          <p className="text-[11px] text-muted-foreground leading-relaxed">
            Submit a GitHub repository containing a{' '}
            <code className="rounded bg-muted/50 px-1 py-0.5 font-mono text-[10px]">.ship/ship.toml</code>
            {' '}to the registry. The repository must have a{' '}
            <code className="rounded bg-muted/50 px-1 py-0.5 font-mono text-[10px]">[module]</code>
            {' '}section with a name and version.
          </p>
        </div>

        {/* Auth gate */}
        {!isPending && !isSignedIn && (
          <div className="rounded-xl border border-border/60 bg-card p-6 text-center space-y-3">
            <div className="flex size-10 items-center justify-center rounded-full bg-muted/50 mx-auto">
              <Upload className="size-4 text-muted-foreground" />
            </div>
            <div>
              <p className="text-sm font-semibold text-foreground">Sign in to publish</p>
              <p className="text-[11px] text-muted-foreground mt-1">
                You need a GitHub account to publish packages to the registry.
              </p>
            </div>
            <button
              onClick={() =>
                void authClient.signIn.social({
                  provider: 'github',
                  callbackURL: '/studio/registry/publish',
                })
              }
              className="inline-flex items-center gap-1.5 rounded-lg bg-violet-600 dark:bg-violet-500 px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-violet-500 dark:hover:bg-violet-400"
            >
              Sign in with GitHub
            </button>
          </div>
        )}

        {/* Loading auth */}
        {isPending && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="size-5 animate-spin text-muted-foreground" />
          </div>
        )}

        {/* Success state */}
        {!isPending && isSignedIn && status === 'success' && result && (
          <div className="space-y-4">
            <div className="rounded-xl border border-emerald-500/30 bg-emerald-500/5 p-5">
              <div className="flex items-start gap-3 mb-4">
                <CheckCircle2 className="size-5 text-emerald-500 shrink-0 mt-0.5" />
                <div>
                  <p className="text-sm font-semibold text-foreground">Package published!</p>
                  <p className="text-[11px] text-muted-foreground mt-1">
                    Version{' '}
                    <span className="font-mono font-medium text-foreground">{result.version}</span>
                    {' '}indexed with{' '}
                    <span className="font-medium text-foreground">{result.skills_indexed}</span>
                    {' '}skill{result.skills_indexed !== 1 ? 's' : ''}.
                  </p>
                </div>
              </div>

              {result.package_path && (
                <Link
                  to={`/studio/registry/${encodeURIComponent(result.package_path)}` as '/'}
                  className="inline-flex items-center gap-1.5 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-xs font-medium text-emerald-400 transition hover:bg-emerald-500/15 no-underline"
                >
                  <ExternalLink className="size-3" />
                  View package
                </Link>
              )}
            </div>

            <button
              onClick={handleReset}
              className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              Publish another
            </button>
          </div>
        )}

        {/* Publish form */}
        {!isPending && isSignedIn && status !== 'success' && (
          <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4">

            {/* Error banner */}
            {status === 'error' && (
              <div className="flex items-start gap-2 rounded-lg border border-destructive/20 bg-destructive/5 px-4 py-3">
                <AlertTriangle className="size-4 text-destructive shrink-0 mt-0.5" />
                <p className="text-xs text-destructive">{errorMsg}</p>
              </div>
            )}

            {/* Repo URL */}
            <div>
              <label htmlFor="repo-url" className="block text-[11px] font-semibold uppercase tracking-widest text-muted-foreground mb-1.5">
                GitHub repository URL <span className="text-destructive">*</span>
              </label>
              <input
                id="repo-url"
                type="url"
                value={repoUrl}
                onChange={(e) => setRepoUrl(e.target.value)}
                placeholder="https://github.com/owner/repo"
                required
                disabled={status === 'loading'}
                className="w-full rounded-lg border border-border/60 bg-card px-3 py-2.5 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50 focus:border-primary/30 disabled:opacity-50 transition"
              />
            </div>

            {/* Tag */}
            <div>
              <label htmlFor="tag" className="block text-[11px] font-semibold uppercase tracking-widest text-muted-foreground mb-1.5">
                Git tag / version
              </label>
              <input
                id="tag"
                type="text"
                value={tag}
                onChange={(e) => setTag(e.target.value)}
                placeholder="latest"
                disabled={status === 'loading'}
                className="w-full rounded-lg border border-border/60 bg-card px-3 py-2.5 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50 focus:border-primary/30 disabled:opacity-50 transition"
              />
              <p className="mt-1 text-[11px] text-muted-foreground/60">
                Leave blank to use the default branch HEAD.
              </p>
            </div>

            {/* Requirements note */}
            <div className="rounded-lg border border-border/40 bg-muted/20 px-4 py-3 space-y-1">
              <p className="text-[11px] font-semibold text-muted-foreground">Requirements</p>
              <ul className="text-[11px] text-muted-foreground/70 space-y-0.5">
                <li>• Repository must contain <code className="font-mono">.ship/ship.toml</code></li>
                <li>• <code className="font-mono">[module]</code> section with <code className="font-mono">name</code> and <code className="font-mono">version</code></li>
                <li>• Skill files at <code className="font-mono">.ship/skills/&lt;id&gt;.md</code></li>
              </ul>
            </div>

            {/* Submit */}
            <button
              type="submit"
              disabled={status === 'loading' || !repoUrl.trim()}
              className="inline-flex items-center gap-1.5 rounded-lg bg-violet-600 dark:bg-violet-500 px-5 py-2.5 text-xs font-semibold text-primary-foreground transition hover:bg-violet-500 dark:hover:bg-violet-400 disabled:opacity-60 disabled:cursor-not-allowed"
            >
              {status === 'loading' ? (
                <><Loader2 className="size-3.5 animate-spin" /> Publishing...</>
              ) : (
                <><Upload className="size-3.5" /> Publish package</>
              )}
            </button>
          </form>
        )}
      </div>
    </div>
  )
}
