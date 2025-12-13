# Benchmarks

Performance benchmarks for iron_types ID types.

## Organization

| File | Responsibility |
|------|----------------|
| id_benchmarks.rs | ID creation, parsing, and serialization performance |

## Benchmark Categories

- **Creation Performance:** ID generation and construction overhead
- **Parsing Performance:** String to ID parsing timing
- **Serialization Performance:** JSON serialization/deserialization timing
- **Comparison Performance:** ID equality and hashing operations

## Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench --bench id_benchmarks

# With baseline comparison
cargo bench --bench id_benchmarks -- --save-baseline before
# ... make changes ...
cargo bench --bench id_benchmarks -- --baseline before
```

## Benchmark Configuration

- Uses Criterion.rs for statistical analysis
- Tests all ID types: AgentId, ProviderId, UserId, etc.
- Compares with/without test-helpers feature
