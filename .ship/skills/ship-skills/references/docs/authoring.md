---
group: Smart Skills
title: Authoring and Publishing
description: How to write, structure, and publish Ship skills. Covers naming, loading, namespaces, and the getship.dev registry.
audience: public
section: guide
order: 5
---

# Authoring and Publishing

This page covers writing new skills, how skills get loaded, namespace directories, and publishing to the registry.

## Writing tips

**Be concise.** Agents have finite context windows. Every line should earn its place. Cut preamble, cut filler, cut "in this skill you will learn."

**Use imperative voice.** "Write the test first" not "The developer should write the test first." You are giving instructions, not describing a process.

**Explain why, not just what.** "Commit at green because each passing test is a stable checkpoint" beats "Commit after tests pass." The why survives context loss; the what alone does not.

**Include concrete examples.** Show the command. Show the file structure. Show the output. Agents follow examples more reliably than abstract descriptions.

**One skill, one concern.** A skill that does TDD, deployment, and code review is three skills. Split them. Focused skills compose better than monoliths.

**Name MCP tools canonically.** Use `create_job`, not `createJob` or `Create Job`. The compiler matches tool names exactly.

**Front-load the trigger condition** in the `description` field. Agents scan descriptions to decide whether a skill is relevant. "Use when the user asks to write a new test" is better than "A comprehensive testing methodology framework."

**Validate naming.** Skill ids must be lowercase alphanumeric with hyphens. 1-64 characters. No leading/trailing hyphens. No consecutive hyphens. `my-skill` is valid. `My_Skill` is not.

## Namespace directories

A namespace directory contains sub-skill directories instead of a `SKILL.md` at its own level. The namespace itself has no `SKILL.md`. Each sub-directory has one.

```
.ship/skills/
  better-auth/               # namespace -- no SKILL.md here
    emailAndPassword/
      SKILL.md
    twoFactor/
      SKILL.md
    socialProviders/
      SKILL.md
```

Namespace expansion applies to dep refs only. When a dep ref points to a directory without `SKILL.md` but containing sub-directories that each have `SKILL.md`, the resolver expands it to all leaf skills. Local skills in `.ship/skills/` are loaded individually -- each sub-directory with a `SKILL.md` becomes its own skill.

## Flat format (legacy)

A single `.md` file directly in the skills directory (e.g. `skills/my-skill.md`) also works. The filename minus extension becomes the id. Prefer the directory format for new skills -- it allows bundled resources.

## How skills get loaded

### Local refs

Local skills live at `.ship/skills/<skill-id>/SKILL.md`. The loader scans every subdirectory of `skills/`, reads `SKILL.md` if present, parses frontmatter, and produces a `Skill` value. No lock file or cache involved.

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

### Agent activation

Skills exist in the library but are activated per-agent. An agent profile's `"skills"` section lists which skill ids to include.

## Publishing

Skills become available to other projects through the registry. The path from local skill to published skill:

### 1. Declare the module

Your `ship.jsonc` must have a `"module"` section with package identity:

```jsonc
{
  "module": {
    "name": "github.com/your-org/your-package",
    "version": "0.1.0",
    "description": "What this package provides",
    "license": "MIT"
  }
}
```

### 2. Export the skills

List the skills you want to publish under `"exports"`:

```jsonc
{
  "exports": {
    "skills": ["skills/my-skill", "skills/another-skill"]
  }
}
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
