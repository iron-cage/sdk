# formatting

## Responsibility

Test CLI output formatting across all format types.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `formatter_test.rs` | Test legacy formatter implementation |
| `tree_formatter_test.rs` | Test tree_fmt-based formatter implementation |

## Notes

Pure function testing with 23 test cases covering data→formatted string transformations. No mocking needed as formatter has no I/O. See `-test_matrix.md` for coverage plan.
