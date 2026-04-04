# Skills

This guide is maintained as the `ship-skills-guide` skill. Agents get it in their compiled context automatically.

For the full guide — SKILL.md format, directory structure, namespace resolution, writing tips, and publishing — see [.ship/skills/ship-skills-guide/SKILL.md](../.ship/skills/ship-skills-guide/SKILL.md).

## Quick start

```bash
ship skill create my-skill        # scaffold .ship/skills/my-skill/SKILL.md
ship skill add github.com/owner/repo  # install from registry
ship skill list                   # see what's installed
ship skill remove my-skill        # remove
```

## Ship's exported skills

Ship publishes 11 public skills via `ship add github.com/madvib/ship`:

| Skill | Purpose |
|-------|---------|
| ship-cli-reference | CLI command reference |
| ship-schema-reference | Config file format reference |
| ship-skills-guide | How to write and publish skills |
| ship-help | Troubleshooting |
| ship-tutorial | Interactive onboarding |
| configure-agent | Agent workspace setup |
| find-skills | Discover skills from the ecosystem |
| write-adr | Architecture decision records |
| tdd | Test-driven development |
| visual-brainstorm | HTML mockup generation |
| visual-spec | Design spec extraction |
