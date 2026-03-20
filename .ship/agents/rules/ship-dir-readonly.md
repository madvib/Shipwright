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
- State and records → Ship MCP tools (`create_note`, `create_target`, `append_job_log`, etc.)
- Handoffs → `handoff.md` in the job worktree root
- Plans → `job-spec.md` in the job worktree root
- Architecture decisions → `create_adr` via MCP
