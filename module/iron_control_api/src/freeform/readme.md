# freeform/ - FreeForm Onboarding Parsers (Task 029)

## Responsibility Table

| File               | Responsibility                                                      |
| ------------------ | ------------------------------------------------------------------- |
| `company_setup.rs` | Parse a company-setup line into name, domain, and account type      |
| `invites.rs`       | Parse a paste block of invite email addresses (one per line)        |
| `mod.rs`           | Declare and re-export the freeform parser submodules                |
| `providers.rs`     | Parse `provider: key` paste blocks into known-provider entries      |
| `usage_policy.rs`  | Parse usage-policy paste blocks into spending cap and default model |

## Directory Purpose

Deterministic, structured-grammar parsers for the FreeForm onboarding flow
(Task 029). Each module turns a raw text paste into a validated, typed result
or a list of per-line parse errors. Parsing is pure and side-effect free: the
route handlers in `routes/freeform.rs` and `routes/workspace.rs` call these
parsers and then persist the results. The grammar is explicit (not
LLM-inferred), so behavior is deterministic and idempotent.
