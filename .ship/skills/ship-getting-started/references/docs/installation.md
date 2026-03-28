---
group: Getting Started
title: Installation
section: guide
order: 2
---

# Installation

## Quick install

```bash
curl -fsSL https://getship.dev/install | sh
```

This downloads the Ship binary, installs it, and adds it to your PATH. Works on macOS, Linux, and Windows (WSL).

Verify:

```bash
ship --version
```

## Install from source

If you prefer to build from source:

```bash
git clone https://github.com/madvib/ship.git
cd ship
cargo install --path apps/ship-studio-cli
```

Requires the [Rust toolchain](https://rustup.rs) (stable channel).

## Next steps

Continue to [Your First Project](./first-project) to set up Ship in your project.
