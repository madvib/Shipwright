const SKILLS = [
  { abbr: 'SC', name: 'ship-coordination' },
  { abbr: 'CR', name: 'code-review' },
  { abbr: 'DE', name: 'debug-expert' },
  { abbr: 'FD', name: 'frontend-design' },
  { abbr: 'VR', name: 'vercel-react' },
]

const MCP_SERVERS = [
  { abbr: 'SH', name: 'ship', tools: 'all', toolColor: 'text-emerald-400' },
  { abbr: 'GH', name: 'github', tools: '8/18', toolColor: 'text-primary' },
  { abbr: 'FS', name: 'filesystem', tools: '4/8', toolColor: 'text-primary' },
]

const PERMISSIONS = ['read-only', 'ship-guarded', 'standard', 'full-access']
const ACTIVE_PERM = 'ship-guarded'

const OUTPUT_TABS = ['Claude', 'Gemini', 'Codex'] as const

export function ProductShowcase() {
  return (
    <section className="mx-auto mb-20 max-w-[62rem] px-6 sm:px-10">
      <div className="overflow-hidden rounded-2xl border border-border/60 shadow-2xl shadow-black/20">
        {/* Browser chrome bar */}
        <div className="flex items-center gap-1.5 border-b border-border/60 bg-muted/40 px-4 py-2.5">
          <span className="size-2.5 rounded-full bg-red-500/50" />
          <span className="size-2.5 rounded-full bg-amber-500/50" />
          <span className="size-2.5 rounded-full bg-emerald-500/50" />
          <span className="flex-1 text-center text-[11px] text-muted-foreground/50">
            Ship Studio
          </span>
        </div>

        {/* Body */}
        <div className="flex flex-col gap-5 bg-card/30 p-5 sm:flex-row">
          {/* Main panel */}
          <div className="flex-1 min-w-0">
            <AgentHeader />
            <SkillsSection />
            <McpSection />
            <PermissionsSection />
          </div>
          {/* Inspector panel */}
          <div className="w-full border-t border-border/40 pt-5 sm:w-60 sm:border-l sm:border-t-0 sm:pl-5 sm:pt-0">
            <OutputPreview />
          </div>
        </div>
      </div>
    </section>
  )
}

function AgentHeader() {
  return (
    <div className="mb-4 flex items-center gap-2.5">
      <div className="flex size-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/60 text-sm font-bold text-primary-foreground">
        W
      </div>
      <div>
        <div className="text-sm font-semibold">web-lane</div>
        <div className="text-[10px] text-muted-foreground/60">
          5 skills / 3 MCP / claude, gemini
        </div>
      </div>
    </div>
  )
}

function SkillsSection() {
  return (
    <div className="mb-3.5">
      <div className="mb-1.5 text-[9px] font-semibold uppercase tracking-wider text-muted-foreground/50">
        Skills
      </div>
      <div className="flex flex-wrap gap-1">
        {SKILLS.map((s) => (
          <span
            key={s.name}
            className="flex items-center gap-1.5 rounded-md border border-border/60 bg-muted/30 px-2 py-1 text-[10px] text-muted-foreground"
          >
            <span className="flex size-4 items-center justify-center rounded bg-primary/10 text-[7px] font-bold text-primary">
              {s.abbr}
            </span>
            {s.name}
          </span>
        ))}
      </div>
    </div>
  )
}

function McpSection() {
  return (
    <div className="mb-3.5">
      <div className="mb-1.5 text-[9px] font-semibold uppercase tracking-wider text-muted-foreground/50">
        MCP Servers
      </div>
      <div className="flex flex-wrap gap-1">
        {MCP_SERVERS.map((m) => (
          <span
            key={m.name}
            className="flex items-center gap-1.5 rounded-md border border-border/60 bg-muted/30 px-2 py-1 text-[10px] text-muted-foreground"
          >
            <span className="flex size-4 items-center justify-center rounded bg-blue-500/10 text-[7px] font-bold text-blue-400">
              {m.abbr}
            </span>
            {m.name}
            <span className={`ml-0.5 text-[9px] ${m.toolColor}`}>
              {m.tools}
            </span>
          </span>
        ))}
      </div>
    </div>
  )
}

function PermissionsSection() {
  return (
    <div>
      <div className="mb-1.5 text-[9px] font-semibold uppercase tracking-wider text-muted-foreground/50">
        Permissions
      </div>
      <div className="flex flex-wrap gap-1">
        {PERMISSIONS.map((p) => (
          <span
            key={p}
            className={`rounded px-1.5 py-0.5 text-[8px] font-medium border ${
              p === ACTIVE_PERM
                ? 'border-primary text-primary bg-primary/5'
                : 'border-border/60 text-muted-foreground/40'
            }`}
          >
            {p}
          </span>
        ))}
      </div>
    </div>
  )
}

function OutputPreview() {
  return (
    <>
      <div className="mb-2 text-[9px] font-semibold uppercase tracking-wider text-muted-foreground/50">
        Output Preview
      </div>
      <div className="mb-2 flex gap-2">
        {OUTPUT_TABS.map((tab, i) => (
          <span
            key={tab}
            className={`text-[9px] ${
              i === 0
                ? 'border-b border-primary pb-0.5 text-primary'
                : 'pb-0.5 text-muted-foreground/40'
            }`}
          >
            {tab}
          </span>
        ))}
      </div>
      <div className="overflow-x-auto rounded-md border border-border/60 bg-background/60 p-2.5 font-mono text-[9px] leading-relaxed text-muted-foreground/60">
        <div>
          <span className="text-muted-foreground/30">
            {'// .claude/settings.json'}
          </span>
        </div>
        <div>{'{'}</div>
        <div>
          {'  '}
          <span className="text-sky-300">{'"permissions"'}</span>
          {': {'}
        </div>
        <div>
          {'    '}
          <span className="text-sky-300">{'"allow"'}</span>
          {': ['}
        </div>
        <div>
          {'      '}
          <span className="text-amber-300">{'"Read"'}</span>,
        </div>
        <div>
          {'      '}
          <span className="text-amber-300">{'"Grep"'}</span>,
        </div>
        <div>
          {'      '}
          <span className="text-amber-300">{'"Bash(git *)"'}</span>
        </div>
        <div>{'    ],'}</div>
        <div>
          {'    '}
          <span className="text-sky-300">{'"deny"'}</span>
          {': ['}
        </div>
        <div>
          {'      '}
          <span className="text-amber-300">{'"Bash(rm -rf *)"'}</span>
        </div>
        <div>{'    ]'}</div>
        <div>{'  },'}</div>
        <div>
          {'  '}
          <span className="text-sky-300">{'"model"'}</span>
          {': '}
          <span className="text-amber-300">{'"sonnet-4-6"'}</span>
        </div>
        <div>{'}'}</div>
      </div>
    </>
  )
}
