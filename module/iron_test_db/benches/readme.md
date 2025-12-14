# Benchmarks

Performance benchmarks for iron_test_db infrastructure.

## Organization

| File | Responsibility |
|------|----------------|
| test_db_creation.rs | Database creation and initialization performance |

## Benchmark Categories

- **Creation Performance:** Database instance creation overhead
- **Migration Performance:** Migration execution timing
- **Cleanup Performance:** Database teardown and cleanup timing

## Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench --bench test_db_creation

# With baseline comparison
cargo bench --bench test_db_creation -- --save-baseline before
# ... make changes ...
cargo bench --bench test_db_creation -- --baseline before
```

## Benchmark Configuration

- Uses Criterion.rs for statistical analysis
- Async runtime: Tokio
- Storage modes: In-memory vs file-based comparison
