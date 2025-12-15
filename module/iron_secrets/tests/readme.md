# Tests

Tests for iron_secrets encryption and key management.

## Responsibility Table

| File | Responsibility | Input→Output | Out of Scope |
|------|----------------|--------------|--------------|
| `crypto_test.rs` | Test cryptographic operations and key derivation | Crypto scenarios → Security validation | NOT integration (smoke_test.rs) |
| `smoke_test.rs` | Test basic functionality and integration smoke tests | End-to-end flows → Integration validation | NOT crypto details (crypto_test.rs) |

## Test Categories

- **Unit Tests:** Encryption/decryption operations
- **Integration Tests:** End-to-end secret lifecycle
- **Security Tests:** Key derivation and access control

## Running Tests

```bash
# All tests
cargo nextest run

# Crypto tests only
cargo nextest run --test crypto_test

# Smoke tests
cargo nextest run --test smoke_test
```

## Test Data

- Test encryption keys and passphrases
- Mock secret values for validation
