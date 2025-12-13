# Tests

Tests for iron_telemetry observability and tracing.

## Organization

| File | Responsibility |
|------|----------------|
| telemetry_test.rs | Tracing, metrics, and logging infrastructure tests |

## Test Categories

- **Unit Tests:** Individual telemetry component validation
- **Integration Tests:** End-to-end telemetry pipeline
- **Configuration Tests:** Telemetry setup and initialization

## Running Tests

```bash
# All tests
cargo nextest run

# Telemetry tests
cargo nextest run --test telemetry_test
```

## Test Data

- Mock trace spans and events
- Test metric collectors
