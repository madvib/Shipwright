// ── Code output helpers for AnimatedShowcase ────────────────────────────────

export type Agent = {
  name: string
  preset: string
  skills: string[]
  mcp: { name: string; tools: string }[]
  permissions: { allow: string[]; deny: string[] }
  rules: string[]
}

export function CodeOutput({ agent, provider }: { agent: Agent; provider: string }) {
  if (provider === 'claude') {
    return (
      <div className="text-muted-foreground/60">
        <Line k="permissions" />
        <Line k="allow" indent={1} />
        {agent.permissions.allow.map((p) => (
          <Val key={p} v={`"${p}"`} indent={2} color="text-amber-300" />
        ))}
        <Line k="deny" indent={1} />
        {agent.permissions.deny.map((p) => (
          <Val key={p} v={`"${p}"`} indent={2} color="text-red-300/60" />
        ))}
        <Line k="model" v='"sonnet-4-6"' indent={0} />
      </div>
    )
  }
  if (provider === 'gemini') {
    return (
      <div className="text-muted-foreground/60">
        <div><span className="text-muted-foreground/30"># GEMINI.md</span></div>
        <div className="mt-1"><span className="text-sky-300">## Role</span></div>
        <div>You are <span className="text-amber-300">{agent.name}</span>.</div>
        <div className="mt-1"><span className="text-sky-300">## Allowed tools</span></div>
        {agent.permissions.allow.map((p) => (
          <div key={p}>- {p}</div>
        ))}
        <div className="mt-1"><span className="text-sky-300">## Rules</span></div>
        {agent.rules.map((r) => (
          <div key={r}>- {r}</div>
        ))}
      </div>
    )
  }
  if (provider === 'cursor') {
    return (
      <div className="text-muted-foreground/60">
        <Line k="mcp" />
        {agent.mcp.map((m) => (
          <div key={m.name}>
            {'  '}<span className="text-sky-300">{`"${m.name}"`}</span>: {'{'}
            <div>{'    '}<span className="text-sky-300">{'"tools"'}</span>: <span className="text-amber-300">{`"${m.tools}"`}</span></div>
            {'  }'}
          </div>
        ))}
      </div>
    )
  }
  // codex
  return (
    <div className="text-muted-foreground/60">
      <div><span className="text-muted-foreground/30"># AGENTS.md</span></div>
      <div className="mt-1"><span className="text-sky-300">## {agent.name}</span></div>
      <div className="mt-1">Skills: {agent.skills.join(', ')}</div>
      <div>Preset: {agent.preset}</div>
      <div className="mt-1"><span className="text-sky-300">## Denied</span></div>
      {agent.permissions.deny.map((p) => (
        <div key={p}>- <span className="text-red-300/60">{p}</span></div>
      ))}
    </div>
  )
}

function Line({ k, v, indent = 0 }: { k: string; v?: string; indent?: number }) {
  const pad = '  '.repeat(indent)
  return (
    <div>
      {pad}<span className="text-sky-300">{`"${k}"`}</span>
      {v ? <>: <span className="text-amber-300">{v}</span></> : ': {'}
    </div>
  )
}

function Val({ v, indent = 0, color = 'text-amber-300' }: { v: string; indent?: number; color?: string }) {
  const pad = '  '.repeat(indent)
  return <div>{pad}<span className={color}>{v}</span>,</div>
}
