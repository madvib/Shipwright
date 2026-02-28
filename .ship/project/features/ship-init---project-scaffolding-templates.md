+++
id = "97f2564e-45bc-408b-babf-cc06f8b3dfd8"
title = "Ship Init — Project Scaffolding Templates"
status = "in-progress"
created = "2026-02-27T15:03:26.611690765Z"
updated = "2026-02-27T15:03:26.611690765Z"
adr_ids = []
tags = []
+++

+++
title = "Ship Init — Project Scaffolding Templates"
status = "in-progress"
created = "2026-02-27T00:00:00Z"
updated = "2026-02-27T00:00:00Z"
release_id = "v0.1.0-alpha.md"
+++

## Why

`ship init` initializes the `.ship/` project overlay but does nothing to scaffold the underlying project. Teams starting a new project have to separately run `npx create-next-app`, `cargo new`, etc., then come back and wire up Ship. The template flag closes this gap: one command sets up both the project structure and Ship's agent context for it.

The agent does the actual scaffolding work — Ship puts the right skill + prompt in front of it. No code generation in Ship itself.

## Acceptance Criteria

- [ ] `ship init --template <name>` accepted by CLI
- [ ] Built-in templates: `nextjs`, `rust-cli`, `tauri` (alpha)
- [ ] Template = bundled skill + optional prompt + optional ship.toml defaults
- [ ] Skill describes how to work in this kind of project (stack idioms, dev commands, conventions)
- [ ] Prompt (optional) is a one-shot scaffold instruction the agent executes on first run
- [ ] `ship init --list-templates` lists available templates with descriptions
- [ ] User-defined templates: `.ship/templates/<name>/` directory with same structure
- [ ] `ship template create <name>` scaffolds a new custom template
- [ ] Init output tells user "run `ship prompt run scaffold` to scaffold your project"
- [ ] Templates do not run code — they configure agent context only (security boundary)

## Delivery Todos

- [ ] Add `--template` and `--list-templates` flags to `Commands::Init`
- [ ] Define `ProjectTemplate` struct: id, name, description, skill content, prompt content, ship_toml_defaults
- [ ] Bundle built-in templates as `include_str!` assets in runtime crate
- [ ] `apply_template(ship_dir, template)` — writes skill + prompt, merges toml defaults
- [ ] Wire into `init_project` or as post-init step in CLI handler
- [ ] Tests: init with template seeds correct skill + prompt files; list-templates output; custom template from directory

## Notes

**Templates are agent context, not code generators.** The nextjs template does not run `npx create-next-app`. It writes a skill that tells the agent: "This is a Next.js project. Use App Router. Run `npm run dev` to start. Components go in `src/components/`. Pages in `src/app/`." Then an optional scaffold prompt says "Initialize a new Next.js project in this directory using App Router and TypeScript." The agent executes that.

This keeps Ship out of the code-generation business while still dramatically improving the new-project experience.

Future: community template registry (V2). Alpha ships 3 built-ins + custom directory support.
