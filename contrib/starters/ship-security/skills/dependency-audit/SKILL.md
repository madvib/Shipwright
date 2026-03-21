---
name: Dependency Audit
description: Auditing third-party dependencies for vulnerabilities, maintenance, and supply chain risks
tags: [security, dependencies, supply-chain, audit, cve]
---

# Dependency Audit

## Audit Process

### Step 1: Run Automated Scanners

Run the language-appropriate audit tool first to catch known CVEs.

| Language | Command | Database |
|----------|---------|----------|
| JavaScript/TypeScript | `npm audit` or `pnpm audit` | GitHub Advisory Database |
| Python | `pip audit` or `safety check` | PyPI Advisory Database |
| Rust | `cargo audit` | RustSec Advisory Database |
| Go | `govulncheck ./...` | Go Vulnerability Database |
| Ruby | `bundle audit check --update` | Ruby Advisory Database |
| Java | `./gradlew dependencyCheckAnalyze` | NVD |

### Step 2: Review Findings

For each vulnerability reported:

| Question | Action |
|----------|--------|
| Is the vulnerability in a direct dependency? | Update to patched version |
| Is it in a transitive dependency? | Check if direct dep has a newer version that pulls patched transitive |
| Is the vulnerable code path reachable? | Check if your code uses the affected function |
| Is there no patch available? | Evaluate workarounds or alternative packages |

### Step 3: Manual Review

Automated scanners miss supply chain risks. Manually review:

1. New dependencies added since last audit
2. Dependencies with maintainer changes
3. Dependencies with suspicious version bumps

## Supply Chain Risk Assessment

### Package Trust Signals

| Signal | Green Flag | Red Flag |
|--------|-----------|----------|
| Maintainer count | 2+ active maintainers | Single maintainer who just took over |
| Download trend | Stable or growing | Sudden spike (typosquatting indicator) |
| Last release | Within 6 months | 2+ years ago |
| Open issues | Reasonable ratio, responsive | Hundreds with no response |
| License | OSI-approved (MIT, Apache, BSD) | No license or SSPL/BSL |
| Dependencies | Few, well-known | Many, unknown packages |

### Typosquatting Detection

Check for packages with names similar to popular packages:

```
lodash     --> Iodash, l0dash, lodash-utils (suspicious)
express    --> expresss, expres, express-framework (suspicious)
requests   --> request, requets (suspicious)
```

When adding a new dependency, verify the package name matches the official documentation URL.

## Severity Classification

### Critical

- Known CVE with public exploit
- Remote code execution (RCE)
- SQL injection in ORM/query builder
- Authentication bypass

### High

- Known CVE without public exploit
- Cross-site scripting (XSS) in templating library
- Path traversal in file handling library
- Denial of service in request parser

### Medium

- Prototype pollution (JavaScript)
- Regular expression denial of service (ReDoS)
- Information disclosure in error handling
- Weak cryptographic defaults

### Low

- Package unmaintained but no known vulnerabilities
- Deprecated API usage
- Unnecessary transitive dependencies (bloat)

## Dependency Update Strategy

### Version Pinning

| Strategy | Pros | Cons | Use When |
|----------|------|------|----------|
| Exact pin (`1.2.3`) | Reproducible builds | Must manually update | Libraries with breaking changes |
| Caret range (`^1.2.3`) | Gets patches and minor updates | May get unexpected changes | Trusted, semver-compliant packages |
| Tilde range (`~1.2.3`) | Gets patches only | Misses minor improvements | Cautious approach |
| Lock file only | Exact + updatable | Requires `npm update` / `cargo update` | Default for most projects |

### Update Process

```
1. Run audit tool to identify vulnerable versions
2. Check changelogs for breaking changes between current and target version
3. Update one dependency at a time (not batch)
4. Run full test suite after each update
5. Deploy to staging before production
```

## Lockfile Hygiene

### Rules

- Always commit lockfiles (`package-lock.json`, `Cargo.lock`, `poetry.lock`)
- Review lockfile diffs in PRs (look for unexpected package additions)
- Regenerate lockfile periodically to clean up orphaned entries
- Never edit lockfiles manually

### Lockfile Review Checklist

When reviewing a PR that changes the lockfile:

- [ ] Are the new packages expected (mentioned in the PR description)?
- [ ] Do new packages have reasonable download counts and maintenance signals?
- [ ] Are removed packages intentional (not accidentally dropped)?
- [ ] Do transitive dependency updates look proportional to the change?

## Reporting Template

```markdown
## Dependency Audit Report

**Date:** YYYY-MM-DD
**Scope:** [project name]
**Tool:** [audit tool + version]

### Critical Findings

| Package | Version | CVE | Severity | Fix Available | Action |
|---------|---------|-----|----------|---------------|--------|

### Maintenance Concerns

| Package | Current Version | Last Release | Issue |
|---------|----------------|--------------|-------|

### Recommendations

1. [Specific actionable recommendation]
2. [Specific actionable recommendation]
```

## Checklist

- [ ] Automated audit tool run with latest advisory database
- [ ] All critical and high vulnerabilities have remediation plan
- [ ] New dependencies reviewed for trust signals
- [ ] No typosquatting risks in package names
- [ ] Lockfile committed and reviewed
- [ ] Update strategy documented for each critical dependency
- [ ] Audit findings reported with severity and remediation
