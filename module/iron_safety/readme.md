# iron_cage_safety

PII detection and output validation for multi-agent systems.

### Scope

**Responsibilities:**
Detects and redacts PII (emails, phones, SSNs, credit cards) from AI agent outputs with configurable safety policies and optional SQLite-based audit logging. Provides real-time validation preventing data breaches before agent responses reach users. Requires Rust 1.75+, all platforms supported, optional SQLite audit trail with `audit` feature.

**In Scope:**
- PII pattern detection (emails, phones, SSNs, credit cards)
- Output redaction and sanitization
- Safety audit trail (with `audit` feature)
- Configurable safety policies
- Real-time validation of agent outputs

**Out of Scope:**
- Cost tracking (see iron_cage_cost)
- Circuit breaker logic (see iron_cage_reliability)
- Agent lifecycle management (see iron_cage_cli)
- Configuration management (see iron_cage_types)

## Installation

```toml
[dependencies]
iron_cage_safety = { version = "0.1", features = ["full"] }
```

## Features

- `enabled` (default): Core PII detection without audit logging
- `full`: All functionality including audit trail
- `audit`: SQLite-based audit logging

## Example

```rust
use iron_cage_safety::PiiDetector;

// Initialize PII detector with default patterns
let detector = PiiDetector::new()?;

// Check for PII in agent output
let text = "Contact me at john@example.com";
if detector.check(text) {
    // Redact PII before displaying
    let safe = detector.redact(text);
    println!("Safe output: {}", safe);
    // Output: "Safe output: Contact me at [EMAIL_REDACTED]"
}
```

## Documentation

- [API Reference](https://docs.rs/iron_cage_safety)
- [Safety Patterns Guide](docs/patterns.md)

## License

MIT

## Directory Structure

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | PII detection and output sanitization |

**Notes:**
- Entries marked 'TBD' require manual documentation
- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities

