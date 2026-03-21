---
name: OWASP Top 10
description: Systematic security audit against the OWASP Top 10 web application security risks
tags: [security, owasp, vulnerabilities, audit]
---

# OWASP Top 10 Audit Guide

## A01: Broken Access Control

Users acting outside their intended permissions.

### What to Look For

- Missing authorization checks on endpoints
- Insecure direct object references (IDOR): changing an ID in the URL exposes another user's data
- Missing function-level access control: regular user can access admin endpoints
- CORS misconfiguration allowing unauthorized origins
- Path traversal in file operations

### Audit Steps

1. List all endpoints that access user-specific data
2. For each endpoint, verify that the handler checks the requesting user owns the resource
3. Check that admin-only endpoints have role verification middleware
4. Review CORS configuration for overly permissive origins

```python
# VULNERABLE — no ownership check
@app.get("/api/documents/{doc_id}")
def get_document(doc_id: int, db: Session):
    return db.query(Document).get(doc_id)

# SECURE — ownership verified
@app.get("/api/documents/{doc_id}")
def get_document(doc_id: int, user: User, db: Session):
    doc = db.query(Document).get(doc_id)
    if doc.owner_id != user.id:
        raise HTTPException(status_code=403)
    return doc
```

## A02: Cryptographic Failures

Weak or missing encryption for sensitive data.

### Checklist

- [ ] Passwords hashed with bcrypt, scrypt, or argon2 (not MD5/SHA)
- [ ] Sensitive data encrypted at rest (database columns, file storage)
- [ ] TLS enforced for all connections (HSTS header set)
- [ ] No secrets in source code, logs, or error messages
- [ ] Tokens generated with CSPRNG, not `Math.random()` or `rand()`

## A03: Injection

Untrusted data sent to an interpreter as part of a command or query.

### SQL Injection

```sql
-- VULNERABLE: string concatenation
"SELECT * FROM users WHERE email = '" + email + "'"

-- SECURE: parameterized query
"SELECT * FROM users WHERE email = $1", [email]
```

### Command Injection

```bash
# VULNERABLE: shell interpolation
os.system(f"ping {user_input}")

# SECURE: argument list
subprocess.run(["ping", "-c", "1", user_input])
```

### NoSQL Injection

```javascript
// VULNERABLE: object from request body used directly
db.users.find({ email: req.body.email })
// Attacker sends: { "email": { "$gt": "" } }

// SECURE: validate type
const email = String(req.body.email);
db.users.find({ email });
```

## A04: Insecure Design

Flaws in the design that cannot be fixed by correct implementation.

### What to Look For

- Missing rate limiting on authentication endpoints
- No account lockout after failed login attempts
- Password reset tokens that do not expire
- Lack of server-side validation (relying on client-side only)
- No logging of security-relevant events (failed logins, permission denials)

## A05: Security Misconfiguration

Default configs, incomplete setups, open cloud storage, verbose errors.

### Checklist

- [ ] Default credentials changed
- [ ] Debug mode disabled in production
- [ ] Error messages do not expose stack traces or internal details
- [ ] Security headers set (CSP, X-Frame-Options, X-Content-Type-Options)
- [ ] Directory listing disabled on web servers
- [ ] Unused features and endpoints removed

## A06: Vulnerable and Outdated Components

Using libraries with known vulnerabilities.

### Audit Steps

1. Run dependency audit tool (`npm audit`, `cargo audit`, `pip audit`)
2. Check for packages with no releases in 12+ months
3. Verify no dependencies are pinned to versions with known CVEs
4. Review new dependency additions for necessity

## A07: Identification and Authentication Failures

Weak authentication mechanisms.

### Checklist

- [ ] Minimum password requirements enforced (length, not complexity theater)
- [ ] Multi-factor authentication available
- [ ] Session tokens invalidated on logout
- [ ] Session IDs regenerated after login (prevents session fixation)
- [ ] Brute force protection (rate limiting, account lockout)

## A08: Software and Data Integrity Failures

Assumptions about integrity of software updates, CI/CD pipelines, or deserialized data.

### What to Look For

- Unsigned or unverified software updates
- CI/CD pipeline without integrity checks
- Deserialization of untrusted data without validation
- Dependencies loaded from untrusted sources

## A09: Security Logging and Monitoring Failures

Insufficient logging to detect or respond to breaches.

### Required Log Events

| Event | Must Log |
|-------|---------|
| Failed login attempts | Yes |
| Permission denied errors | Yes |
| Input validation failures | Yes |
| Password changes | Yes |
| Admin actions | Yes |
| Successful logins | Recommended |

## A10: Server-Side Request Forgery (SSRF)

Application fetches remote resources without validating the URL.

```python
# VULNERABLE — user controls the URL
@app.get("/fetch")
def fetch_url(url: str):
    return requests.get(url).text  # can hit internal services

# SECURE — allowlist of domains
ALLOWED_HOSTS = {"api.example.com", "cdn.example.com"}

@app.get("/fetch")
def fetch_url(url: str):
    parsed = urlparse(url)
    if parsed.hostname not in ALLOWED_HOSTS:
        raise HTTPException(400, "Domain not allowed")
    return requests.get(url).text
```

## Audit Report Format

```
## Finding: [Title]
**Severity:** Critical | High | Medium | Low
**Category:** A01-A10
**Location:** file.py:42
**Description:** What the vulnerability is
**Attack Vector:** How an attacker would exploit it
**Impact:** What happens if exploited
**Remediation:** Specific fix with code example
```
