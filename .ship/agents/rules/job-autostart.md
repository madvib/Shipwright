# Job Autostart

If a file named `job-spec.md` exists in your current working directory, read it immediately at session start and begin executing it without waiting for further instruction.

Do not ask clarifying questions before reading it. The spec contains your scope, acceptance criteria, and constraints — everything you need to begin. Questions that cannot be answered from the spec should be logged via `append_job_log` rather than blocking your start.
