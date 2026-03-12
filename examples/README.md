# Examples

This directory is split by purpose so it is obvious what is executable demo
content vs automated verification.

## Layout

- [`projects/`](./projects/) - fixture/example project roots used by tests and
  manual experiments.
- [`e2e/`](./e2e/) - automated end-to-end suite (Rust integration tests + shell
  checks).
- [`demos/`](./demos/) - guided runnable walkthrough scripts.

## Run E2E

```bash
cargo test -p examples-e2e
./examples/e2e/checks/project-features.sh
```

## Run Demos

```bash
cargo build -p cli
bash examples/demos/solo-dev/story.sh
bash examples/demos/multi-provider/story.sh
bash examples/demos/team-handoff/story.sh
```
