# Task 027: Dashboard integration

## Dependencies
- PR #55

## Context
Dashboard UI/UX overhaul (PR #55) is APPROVED. Mobile adaptation (PR #61) is in progress. Dashboard needs to be connected to server-proxy API for end-to-end functionality.

Critical areas:
- Dashboard frontend (PR #55, PR #61)
- `module/iron_control_api/src/routes/`
- Server-proxy API endpoints

## Implementation plan
1. Land PR #55 (dashboard overhaul).
2. Land PR #61 (mobile adaptation).
3. Connect dashboard to server-proxy API endpoints.
4. Verify dashboard displays real-time analytics, budget status, agent management.

## Acceptance criteria
- Dashboard shows live agent status and budget usage.
- Mobile layout works on common screen sizes.
- All CRUD operations (agents, providers, tokens) work through dashboard.
