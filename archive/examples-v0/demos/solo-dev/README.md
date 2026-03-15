# Story: Solo Developer

**Persona**: Alex, an indie developer building TaskFlow — a lightweight task
management SaaS. Working alone with Claude as their AI pair.

**Arc**: Project initialization → planning a release → breaking it into
features and specs → creating work items → configuring the agent → running a
session → shipping.

## What this demonstrates

- `ship init` bootstrapping a project with git
- Release → Feature → Spec → Issue lifecycle (the core planning spine)
- Custom workflow modes to scope agent capabilities
- Skill creation to give the agent project context
- Provider detection and config export to Claude
- Session lifecycle: start, log progress, end

## Key insight

Ship treats your project planning artifacts as **committed memory** — releases,
features, specs, ADRs, and agent config all live in `.ship/` alongside your
code. Your AI agent always has full project context without you having to
paste it into every chat.

## Run it

```bash
bash examples/demos/solo-dev/story.sh
```
