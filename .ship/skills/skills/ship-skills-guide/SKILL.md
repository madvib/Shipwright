---
name: ship-skills-guide
description: How to create, structure, and publish Ship skills. Use when users want to write a new skill, understand the SKILL.md format, organize skill directories, add frontmatter metadata, or prepare skills for publishing to the registry. Covers the agentskills.io specification.
tags: [guide, skills, authoring, documentation]
authors: [ship]
---

# Ship Skills Guide

A skill is a markdown document that teaches an agent how to do something. Skills are compiled into provider-specific config (CLAUDE.md, .cursor/rules, etc.) by `ship use` and loaded at agent startup. Each skill covers one concern: a protocol, a reference, a workflow. Skills follow the [agentskills.io](https://agentskills.io) specification.

## SKILL.md Format

Every skill is a file named `SKILL.md` with YAML frontmatter followed by a markdown body.

### Frontmatter fields

```yaml
---
name: my-skill
description: One sentence explaining what this skill does and when to use it.
tags: [category, domain, purpose]
authors: [your-handle]
license: MIT
compatibility: claude, cursor, codex
allowed-tools: create_job update_job log_progress
metadata:
  version: 0.1.0
  author: Your Name
---
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Human-readable skill name. Must match the directory name (lowercase, hyphens, digits, 1-64 chars). No leading/trailing hyphens, no double hyphens. |
| `description` | yes | When to invoke this skill. Be specific about trigger conditions -- agents use this to decide whether to activate the skill. |
| `tags` | no | Category labels for discovery. Bracket-delimited list. |
| `authors` | no | Who wrote the skill. Bracket-delimited list. |
| `license` | no | SPDX license identifier (e.g. `MIT`, `Apache-2.0`). |
| `compatibility` | no | Comma-separated provider names the skill is designed for. Omit to indicate universal compatibility. |
| `allowed-tools` | no | Space-delimited list of MCP tool names the skill requires. Used for permission auditing -- the compiler warns if the agent profile does not grant these tools. |
| `metadata` | no | Arbitrary key-value pairs. Indented under `metadata:`. Legacy `version` and `author` top-level keys are folded into metadata automatically. |

The frontmatter parser is line-based, not a full YAML parser. One key per line. No multi-line values except indented lines under `metadata:`.

### Markdown body

Everything below the closing `---` is the skill content. This is compiled verbatim into the provider's instruction format. Write it as if you are briefing a capable colleague.

## Directory Structure

### Single skill

The standard layout. One directory, one `SKILL.md`.

```
.ship/agents/skills/
  my-skill/
    SKILL.md
```

The directory name is the skill id. The loader reads `<skill-id>/SKILL.md` and uses the directory name as the id. The `name` field in frontmatter provides the human-readable name.

### Namespace (multiple sub-skills)

A namespace directory contains sub-skill directories instead of a `SKILL.md` at its own level. The namespace itself has no `SKILL.md`. Each sub-directory has one.

```
.ship/agents/skills/
  better-auth/               # namespace -- no SKILL.md here
    emailAndPassword/
      SKILL.md
    twoFactor/
      SKILL.md
    socialProviders/
      SKILL.md
```

Namespace expansion applies to dep refs only. When a dep ref like `github.com/better-auth/skills/better-auth` points to a directory without `SKILL.md` but containing sub-directories that each have `SKILL.md`, the resolver expands it to all leaf skills. Local skills in `.ship/agents/skills/` are loaded individually -- each sub-directory with a `SKILL.md` becomes its own skill.

### Flat format (legacy)

A single `.md` file directly in the skills directory (e.g. `skills/my-skill.md`) also works. The filename minus extension becomes the id. Prefer the directory format for new skills -- it allows bundled resources.

## How Skills Get Loaded

### Local refs

Local skills live at `.ship/agents/skills/<skill-id>/SKILL.md`. The loader scans every subdirectory of `agents/skills/`, reads `SKILL.md` if present, parses frontmatter, and produces a `Skill` value. No lock file or cache involved.

### Dep refs

Dep skills are referenced by their full package path:

```
github.com/owner/package/skill-name
```

Resolution path:
1. Split into package path (`github.com/owner/package`) and within-package path (`skill-name`)
2. Look up the package in `ship.lock` to get the `sha256:<hex>` content hash
3. Find the cached content at `~/.ship/cache/objects/<hex>/<within-path>`
4. If `<within-path>/SKILL.md` exists, load it as a single skill
5. If `<within-path>/` is a namespace (sub-dirs with `SKILL.md`), expand to all leaf skills
6. If neither, error with a list of available skills in that package

Dep skills require `ship install` to populate the cache. If the package is not in `ship.lock`, the resolver errors with an actionable message.

### Profiles control which skills are active

Skills exist in the library but are activated per-profile. An agent profile's `[skills]` section lists which skill ids to include. Modes can further filter the active skill set.

## Bundled Resources

Skills that need supporting files use subdirectories within the skill directory:

```
my-skill/
  SKILL.md
  references/        # reference docs, specs, API tables
  scripts/           # helper scripts the skill instructs agents to run
  assets/            # images, templates, config snippets
```

Reference supporting files from the skill body using relative paths. The skill directory is self-contained -- everything the skill needs to function should be co-located.

## Writing Tips

**Be concise.** Agents have finite context windows. Every line should earn its place. Cut preamble, cut filler, cut "in this skill you will learn."

**Use imperative voice.** "Write the test first" not "The developer should write the test first." You are giving instructions, not describing a process.

**Explain why, not just what.** "Commit at green because each passing test is a stable checkpoint" beats "Commit after tests pass." The why survives context loss; the what alone does not.

**Include concrete examples.** Show the command. Show the file structure. Show the output. Agents follow examples more reliably than abstract descriptions.

**One skill, one concern.** A skill that does TDD, deployment, and code review is three skills. Split them. Focused skills compose better than monoliths.

**Name MCP tools canonically.** Use `create_job`, not `createJob` or `Create Job`. The compiler matches tool names exactly.

**Front-load the trigger condition** in the `description` field. Agents scan descriptions to decide whether a skill is relevant. "Use when the user asks to write a new test" is better than "A comprehensive testing methodology framework."

**Validate naming.** Skill ids must be lowercase alphanumeric with hyphens. 1-64 characters. No leading/trailing hyphens. No consecutive hyphens. `my-skill` is valid. `My_Skill` is not.

## Publishing

Skills become available to other projects through the registry. The path from local skill to published skill:

### 1. Declare the module

Your `ship.toml` must have a `[module]` section with package identity:

```toml
[module]
name = "github.com/your-org/your-package"
version = "0.1.0"
description = "What this package provides"
license = "MIT"
```

### 2. Export the skills

List the skills you want to publish under `[exports]`:

```toml
[exports]
skills = [
    "agents/skills/my-skill",
    "agents/skills/another-skill",
]
```

Paths are relative to the `.ship/` directory. Only exported skills are available to consumers. Private skills (project-specific protocols, internal conventions) should be omitted from exports.

### 3. Publish

```bash
ship publish
```

This pushes the package to the registry. Consumers install with:

```bash
ship skill add github.com/your-org/your-package
```

The resolver scans the package for all exported skills automatically.

### What not to export

Skills that encode project-specific conventions (your team's branching strategy, your deployment pipeline, your internal tools) are private. Export skills that provide value to any project in the domain -- testing protocols, framework guides, common workflow patterns.
