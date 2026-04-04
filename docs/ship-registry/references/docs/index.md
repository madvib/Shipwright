---
group: Registry
title: Registry
order: 1
---

# Registry

The registry is a package system for distributing skills and agents. Packages are Git repositories with a `.ship/ship.jsonc` manifest. In v0.1, resolution happens directly against Git remotes -- no central server required.

## Package structure

A package is any Git repository containing `.ship/ship.jsonc` with a `module` section:

```jsonc
{
  "module": {
    "name": "github.com/yourorg/your-package",
    "version": "0.1.0",
    "description": "What this package provides",
    "license": "MIT"
  }
}
```

The `name` field is the package path. It doubles as the Git clone URL (`https://github.com/yourorg/your-package.git`). The `version` follows semver and corresponds to Git tags (`v0.1.0` resolves to version `0.1.0`).

## Exports

The `exports` section declares what consumers receive when they install the package:

```jsonc
{
  "exports": {
    "skills": [
      "skills/tdd",
      "skills/code-review"
    ],
    "agents": [
      "agents/reviewer.jsonc"
    ]
  }
}
```

Paths are relative to `.ship/`. Anything not listed in `exports` stays private -- it works locally but is not installed by consumers.

Here is a real example from the Ship project's manifest, which exports 18 skills and 10 agents:

```jsonc
{
  "exports": {
    "skills": [
      "skills/no-slop",
      "skills/browse",
      "skills/tdd",
      "skills/code-review",
      "skills/ship-tutorial"
    ],
    "agents": [
      "agents/mission-control.jsonc",
      "agents/red-green.jsonc",
      "agents/reviewer.jsonc"
    ]
  }
}
```

## Dependencies

Packages can depend on other packages:

```jsonc
{
  "dependencies": {
    "github.com/better-auth/skills": "main",
    "github.com/acme/toolkit": "^1.0.0"
  }
}
```

Each key is a package path. The value is a version constraint. Ship resolves transitive dependencies automatically.

The object form grants explicit tool permissions to a dependency's skills:

```jsonc
{
  "dependencies": {
    "github.com/acme/pinned": {
      "version": "main",
      "grant": ["Bash"]
    }
  }
}
```

## Package naming

| Format | Example | Description |
|--------|---------|-------------|
| Canonical | `github.com/owner/repo` | Direct Git reference |
| Scoped | `@owner/package` | Short alias on the registry |
| Unofficial | `@unofficial/package` | Community-seeded, claimable by owner |

See [Publishing](./publishing.md) for how to publish and [Installing](./installing.md) for how to consume packages.
