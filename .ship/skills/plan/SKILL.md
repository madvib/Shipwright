---
name: plan
stable-id: plan
description: Create structured job plans in .ship-session/plans/. Plans become launchable jobs via the daemon.
tags: [planning, jobs]
authors: [ship]
---

# Plan

Write job specs to `.ship-session/plans/`. Each plan is a markdown file with YAML frontmatter that the daemon can launch as a job.

## Output location

All plans go in `.ship-session/plans/<slug>.md`. Create the directory if it doesn't exist.

## Plan format

```markdown
---
slug: <short-name>
agent: <agent-profile>
model: <model-id or null>
provider: <provider or null>
mode: autonomous | interactive
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
| `slug` | yes | Branch and worktree name: `job/<slug>` |
| `agent` | yes | Ship agent profile to run the job |
| `model` | no | Model override (e.g., `sonnet`, `opus`) |
| `provider` | no | Provider override (e.g., `anthropic`, `openai`) |
| `mode` | no | `autonomous` (default) or `interactive` |
| `depends_on` | no | Array of slugs that must complete first |

## Rules

- **One plan per file.** Name matches slug: `<slug>.md`.
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
.ship-session/plans/auth-tests.md    (agent: test-writer)
.ship-session/plans/auth-impl.md     (agent: rust-runtime, depends_on: [auth-tests])
```

## Launching

Plans are launched by:
1. **Human** — from Studio UI or by telling an agent "launch the auth-tests plan"
2. **Agent** — calls `create_job` with the plan path

The daemon handles worktree creation, spec delivery, and terminal spawn. The planner never dispatches directly.
