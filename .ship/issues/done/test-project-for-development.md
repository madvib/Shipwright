+++
title = "Test project for development"
created = "2026-02-22T07:02:04.495522808Z"
updated = "2026-02-22T07:31:01.380183526Z"
tags = []
links = []
+++

Create a dedicated test/demo project separate from the real .ship/ data. Commands: ship demo init [path] seeds sample issues, ADRs, and log entries. Support SHIP_TEST_DIR env var for Tauri dev mode. Prevents corrupting real project data during development and testing.

## Implementation
Added crates/logic/src/demo.rs with init_demo_project(base_dir). Seeds 6 issues across all statuses, 3 ADRs, 3 log entries. CLI: ship demo [path]. Re-exported from logic::lib.rs.
