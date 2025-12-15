# bin

## Responsibility

Binary entry points for Iron CLI tools.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `iron_control_unilang.rs` | Control API CLI with 47 commands (agents, providers, analytics, budgets, users) |
| `iron_token_unilang.rs` | Token Management CLI with 22 commands (auth, tokens, usage, limits, traces) |

## Notes

Both binaries use the Unilang framework with YAML-defined commands. They follow a handler-adapter architecture where handlers are pure validation functions and adapters handle I/O operations (HTTP requests, file operations).
