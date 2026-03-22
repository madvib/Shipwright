import { ArrowRight } from 'lucide-react'

const STEPS = [
  {
    num: '1',
    title: 'Define your agent',
    description:
      'Pick skills, MCP servers, permissions, and rules in Ship Studio. Or import from an existing CLAUDE.md.',
  },
  {
    num: '2',
    title: 'Compile',
    description:
      'One click generates config files for every provider you target. CLAUDE.md, .mcp.json, GEMINI.md — all at once.',
  },
  {
    num: '3',
    title: 'Activate',
    description: 'Run ',
    cmd: 'ship use <agent>',
    descriptionAfter:
      ' to compile and emit config files on demand. Switch agents instantly.',
  },
]

export function HowItWorks() {
  return (
    <section className="mx-auto max-w-[62rem] px-6 pb-20 sm:px-10">
      <div className="mb-12 text-center">
        <h2 className="mb-2 font-display text-3xl font-extrabold sm:text-4xl">
          Three steps to configured agents
        </h2>
        <p className="text-[15px] text-muted-foreground">
          From zero to fully configured in under a minute.
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-3">
        {STEPS.map((step, i) => (
          <div
            key={step.num}
            className="relative rounded-xl border border-border/60 bg-card/30 p-6"
          >
            <div className="mb-3 flex size-7 items-center justify-center rounded-lg bg-primary/10 text-[13px] font-bold text-primary">
              {step.num}
            </div>
            <h3 className="mb-1.5 text-sm font-semibold">{step.title}</h3>
            <p className="text-xs leading-relaxed text-muted-foreground">
              {step.description}
              {step.cmd && (
                <code className="rounded bg-muted px-1 py-0.5 text-[11px] text-emerald-400">
                  {step.cmd}
                </code>
              )}
              {step.descriptionAfter}
            </p>
            {step.num === '3' && (
              <div className="mt-2.5 rounded-md border border-border/60 bg-background/60 px-2.5 py-2 font-mono text-[10px] text-emerald-400">
                $ ship use web-lane
              </div>
            )}
            {/* Arrow connector */}
            {i < STEPS.length - 1 && (
              <div className="pointer-events-none absolute -right-3 top-1/2 hidden -translate-y-1/2 text-border/40 sm:block">
                <ArrowRight className="size-4" />
              </div>
            )}
          </div>
        ))}
      </div>
    </section>
  )
}
