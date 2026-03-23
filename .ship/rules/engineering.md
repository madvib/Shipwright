# Engineering

* Write the failing test before the implementation.
* Add or update tests for every bug fix and behavior change.
* Cover happy paths and meaningful failure paths.
* Keep error messages actionable and specific. No silent fallbacks.
* Keep command behavior idempotent where practical.
* Keep Rust domain logic in runtime/modules. Keep CLI/MCP transport thin.
* Keep React component state and API contracts explicit and stable.
* Review changes for regressions, architecture drift, and missing tests before merge.
* File length cap: 300 lines. If it needs more, split it.
* Stage explicit files only. Keep commit subjects imperative and concise.
* Commit types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`.
* No AI attribution or co-author noise in commit messages.
* One way to do things. One auth system (Better Auth), one parser (WASM compiler), one migration tool (drizzle-kit). If a solution exists, use it. Do not build a parallel system.
* No backward compatibility without downstream consumers. Make hard breaks.
* Only allow temporary compat for data-safety migrations. Must include explicit scope, removal criteria, and a test that fails once the exception is no longer needed.

