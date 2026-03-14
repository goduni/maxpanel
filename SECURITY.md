# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability, **do not open a public issue**.

Instead, report it privately:

1. **Telegram:** [@goduni](https://t.me/goduni) (preferred)
2. **GitHub:** Use [Security Advisories](https://github.com/goduni/maxpanel/security/advisories/new)

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We aim to respond within 48 hours and release a fix within 7 days for critical issues.

## Security Measures

MaxPanel implements the following security measures:

- **Encryption:** Bot tokens encrypted with AES-256-GCM, per-bot HKDF-derived keys
- **Authentication:** JWT with HMAC-SHA256, Argon2 password hashing, refresh token rotation with family-based reuse detection
- **Rate Limiting:** Token-bucket rate limiter on auth, webhook, and API proxy endpoints
- **Headers:** CSP, HSTS, X-Frame-Options, X-Content-Type-Options
- **Input Validation:** `deny_unknown_fields` on all request DTOs, path validation on API proxy
- **SQL Safety:** Compile-time checked queries via SQLx
- **Separation:** BotRow (DB) vs BotResponse (API) — sensitive fields never serialized to responses
