---
name: GitHub Actions
description: GitHub Actions CI/CD patterns including caching, matrix builds, and security
tags: [ci, cd, github-actions, automation, workflows]
---

# GitHub Actions

## Workflow Structure

```yaml
name: CI

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  lint:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
      - run: npm ci
      - run: npm run lint

  test:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
      - run: npm ci
      - run: npm test
```

### Key Rules

| Rule | Why |
|------|-----|
| Set `timeout-minutes` on every job | Prevent runaway jobs from burning minutes |
| Use `concurrency` with `cancel-in-progress` | Cancel outdated PR runs when new commits push |
| Pin action versions | Prevent supply chain attacks from tag mutations |
| Separate lint and test jobs | Fail fast on lint, run tests in parallel |

## Caching

### Dependency Caching

Most `setup-*` actions have built-in caching. Use it.

```yaml
- uses: actions/setup-node@v4
  with:
    node-version: 20
    cache: npm    # automatically caches ~/.npm

- uses: actions/setup-python@v5
  with:
    python-version: "3.12"
    cache: pip    # automatically caches pip packages
```

### Custom Caching

For build artifacts, compiled outputs, or tools:

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-
```

### Cache Key Strategy

```
key: ${{ runner.os }}-<tool>-${{ hashFiles('<lockfile>') }}
restore-keys: ${{ runner.os }}-<tool>-
```

The `restore-keys` fallback ensures a partial cache hit when the lockfile changes.

## Matrix Builds

Test across multiple versions or platforms:

```yaml
test:
  runs-on: ${{ matrix.os }}
  timeout-minutes: 10
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest]
      node-version: [18, 20, 22]
    fail-fast: false
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
      with:
        node-version: ${{ matrix.node-version }}
    - run: npm ci
    - run: npm test
```

Set `fail-fast: false` to see all failures, not just the first.

## Conditional Steps

```yaml
steps:
  - name: Deploy to production
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    run: ./deploy.sh production

  - name: Deploy preview
    if: github.event_name == 'pull_request'
    run: ./deploy.sh preview
```

### Common Conditions

| Condition | Expression |
|-----------|-----------|
| Main branch push | `github.ref == 'refs/heads/main' && github.event_name == 'push'` |
| Pull request | `github.event_name == 'pull_request'` |
| Tagged release | `startsWith(github.ref, 'refs/tags/v')` |
| Previous step failed | `failure()` |
| Always run (cleanup) | `always()` |

## Secrets

```yaml
env:
  DATABASE_URL: ${{ secrets.DATABASE_URL }}
  API_KEY: ${{ secrets.API_KEY }}
```

### Secrets Rules

- Never echo or print secrets in logs
- Use environment-level secrets for environment-specific values
- Rotate secrets regularly (at minimum on team member departure)
- Use `GITHUB_TOKEN` (automatically provided) for GitHub API calls

## Reusable Workflows

Extract common patterns into reusable workflows:

```yaml
# .github/workflows/deploy.yml
on:
  workflow_call:
    inputs:
      environment:
        required: true
        type: string
    secrets:
      deploy_key:
        required: true

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: ${{ inputs.environment }}
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - run: ./deploy.sh ${{ inputs.environment }}
        env:
          DEPLOY_KEY: ${{ secrets.deploy_key }}
```

## Anti-Patterns

| Anti-Pattern | Problem | Fix |
|-------------|---------|-----|
| `runs-on: ubuntu-latest` without pinning | Runner image changes break builds | Accept the tradeoff or pin to specific version |
| No timeout | Hung jobs run for 6 hours | Set `timeout-minutes` |
| `actions/checkout@main` | Tag can be force-pushed to malicious code | Pin to commit SHA |
| Single monolithic job | Slow feedback, no parallelism | Split into lint, test, build, deploy |
| Secrets in workflow file | Visible to anyone reading the repo | Use GitHub Secrets |
| No concurrency control | Multiple deploys race | Use `concurrency` groups |

## Checklist

- [ ] Every job has `timeout-minutes` set
- [ ] Action versions pinned (SHA for third-party, tag for official)
- [ ] Dependency caching configured
- [ ] `concurrency` with `cancel-in-progress` for PR workflows
- [ ] Secrets stored in GitHub Secrets, not in workflow files
- [ ] Lint and test jobs separated for parallel execution
- [ ] Deployment steps are conditional on branch/event
- [ ] Matrix builds cover supported versions
