# Tests

Tests for iron_secrets encryption and key management.

## Organization

| File | Responsibility |
|------|----------------|
| crypto_test.rs | Cryptographic operations and key derivation tests |
| smoke_test.rs | Basic functionality and integration smoke tests |

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
