# Directory: provider_key_storage

## Responsibility Table

| File | Responsibility |
|------|----------------|
| key_crud.rs | CRUD operations, quota-guarded creation, and field updates for provider keys |
| key_projects.rs | Key-to-project assignment, unassignment, and batch lookups |
| key_spending.rs | Spending caps, reserve/adjust flow, and usage limit enforcement |
