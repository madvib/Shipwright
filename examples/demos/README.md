# Ship Demo Scenarios

Runnable end-to-end scenarios that walk through real Ship workflows. Each story
sets up an isolated temporary workspace, runs through a complete narrative arc,
and prints annotated output so you can follow along.

## Demos

| Demo | Description | Key Features |
|-------|-------------|--------------|
| [solo-dev](./solo-dev/) | Solo developer shipping a v0.1.0 release | Init, release/feature lifecycle, ADRs, skills, modes, agent export |
| [multi-provider](./multi-provider/) | Dev with Claude + Codex + Gemini configured | Provider detection, mode-per-workflow, multi-client export |
| [team-handoff](./team-handoff/) | Senior dev handing off to a new contributor | Committed project state, rules, skills, agent config as shared context |

## Running

Each demo is self-contained. From the repo root:

```bash
# Build Ship CLI first (once)
cargo build -p cli

# Run any demo
bash examples/demos/solo-dev/story.sh
bash examples/demos/multi-provider/story.sh
bash examples/demos/team-handoff/story.sh
```

Pass `--skip-build` to skip the cargo build step if the binary is already fresh:

```bash
bash examples/demos/solo-dev/story.sh --skip-build
```

## Design Principles

- **Isolated**: each story creates its own temp workspace under `.tmp/` and
  sets `HOME` to a sandboxed directory so global Ship config is not polluted
- **Narrative**: output is structured like a story with scene headings and
  commentary explaining *why* each step matters
- **Runnable**: every command is real — no mocking, no stubs
- **Inspectable**: set `KEEP_TMP=1` to preserve the temp workspace after the run
  so you can poke around with `ship` commands
