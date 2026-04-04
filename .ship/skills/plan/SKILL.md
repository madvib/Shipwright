---
name: plan
stable-id: plan
description: Create structured job specs in .ship-session/specs/. Specs become launchable jobs via the daemon.
tags: [planning, jobs]
authors: [ship]
---

# Plan

Write job specs to `.ship-session/specs/`. Each spec is a directory containing a `spec.md` with YAML frontmatter that the daemon can launch as a job, plus optional supporting artifacts.

## Output location

All specs go in `.ship-session/specs/<feature>/<spec-name>/spec.md`. Create directories as needed.

- `<feature>` maps to a doc skill name (e.g., `ship-skills`, `ship-runtime`) matching `docs/ship-*/`.
- `<spec-name>` is the specific capability or work item. Must match the `slug` in frontmatter.
- Supporting artifacts (mockups, diagrams, notes) are peers of `spec.md` in the same directory.

## Plan format

```markdown
---
slug: eval-runner
feature: ship-skills
agent: rust-runtime
priority: 2
mode: autonomous
depends_on: []
---

# <Title>

## Goal

<One paragraph describing the outcome.>

## File scope

<Directories and files the agent may modify.>

## What to change

<Specific instructions. Be precise — the agent starts from zero context.>

## Acceptance criteria

1. <Verifiable outcome>
2. <Verifiable outcome>
```

## Frontmatter fields

| Field | Required | Description |
|-------|----------|-------------|
| `slug` | yes | Branch and worktree name: `job/<slug>`. Must match directory name. |
| `feature` | yes | Doc skill namespace (e.g., `ship-skills`, `ship-runtime`). Matches `docs/ship-*/`. |
| `agent` | yes | Ship agent profile to run the job |
| `priority` | no | 1 (highest) to 5 (lowest). Default 3. |
| `mode` | no | `autonomous` (default) or `interactive` |
| `depends_on` | no | Array of slugs that must complete first |

## Rules

- **One spec per directory.** Directory name matches slug.
- **Supporting artifacts** (diagrams, mockups, notes) are peers of `spec.md` in the same directory.
- **Feature namespace** matches a doc skill name from `docs/ship-*/`.
- **Acceptance criteria must be verifiable** without asking a human. "Improve X" is not a criterion. "X passes test Y" is.
- **File scope is a contract.** The agent must not modify files outside scope. Gate enforces this.
- **Goal is one paragraph.** If it takes more, the job is too big — split it.
- **No implementation details in goals.** Say what, not how. The agent decides how.

## Agent selection

Pick the agent whose specialty matches the work:

{% if runtime.agents %}
{% for a in runtime.agents %}- **{{ a.id }}**{% if a.description %} — {{ a.description }}{% endif %}
{% endfor %}
{% else %}
See `.ship/agents/` for available profiles.
{% endif %}

## Test/implementation split

For feature work, create two plans with a dependency:

```
.ship-session/specs/ship-runtime/auth-tests/spec.md    (agent: test-writer)
.ship-session/specs/ship-runtime/auth-impl/spec.md     (agent: rust-runtime, depends_on: [auth-tests])
```

## Launching

Plans are launched by:
1. **Human** — from Studio UI or by telling an agent "launch the auth-tests plan"
2. **Agent** — calls `create_job` with the plan path

The daemon handles worktree creation, spec delivery, and terminal spawn. The planner never dispatches directly.
