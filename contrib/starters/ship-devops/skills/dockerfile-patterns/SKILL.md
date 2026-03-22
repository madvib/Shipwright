---
name: Dockerfile Patterns
description: Production-ready Dockerfile patterns with multi-stage builds, security, and optimization
tags: [docker, containerization, devops, security]
---

# Dockerfile Patterns

## Multi-Stage Build

Every compiled language project must use multi-stage builds. The final image contains only the runtime and the binary.

### Node.js Example

```dockerfile
# Stage 1: Build
FROM node:20.11-slim AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts
COPY . .
RUN npm run build

# Stage 2: Production
FROM node:20.11-slim
WORKDIR /app
RUN addgroup --system app && adduser --system --ingroup app app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY package.json ./
USER app
EXPOSE 3000
CMD ["node", "dist/index.js"]
```

### Rust Example

```dockerfile
# Stage 1: Build
FROM rust:1.77-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Stage 2: Production
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN adduser --system --no-create-home app
COPY --from=builder /app/target/release/myapp /usr/local/bin/
USER app
CMD ["myapp"]
```

### Go Example

```dockerfile
# Stage 1: Build
FROM golang:1.22-alpine AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o /server ./cmd/server

# Stage 2: Production (scratch for Go static binaries)
FROM scratch
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /server /server
USER 65534
ENTRYPOINT ["/server"]
```

## Base Image Selection

| Base Image | Size | Use When |
|-----------|------|----------|
| `scratch` | 0 MB | Go static binaries, no shell needed |
| `distroless` | ~2 MB | Need SSL certs but no shell |
| `alpine` | ~5 MB | Need a package manager and shell |
| `debian-slim` | ~80 MB | Need glibc compatibility |
| `ubuntu` | ~80 MB | Need specific Ubuntu packages |

Always specify exact version tags. Never use `latest`.

```dockerfile
# Good
FROM node:20.11-slim
FROM python:3.12-slim-bookworm
FROM golang:1.22-alpine3.19

# Bad
FROM node
FROM python:latest
FROM golang:alpine
```

## Layer Optimization

Docker caches layers top-down. Put rarely changing operations first.

```dockerfile
# Good — dependencies cached separately from source code
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

# Bad — any source change invalidates the npm ci cache
COPY . .
RUN npm ci
RUN npm run build
```

### Layer Rules

| Rule | Rationale |
|------|-----------|
| Copy lockfile first, install deps, then copy source | Dependency layer is cached unless lockfile changes |
| Combine related RUN commands with `&&` | Reduces layer count and image size |
| Clean up in the same RUN as install | Deleted files persist in earlier layers |
| Use `.dockerignore` | Prevents copying `node_modules`, `.git`, build artifacts |

```dockerfile
# Good — cleanup in same layer
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Bad — cleanup in separate layer (delete does not save space)
RUN apt-get update && apt-get install -y ca-certificates
RUN rm -rf /var/lib/apt/lists/*
```

## Security

### Non-Root User

Never run containers as root in production.

```dockerfile
RUN addgroup --system app && adduser --system --ingroup app app
USER app
```

### .dockerignore

```
.git
node_modules
*.md
.env
.env.*
docker-compose*.yml
.github
coverage
dist
```

### Security Checklist

- [ ] Final image runs as non-root user
- [ ] Base image pinned to exact version
- [ ] No secrets in Dockerfile or build args
- [ ] `.dockerignore` excludes `.git`, `node_modules`, `.env`
- [ ] Installed packages have `--no-install-recommends` (Debian) or `--no-cache` (Alpine)
- [ ] No `curl | bash` patterns (download, verify, then execute)

## Health Checks

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1
```

## Image Size Monitoring

Keep production images small. Investigate if an image exceeds 500 MB.

```bash
# Check image size
docker images myapp

# Analyze layers
docker history myapp:latest

# Deep dive with dive tool
dive myapp:latest
```

### Size Reduction Strategies

| Strategy | Savings |
|----------|---------|
| Multi-stage build | 50-90% (removes build tools) |
| Slim/alpine base | 50-80% vs full image |
| `.dockerignore` | Variable (prevents bloat from dev files) |
| `--no-install-recommends` | 10-30% on apt-get |
| Static linking (Go, Rust) + scratch | Up to 95% |

## Checklist

- [ ] Multi-stage build for compiled languages
- [ ] Base image version pinned (not `latest`)
- [ ] Non-root user in production image
- [ ] `.dockerignore` configured
- [ ] Dependencies installed before source code copied
- [ ] Cleanup done in same RUN layer as install
- [ ] Health check configured
- [ ] Image size under 500 MB
