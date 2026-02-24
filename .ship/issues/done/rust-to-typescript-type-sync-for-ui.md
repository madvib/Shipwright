+++
id = "f432f0af-0956-4f24-80c4-d40a9f458bcf"
title = "rust-to-typescript type sync for ui"
created = "2026-02-23T05:29:51.007088Z"
updated = "2026-02-23T05:29:57.053106Z"
tags = []
links = []
+++

Generate UI transport types from Rust structs so crates/ui/src/types.ts no longer drifts from logic/tauri models.

Plan: 1) Add a Rust type-export crate/tool (ts-rs or specta) for shared DTOs used by Tauri commands (Issue, IssueEntry, ADR, AdrEntry, ProjectConfig, StatusConfig, LogEntry). 2) Generate crates/ui/src/types.generated.ts from Rust in a deterministic command (pnpm gen:types). 3) Split UI-only helpers into types.ui.ts and re-export via types.ts. 4) Add CI check that regenerated types produce no diff. 5) Migrate imports to generated types and remove duplicated handwritten interfaces.
