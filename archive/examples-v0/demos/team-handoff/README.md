# Story: Team Handoff

**Personas**:
- **Sam** — senior dev who set up the project and has been running it for two
  months. Has context, rules, skills, and agent config dialed in.
- **Alex** — new contributor joining the project. Needs to get productive fast.

**Arc**: Sam prepares the handoff (committed state, agent config, rules, skills,
handoff note) → Alex clones the repo → Alex activates the project and is
immediately productive with full AI context.

## What this demonstrates

- **`.ship/` as committed project memory**: planning artifacts, agent config,
  rules, and skills are all tracked in git. When Alex clones the repo, they
  get everything.
- **`ship skill` for onboarding**: Sam writes a skill that gives any AI session
  full project context — stack, conventions, gotchas, links to docs.
- **Rules as guardrails**: Sam's `rules/` files apply to every agent session,
  preventing the AI from going off-script even for a new contributor.
- **Agent config as shared infrastructure**: modes, MCP server config, and
  provider export setup are committed. Alex runs one export command and
  their local AI client is configured identically to Sam's.

## Key insight

Institutional knowledge is usually locked in people's heads or scattered
across wikis. Ship makes it **executable** — the AI agent in every session
automatically has the team's context, constraints, and conventions, enforced
at the runtime level.

## Run it

```bash
bash examples/demos/team-handoff/story.sh
```
