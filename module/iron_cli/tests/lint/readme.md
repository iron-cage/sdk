# lint

## Responsibility

Enforce code quality standards via compile-time lint tests.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `no_direct_printing_test.rs` | Enforce TreeFmtFormatter usage via static analysis |

## Notes

Lint tests fail at compile time if violations detected. Ensures code standards maintained automatically.
