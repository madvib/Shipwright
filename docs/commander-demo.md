# Commander Demo — Dev Preview

> Requires `SHIP_COMMANDER_DEMO=1` in your environment.
> This is a documentation convention signaling you've read this guide.
> It does not gate any code paths.

## What this is

Ship's commander workflow lets a human direct multiple Claude Code agents
through a plan-dispatch-gate cycle. The commander plans work, dispatches
specialists into isolated worktrees, monitors progress, and gates completed
work — all through MCP tool calls against Ship's platform DB.

## Prerequisites

```bash
export SHIP_COMMANDER_DEMO=1
just install          # builds ship + ship-mcp to ~/.cargo/bin
ship use commander    # compiles commander config → CLAUDE.md, .mcp.json
```

## The three phases

### 1. PLANNER — translate intent into capabilities

Commander reads the active target, asks 1–2 clarifying questions, decomposes
into verifiable capabilities, and presents a plan for confirmation.

```
create_capability(
  target_id: "surface-compiler",
  title: "WASM output validates against component-model spec",
  acceptance_criteria: ["ship compile produces valid .wasm", "test_wasm_validation passes"],
  preset_hint: "rust-compiler",
  file_scope: ["crates/core/compiler/"]
)

create_job(
  kind: "feature",
  description: "Add WASM component-model validation to compiler output",
  branch: "job/wasm-validation",
  capability_id: "<cap-id>",
  preset_hint: "rust-compiler",
  file_scope: ["crates/core/compiler/"],
  acceptance_criteria: ["ship compile produces valid .wasm", "test_wasm_validation passes"]
)
```

**Commander asks for confirmation before dispatching.** The `ship-standard`
preset sets `default_mode = "default"` — the human must approve each dispatch.

### 2. ORCHESTRATOR — monitor and route running work

After confirmation, commander spawns worktrees and tracks progress.

```
list_jobs(status: "running")                    # what's in flight?
list_jobs(assigned_to: "human")                 # surface blockers

append_job_log(                                 # agent logs from worktree
  job_id: "<job-id>",
  message: "touched: crates/core/compiler/src/wasm.rs",
  level: "info"
)
```

Commander never does specialist work — it reads MCP state and routes jobs.

### 3. GATE — verify completed work

When an agent signals completion (status + handoff.md + `complete:` commit):

```
update_job(id: "<job-id>", status: "complete")  # agent signals done

mark_capability_actual(                         # commander verifies, then marks
  id: "<cap-id>",
  evidence: "test_wasm_validation passes — commit abc1234"
)

complete_workspace(                             # clean up
  workspace_id: "<ws-id>",
  summary: "WASM validation added. All acceptance criteria met."
)
```

Gate tiers: **auto** (docs, compile) — inline review. **review** (features,
schema) — spawns gate agent. **human** (deploy, credentials) — surfaces to you.

## End-to-end scenario: `ship status` command

**Goal:** Add `ship status` that prints active workspace, session, and running
jobs. Completable in under 30 minutes.

### Launch commander

```bash
export SHIP_COMMANDER_DEMO=1
cd ~/dev/ship && ship use commander && claude .
```

Tell commander: *"Add a `ship status` CLI command that prints the active
workspace, current session, and running jobs."*

### Commander plans (PLANNER)

Commander creates a capability scoped to `apps/ship-studio-cli/` with criteria:
- `ship status` prints workspace name and branch
- `ship status` prints session goal if active
- `ship status` lists running jobs with descriptions
- `ship status` exits 0 when no session is active

**You confirm the plan before dispatch.**

### Commander dispatches (ORCHESTRATOR)

Commander spawns the worktree and gives you the launch command:

```bash
git worktree add ~/dev/ship-worktrees/ship-status -b job/ship-status
cd ~/dev/ship-worktrees/ship-status && ship use cli-lane
```

Writes `.ship-session/job-spec.md` with scope and completion contract.
You open a new terminal:

```bash
cd ~/dev/ship-worktrees/ship-status && claude .
```

The specialist reads the job spec automatically and begins working.

### Specialist works autonomously

The cli-lane agent implements, tests, commits (`complete: ship status`),
writes `handoff.md`, and calls `update_job(status: "complete")`.

### Commander gates (GATE)

Back in the commander terminal, ask it to check on the job. Commander reads
`handoff.md`, verifies each criterion, and on pass:
- `mark_capability_actual(id, evidence)` with concrete evidence
- `complete_workspace(workspace_id, summary)` to prune the worktree

## Key constraints

- **Commander has zero file scope.** Reads MCP state only — never writes code.
- **Agents don't call each other.** They emit jobs; commander routes them.
- **File scope is authority.** `file_scope: ["apps/web/"]` blocks `crates/`.
- **Gate is non-negotiable.** Every job passes gate before capability is actual.
- **One claim per file.** No two running jobs claim the same path.
- **Human confirms dispatch.** `default_mode = "default"` (not `dontAsk`).

## Permission model

| Preset | Mode | Used by |
|---|---|---|
| `ship-standard` | `default` | Commander — asks before acting |
| `ship-autonomous` | `dontAsk` | Specialists in worktrees |
| `ship-readonly` | `plan` | Reviewer, gate agents |

## MCP tools reference

| Tool | Phase | Purpose |
|---|---|---|
| `create_capability` | PLANNER | Define verifiable outcome |
| `create_job` | PLANNER | Create work unit |
| `update_job` | ALL | Update status/assignment |
| `append_job_log` | ORCHESTRATOR | Agent progress logging |
| `mark_capability_actual` | GATE | Record completion evidence |
| `complete_workspace` | GATE | Write handoff, prune worktree |
| `list_jobs` | ORCHESTRATOR | Monitor running/blocked work |
| `list_capabilities` | ALL | Track aspirational vs actual |
