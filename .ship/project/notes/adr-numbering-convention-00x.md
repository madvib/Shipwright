+++
id = "cf0cce85-34d9-48b2-8c27-ae45a228b5f4"
title = "ADR numbering convention (#00X)"
created = "2026-02-27T04:17:25Z"
updated = "2026-02-27T04:17:25Z"
tags = ["adr", "convention", "follow-up"]
+++

Summary: Adopt explicit ADR numbering (`#001`, `#002`, ...) so decision history is stable, sortable, and easy to reference across docs and tooling.

Scope for later:
- Define filename and title convention (for example `adr-001-...md` with visible `#001` label).
- Auto-assign the next sequence number during ADR creation in CLI, MCP, and UI.
- Handle renamed or archived ADRs without reusing sequence numbers.

Status: Deferred follow-up after current UI polish cycle.
