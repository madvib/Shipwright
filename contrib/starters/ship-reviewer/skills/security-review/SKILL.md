---
name: Security Review
description: Security-focused code review covering authentication, injection, and data exposure
tags: [security, review, authentication, injection, owasp]
---

# Security Review

## Authentication and Authorization

### Authentication Checks

Every endpoint that accesses user data must verify identity.

```
Public endpoint? --> No auth needed (login, registration, health check)
Authenticated endpoint? --> Verify session/token is valid
Authorized endpoint? --> Verify user has permission for THIS resource
```

| Check | What to Look For |
|-------|-----------------|
| Missing auth middleware | Endpoint handler with no auth check |
| Broken object-level auth | User A can access User B's data by changing an ID |
| Privilege escalation | Regular user can access admin endpoints |
| Token validation | Expired tokens rejected, signature verified |
| Session fixation | Session ID regenerated after login |

### Authorization Patterns

Always check ownership, not just authentication.

```python
# WRONG — only checks if user is logged in
@login_required
def delete_document(request, doc_id):
    Document.objects.get(id=doc_id).delete()

# RIGHT — checks if user owns the document
@login_required
def delete_document(request, doc_id):
    doc = Document.objects.get(id=doc_id)
    if doc.owner_id != request.user.id:
        raise PermissionDenied()
    doc.delete()
```

## Injection Vulnerabilities

### SQL Injection

Never interpolate user input into SQL queries.

```python
# VULNERABLE
cursor.execute(f"SELECT * FROM users WHERE name = '{name}'")

# SAFE — parameterized query
cursor.execute("SELECT * FROM users WHERE name = %s", [name])
```

### Cross-Site Scripting (XSS)

User input rendered in HTML must be escaped.

| Context | Escape Method |
|---------|--------------|
| HTML body | HTML entity encoding (`&lt;`, `&gt;`) |
| HTML attribute | Attribute encoding + quote the value |
| JavaScript | JSON encoding, never string interpolation |
| URL parameter | URL encoding (`encodeURIComponent`) |
| CSS | CSS encoding or allowlist values |

Framework-specific:
- React: JSX auto-escapes by default. Watch for `dangerouslySetInnerHTML`.
- Rails: ERB auto-escapes by default. Watch for `raw` and `html_safe`.
- Go templates: `html/template` auto-escapes. Watch for `template.HTML()` type.

### Command Injection

Never pass user input to shell commands.

```python
# VULNERABLE
os.system(f"convert {filename} output.png")

# SAFE — use subprocess with argument list
subprocess.run(["convert", filename, "output.png"], check=True)
```

## Data Exposure

### Sensitive Data in Responses

| Data Type | Rule |
|-----------|------|
| Passwords | Never return, even hashed |
| API keys / tokens | Never log, never return in full |
| Email addresses | Only to the owner or admin |
| Internal IDs | Prefer UUIDs over sequential integers |
| Stack traces | Never in production responses |
| Database errors | Wrap in generic error, log details server-side |

### Logging

```
NEVER log: passwords, tokens, credit cards, SSNs, API keys
OK to log: user IDs, action names, timestamps, error codes
REDACT before logging: email addresses, IP addresses (depending on jurisdiction)
```

## Cryptography

| Need | Use | Do Not Use |
|------|-----|-----------|
| Password hashing | bcrypt, scrypt, argon2 | MD5, SHA-1, SHA-256 alone |
| Token generation | CSPRNG (`crypto.randomBytes`, `secrets.token_hex`) | `Math.random`, `rand()` |
| Encryption at rest | AES-256-GCM | DES, 3DES, ECB mode |
| HTTPS certificates | Automated (Let's Encrypt) | Self-signed in production |

## Dependency Review

When a PR adds or updates dependencies:

- [ ] Is the package actively maintained (commits in last 6 months)?
- [ ] Does it have known vulnerabilities (`npm audit`, `cargo audit`, `pip audit`)?
- [ ] Is the license compatible with the project?
- [ ] Does it pull in excessive transitive dependencies?
- [ ] Is there a lighter alternative that does the same thing?

## Security Review Checklist

- [ ] All endpoints have appropriate authentication
- [ ] Object-level authorization checked (not just role-based)
- [ ] No SQL injection (all queries parameterized)
- [ ] No XSS (all user input escaped in output context)
- [ ] No command injection (no shell interpolation of user input)
- [ ] Sensitive data not exposed in API responses or logs
- [ ] Passwords hashed with bcrypt/scrypt/argon2
- [ ] Tokens generated with CSPRNG
- [ ] New dependencies audited for vulnerabilities
- [ ] Error messages do not leak internal details
- [ ] Rate limiting on authentication endpoints
- [ ] CORS configured to allow only trusted origins
