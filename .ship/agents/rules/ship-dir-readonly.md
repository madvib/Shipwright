---
rule: .ship/ is read-only for agents
---

Agents never write to `.ship/`. The only writers are:
- `ship install` — populates installed deps into `.ship/agents/` from the registry
- `ship use` — writes compiled provider outputs (CLAUDE.md, .mcp.json, etc.)

**Never do any of these:**
- Create or modify files in `.ship/`
- Write notes, plans, handoffs, capability maps, or docs to `.ship/`
- Use `.ship/` as scratch space or a document dump
- Invent CLI commands in `[hooks]` profile sections without verifying they exist

**Where things actually go:**
- Agent progress → `log_progress`, `append_job_log`
- Agent scratch work → `.ship-session/` files (gitignored scratchpad)
- Handoffs → `handoff.md` in the job worktree root
- Plans → `.ship-session/` or `job-spec.md` in the job worktree root
- Notes and ADRs → human-facing documents, created only when the human asks
