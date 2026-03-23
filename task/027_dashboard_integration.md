# Task 027: Dashboard integration

## Goal
Land the approved dashboard UI overhaul (PR #55) and mobile adaptation (PR #61), then connect the dashboard frontend to the server-proxy API so that operators can view real-time analytics, manage agents, and monitor budget status through a working end-to-end web interface.

## Dependencies
- PR #55

## In Scope
- Landing PR #55 (dashboard UI/UX overhaul)
- Landing PR #61 (mobile adaptation)
- Connecting dashboard frontend to server-proxy API endpoints
- Real-time analytics display (agent status, request counts)
- Budget status visualization
- Agent, provider, and token CRUD operations through the dashboard

## Out of Scope
- New dashboard features beyond what PR #55 and PR #61 provide
- Authentication/authorization for the dashboard (separate concern)
- Dashboard deployment infrastructure (CDN, hosting)
- Offline mode or service worker support

## Description
The dashboard UI/UX overhaul in PR #55 has been approved and the mobile adaptation in PR #61 is in progress. Once both are landed, the dashboard frontend needs to be connected to the server-proxy API to provide end-to-end functionality. This means wiring up API calls for all CRUD operations (agents, providers, tokens) and ensuring real-time data flows from the backend to the dashboard.

The integration work covers three areas: data display (live agent status, request analytics, budget usage), management operations (creating, updating, and deleting agents, providers, and tokens through the dashboard), and mobile responsiveness (verifying the layout works on common screen sizes). The server-proxy API endpoints in `iron_control_api` already exist - this task is about connecting them to the frontend, not building new backend endpoints.

## Context
Dashboard UI/UX overhaul (PR #55) is APPROVED. Mobile adaptation (PR #61) is in progress. Dashboard needs to be connected to server-proxy API for end-to-end functionality.

Critical areas:
- Dashboard frontend (PR #55, PR #61)
- `module/iron_control_api/src/routes/`
- Server-proxy API endpoints

## Work Procedure
1. Review PR #55 for any outstanding comments or required changes, then merge.
2. Review PR #61 for mobile adaptation completeness, then merge.
3. Identify all server-proxy API endpoints that the dashboard needs to consume.
4. Wire up agent management pages to the agent CRUD API endpoints.
5. Wire up provider management pages to the provider CRUD API endpoints.
6. Wire up token management pages to the token CRUD API endpoints.
7. Connect the analytics display to the real-time analytics API.
8. Connect the budget status display to the budget API endpoints.
9. Test all CRUD operations end-to-end through the dashboard.
10. Test the mobile layout on common screen sizes (320px, 375px, 768px, 1024px).

## Implementation plan
1. Land PR #55 (dashboard overhaul).
2. Land PR #61 (mobile adaptation).
3. Connect dashboard to server-proxy API endpoints.
4. Verify dashboard displays real-time analytics, budget status, agent management.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Create agent via dashboard | Agent appears in list | API returns 201 and dashboard updates |
| Update agent via dashboard | Agent details updated | API returns 200 and dashboard reflects changes |
| Delete agent via dashboard | Agent removed from list | API returns 200 and agent no longer visible |
| View analytics page | Real-time data displayed | Charts render with current data |
| View budget status | Current budget usage shown | Budget bar matches API response |
| Dashboard on 375px screen | Mobile layout applied | All content accessible, no horizontal scroll |
| Dashboard on 1024px screen | Desktop layout applied | Full layout with sidebar navigation |

## Validation List
- [ ] PR #55 merged successfully
- [ ] PR #61 merged successfully
- [ ] Agent CRUD operations work through the dashboard
- [ ] Provider CRUD operations work through the dashboard
- [ ] Token CRUD operations work through the dashboard
- [ ] Analytics page displays real-time data
- [ ] Budget status displays current usage
- [ ] Mobile layout renders correctly on 320px, 375px, 768px widths
- [ ] No console errors in browser developer tools during normal operation

## Validation Procedure
1. Start the server-proxy and dashboard locally.
2. Create, update, and delete an agent through the dashboard and verify changes persist via the API.
3. Create, update, and delete a provider through the dashboard and verify changes persist via the API.
4. Create, update, and delete a token through the dashboard and verify changes persist via the API.
5. Navigate to the analytics page and verify charts render with current data.
6. Navigate to the budget page and verify the budget display matches the API response.
7. Resize the browser to 375px width and verify the mobile layout is fully functional.
8. Open browser developer tools and verify no JavaScript errors during all above operations.

## Acceptance criteria
- Dashboard shows live agent status and budget usage.
- Mobile layout works on common screen sizes.
- All CRUD operations (agents, providers, tokens) work through dashboard.
