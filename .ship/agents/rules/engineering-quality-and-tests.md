# Engineering Quality And Tests

* Write the failing test before the implementation. If you didn't watch it fail, you don't know if it tests the right thing. Use the `tdd` skill for the full protocol.

* Add or update tests for every bug fix and behavior change.

* Cover happy paths and meaningful failure paths.

* Keep error messages actionable and specific.

* Avoid silent fallbacks that hide broken state.

* Keep command behavior idempotent where practical.

* Keep Rust domain logic in runtime/modules and keep CLI/UI transport thin.

* Keep React component state and API contracts explicit and stable.

* Review changes for regressions, architecture drift, and missing tests before merge.

* Stage explicit files only; keep commit subjects imperative and concise.

* Use commit types consistently: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`.

* Do not add AI attribution or co-author noise to commit messages.

