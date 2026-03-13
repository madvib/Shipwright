# Core Principles

* Keep operations deterministic: same inputs and state should produce the same outputs.

* Prefer explicit failures over silent fallback; surface actionable errors.

* Make state transitions observable through events/logs and durable records.

* Keep business logic in runtime/modules; keep transport layers thin.

* Require tests for behavior changes and bug fixes before marking work done.

* Treat migration and sync steps as idempotent by default.

