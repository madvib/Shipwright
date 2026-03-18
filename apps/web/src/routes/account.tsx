import { createFileRoute, Link, useNavigate } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useEffect } from 'react'
import { authClient } from '#/lib/auth-client'
import { authKeys } from '#/lib/query-keys'
import { fetchApi } from '#/lib/api-errors'
import { LogOut, Github, Building2, CheckCircle2 } from 'lucide-react'

// ── Types ────────────────────────────────────────────────────────────────────

interface MeUser {
  id: string
  name: string
  email: string
  image: string | null
  createdAt: number
}

interface MeOrg {
  id: string
  name: string
  slug: string
  created_at: number
}

interface MeResponse {
  user: MeUser
  org: MeOrg | null
}

// ── Route ────────────────────────────────────────────────────────────────────

export const Route = createFileRoute('/account')({ component: AccountPage })

// ── Sub-components ───────────────────────────────────────────────────────────

function Avatar({ name, image, size = 'xl' }: { name: string; image?: string | null; size?: 'xl' | 'lg' }) {
  const cls = size === 'xl'
    ? 'size-16 text-xl ring-2 ring-border/40'
    : 'size-10 text-base ring-1 ring-border/30'

  if (image) {
    return <img src={image} alt={name} className={`${cls} rounded-full object-cover`} />
  }

  return (
    <span className={`${cls} rounded-full bg-primary/15 flex items-center justify-center font-bold text-primary`}>
      {name.charAt(0).toUpperCase()}
    </span>
  )
}

function SectionCard({ children }: { children: React.ReactNode }) {
  return (
    <div className="rounded-xl border border-border/60 bg-card overflow-hidden">
      {children}
    </div>
  )
}

function SectionHeader({ title }: { title: string }) {
  return (
    <div className="px-5 py-3 border-b border-border/40 bg-muted/20">
      <p className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">{title}</p>
    </div>
  )
}

function Row({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 px-5 py-4">
      {children}
    </div>
  )
}

// ── Page ─────────────────────────────────────────────────────────────────────

function AccountPage() {
  const navigate = useNavigate()
  const { data: session, isPending: sessionPending } = authClient.useSession()
  const user = session?.user

  // Redirect unauthenticated users after session check completes
  useEffect(() => {
    if (!sessionPending && !user) {
      void navigate({ to: '/' })
    }
  }, [sessionPending, user, navigate])

  const { data: me, isLoading: meLoading } = useQuery({
    queryKey: authKeys.me(),
    queryFn: () => fetchApi<MeResponse>('/api/me', { credentials: 'include' }),
    enabled: !!user,
    staleTime: 30_000,
  })

  const isLoading = sessionPending || meLoading

  const handleSignOut = () => {
    void authClient.signOut()
  }

  // GitHub connection heuristic: image URL from GitHub CDN implies OAuth via GitHub
  const hasGithub = Boolean(user?.image && user.image.includes('github'))

  // Show nothing while redirecting unauthenticated users
  if (!sessionPending && !user) return null

  return (
    <main className="min-h-screen px-4 py-12 sm:px-6">
      <div className="mx-auto max-w-lg space-y-5">

        {/* Page heading */}
        <div className="mb-8">
          <Link
            to="/studio"
            className="inline-flex items-center gap-1.5 text-[11px] text-muted-foreground hover:text-foreground transition-colors mb-4 no-underline"
          >
            &#8592; Studio
          </Link>
          <h1 className="font-display text-2xl font-bold tracking-tight">Account</h1>
        </div>

        {/* Profile card */}
        <SectionCard>
          <SectionHeader title="Profile" />
          <Row>
            <div className="flex items-center gap-4">
              {isLoading ? (
                <span className="size-16 rounded-full bg-muted animate-pulse" />
              ) : (
                <Avatar
                  name={me?.user.name ?? user?.name ?? '?'}
                  image={me?.user.image ?? user?.image}
                />
              )}
              <div className="min-w-0">
                {isLoading ? (
                  <div className="space-y-2">
                    <div className="h-3.5 w-28 rounded bg-muted animate-pulse" />
                    <div className="h-3 w-40 rounded bg-muted/60 animate-pulse" />
                  </div>
                ) : (
                  <>
                    <p className="font-semibold text-foreground text-sm truncate">
                      {me?.user.name ?? user?.name}
                    </p>
                    <p className="text-xs text-muted-foreground mt-0.5 truncate">
                      {me?.user.email ?? user?.email}
                    </p>
                    {me?.user.createdAt && (
                      <p className="text-[11px] text-muted-foreground/50 mt-1">
                        Member since{' '}
                        {new Date(me.user.createdAt).toLocaleDateString('en-US', {
                          month: 'long',
                          year: 'numeric',
                        })}
                      </p>
                    )}
                  </>
                )}
              </div>
            </div>
          </Row>
        </SectionCard>

        {/* Connected accounts */}
        <SectionCard>
          <SectionHeader title="Connected accounts" />
          <Row>
            <div className="flex items-center gap-3">
              <div className="flex size-8 items-center justify-center rounded-lg border border-border/60 bg-muted/30">
                <Github className="size-4 text-foreground/70" />
              </div>
              <div>
                <p className="text-sm font-medium text-foreground">GitHub</p>
                <p className="text-[11px] text-muted-foreground mt-0.5">
                  {hasGithub ? 'Used to sign in' : 'Not connected'}
                </p>
              </div>
            </div>
            {hasGithub ? (
              <span className="flex items-center gap-1.5 text-[11px] font-medium text-emerald-600 dark:text-emerald-400">
                <CheckCircle2 className="size-3.5" />
                Connected
              </span>
            ) : (
              <button
                onClick={() =>
                  void authClient.signIn.social({ provider: 'github', callbackURL: '/account' })
                }
                className="text-[11px] font-medium text-primary hover:text-primary/80 transition-colors"
              >
                Connect
              </button>
            )}
          </Row>
        </SectionCard>

        {/* Organisation — only shown when org data is available */}
        {(me?.org || meLoading) && (
          <SectionCard>
            <SectionHeader title="Organisation" />
            <Row>
              <div className="flex items-center gap-3">
                <div className="flex size-8 items-center justify-center rounded-lg border border-border/60 bg-muted/30">
                  <Building2 className="size-4 text-foreground/70" />
                </div>
                <div>
                  {meLoading ? (
                    <div className="space-y-1.5">
                      <div className="h-3 w-24 rounded bg-muted animate-pulse" />
                      <div className="h-2.5 w-16 rounded bg-muted/60 animate-pulse" />
                    </div>
                  ) : (
                    <>
                      <p className="text-sm font-medium text-foreground">{me?.org?.name}</p>
                      <p className="font-mono text-[11px] text-muted-foreground mt-0.5">
                        {me?.org?.slug}
                      </p>
                    </>
                  )}
                </div>
              </div>
            </Row>
          </SectionCard>
        )}

        {/* Sign out */}
        <SectionCard>
          <Row>
            <div className="flex items-center gap-3">
              <div className="flex size-8 items-center justify-center rounded-lg border border-border/60 bg-muted/30">
                <LogOut className="size-4 text-foreground/70" />
              </div>
              <div>
                <p className="text-sm font-medium text-foreground">Sign out</p>
                <p className="text-[11px] text-muted-foreground mt-0.5">
                  You can sign back in with GitHub at any time
                </p>
              </div>
            </div>
            <button
              onClick={handleSignOut}
              className="shrink-0 rounded-lg border border-border/60 bg-card px-3 py-1.5 text-xs font-medium text-muted-foreground transition hover:border-red-500/40 hover:bg-red-500/5 hover:text-red-600 dark:hover:text-red-400"
            >
              Sign out
            </button>
          </Row>
        </SectionCard>

      </div>
    </main>
  )
}
