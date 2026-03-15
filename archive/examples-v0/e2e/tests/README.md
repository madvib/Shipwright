# E2E Test Map

This folder contains the automated end-to-end suite for Ship.

- `workflow.rs` - baseline project init/layout and namespace behavior.
- `entity_cli.rs` - CRUD flows and CLI surface compatibility checks.
- `workspace_lifecycle.rs` - workspace + session lifecycle behavior.
- `agent_config.rs` - provider config export/import and agent surface checks.
- `compiler_matrix.rs` - multi-provider compilation against fixture projects.
- `branch_config.rs` - branch-aware config behavior.
- `git_hook.rs` - hook install + invocation checks.
- `git_workflow.rs` - git-centric workflows and cross-branch behavior.
- `project_registry.rs` - project registration/discovery behaviors.
- `helpers/mod.rs` - shared test harness and fixture utilities.
