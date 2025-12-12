# middleware

## Responsibility

HTTP request processing middleware for Iron Control API server.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Declares and re-exports middleware modules |
| `url_redirect.rs` | Implements 308 Permanent Redirect middleware for deprecated URL paths to spec-compliant paths |
