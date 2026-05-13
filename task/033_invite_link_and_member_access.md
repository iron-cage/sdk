# Task 033: Magic invite link + Member-side Model Access

## Goal

Close the 60-second onboarding loop by generating a magic invite link with policy chips that admins can copy and share, and by giving each invited member a self-service Model Access screen that shows their gateway URL, IC token, active default model, and Request buttons for gated `requestable:` models. The result is observable through the admin Users screen showing a "Magic Invite Link Generated" dialog after the FreeForm Providers+Policy apply, and through the member dashboard showing "Your Iron Cage access" with the IC token visible and Request buttons rendering for each gated model. Testable by walking the full deck flow end-to-end: admin pastes block → invite link generated → member clicks link → member completes FreeForm Registration → member lands on dashboard with Model Access populated → member requests a gated model.

## Dependencies
- Task 029 (FreeForm component infrastructure) — required for member Registration FreeForm
- Task 030 (Registration FreeForm twin) — member Registration FreeForm twin reuses 030's implementation directly; member side has no extra schema work
- Task 032 (Providers + Usage Policy FreeForm) — invite generation is triggered after Apply on that flow; depends on the `workspace_policy` model defined there for the policy-snapshot binding

## Codebase reality check (2026-05-13)
- **No member dashboard route or page exists.** This task scaffolds `MemberDashboardPage.vue`.
- **No invite token model exists.** Greenfield (already accounted for in scope).
- **No access-request model exists.** Greenfield (already accounted for in scope).
- **Magic-link login**: the deck opens with a magic-link login screen at 0:00 (admin) and 0:00 (member). Verify before starting whether magic-link auth has been implemented elsewhere in the codebase (`module/iron_control_api/src/auth/` or `module/iron_token_manager/src/auth/`). If absent, magic-link auth is a **hard prerequisite** that must be carved into a separate task before 033 can ship — invite links are useless without an authentication mechanism for the joiner to actually log in afterward.
- **Only the `Admin` role exists** — no SuperUser. Invite generation and access-request resolution are Admin-only.
- IC token model already stores tokens as hash with raw surfaced once at generation — reuse this pattern for invite tokens.

## In Scope

- **Scaffold `MemberDashboardPage.vue`** at `iron_site/src/views/` (the member-facing dashboard does not yet exist) with the layout from the deck: top stats row + "Your Iron Cage access" panel; register the route in the router
- Magic invite link generation on the admin Users screen: `https://ironcage.app/join/ic_team_<token>` with 7-day expiry, configurable seat count, and policy chips (default model, spending limit, seat count) displayed on the dialog
- "Magic Invite Link Generated" modal: copy-link button, expiry text, seats remaining, policy chips matching the deck (`gpt-5.4-mini default`, `$100/week limit`, `10 seats`)
- Server-side invite token model: bound to workspace policy snapshot at generation time, single-use per seat, decremented on successful registration
- Member join flow: invite link routes to a member-facing Registration page (same FreeForm twin from 030); on successful Registration + Confirm, member is enrolled into the workspace with the snapshot policy
- Member dashboard "Your Iron Cage access" panel: gateway URL, IC token (with copy-to-clipboard, hash-only on server per existing token model), active default model with a green active indicator, and a "Request Access" list of gated models
- "Request" button per gated model: creates a pending access request visible to admins; "Requested" badge replaces the button until approved
- Admin-side: a Pending Requests area on the Users or Providers screen lets admins approve or deny requests in one click
- Telemetry: `invite.link.generated`, `invite.consumed`, `member.access_request.created`, `member.access_request.resolved`

## Out of Scope

- Multi-seat link sharing beyond simple seat-count decrement (no per-user invite link customization)
- Email delivery of the invite — only link generation + copy is in scope; email send can come later
- SMS or other delivery channels
- Bulk approval UI for access requests (single approval is sufficient for v1)
- Member-side editing of their own profile beyond what Registration already covers
- Time-bounded access grants — approved means approved until revoked

## Description

The deck closes the admin loop at 0:34 with "Magic Invite Link Generated" and opens the member loop at 0:00 (member side). The admin doesn't enter ten email addresses one by one — the Providers+Policy paste from task 032 queues the invitees, and this task converts that queue into a single magic link the admin copies and pastes into whatever channel they prefer (Slack, email, etc.).

The link encodes a workspace-scoped invite token bound to a policy snapshot — at click time the joining member inherits exactly the cap, default model, and requestable allowlist that were in effect when the link was generated. This sidesteps the "what if the admin changes the policy after the link is sent?" race. Seat decrement is atomic.

The member's first surface after Registration is the dashboard showing "Your Iron Cage access" — gateway URL, IC token, default model marked active, and a request-access list for any model gated under `requestable:`. The deck shows this at 0:13 (member side) with `claude-4-6-sonnet` already "Requested" and `gemini-3.1-pro-preview` in the Request state. Clicking Request flips the button to a Requested badge; an admin then approves or denies from their side.

End-to-end, the deck's promise — "Zero to 10-person protected team. Under 60 seconds. No forms filled." — holds when this task lands on top of 029, 030, and 032.

## Context

Critical areas:
- `module/iron_token_manager` — new invite-token model with policy snapshot, seat count, expiry; existing IC-token issuance is reused
- `module/iron_control_api/src/routes/invites.rs` (new) — generate / list / revoke invite links; consume on member registration
- `module/iron_control_api/src/routes/access_requests.rs` (new) — create / list / approve / deny model access requests
- `src/views/UsersPage.vue` — host for invite link generation dialog
- `src/views/MemberDashboardPage.vue` — host for "Your Iron Cage access" panel
- Member Registration route — wraps the FreeForm twin from 030; the only difference is the invite-token capture and the workspace enrollment step on Confirm

## Work Procedure

0. Verify magic-link auth is implemented elsewhere; if not, block on a separate magic-link auth task before proceeding.
1. Scaffold `MemberDashboardPage.vue` and register its route.
2. Model the invite token: workspace_id, policy_snapshot, seats_total, seats_used, expires_at, raw token hashed on storage.
3. Implement `POST /invites/generate` (Admin) that takes seat count and produces the magic link.
4. Implement `GET /invites/:token` to resolve a link and return the policy snapshot for the member-facing Registration screen.
5. Implement `POST /invites/:token/consume` invoked on member Registration Confirm; atomically decrements seats and creates the IC token.
6. Build the "Magic Invite Link Generated" modal on the admin Users screen with copy-link, expiry text, seats text, policy chips.
7. Build the member-facing Registration route (reuses the 030 FreeForm twin); on Confirm, the consume endpoint runs and the member lands on the dashboard.
8. Build the "Your Iron Cage access" panel on the member dashboard with gateway URL, IC token (copy-to-clipboard), active default model, Request buttons for gated models.
9. Implement `POST /access-requests` (member-initiated) and `POST /access-requests/:id/resolve` (Admin).
10. Surface a Pending Requests area on the admin Users screen with Approve/Deny.
11. End-to-end test the full deck flow: admin paste → Apply → invite link → member clicks → member Registration (FreeForm) → Confirm → member dashboard shows access panel → member Requests a gated model → admin approves → member sees model unlocked.

## Implementation plan

1. Invite token data model + generate/resolve/consume endpoints.
2. Invite Link Generated modal on the admin Users screen.
3. Member Registration route reusing the 030 FreeForm twin + invite-token consume on Confirm.
4. "Your Iron Cage access" panel on the member dashboard.
5. Access-request endpoints + admin Pending Requests UI.
6. End-to-end deck-flow integration test.

## Test Matrix

| Scenario | Expected behavior | Pass criteria |
|---|---|---|
| Admin generates link after Example 1 apply | Modal shows URL, expiry, seats, policy chips | All four elements match deck |
| Member clicks valid link | Member-facing Registration loaded | FreeForm twin available; policy snapshot visible |
| Member completes Registration and Confirms | Seat decremented, IC token issued | Atomic; member lands on dashboard |
| Member dashboard renders access panel | Gateway, IC token, default model active, Request buttons for gated models | Matches deck visual |
| Member clicks Request on a gated model | Button becomes "Requested" badge | DB row created; admin sees pending |
| Admin approves request | Member's Request button disappears next render | Model now usable by member |
| Link expired | Member sees clear error | No registration possible |
| All seats used | Link rejects further joins | Clear "no seats remaining" error |
| Policy changed after link generated | Joiner still gets snapshot policy | Snapshot binding works |
| Member tries to use a non-requestable model | Gateway rejects | Standard model-not-allowed error |
| End-to-end deck flow under 60 seconds | Full path completes | Timestamp at member access ≤ 60s from admin start |

## Validation Checklist

- [ ] Invite token model with workspace_id, policy_snapshot, seats, expiry; raw token hashed
- [ ] `POST /invites/generate` (Admin only)
- [ ] `GET /invites/:token` returns policy snapshot for member view
- [ ] `POST /invites/:token/consume` atomic seat decrement + IC token issuance
- [ ] Magic Invite Link Generated modal with all deck elements (URL, expiry, seats, policy chips)
- [ ] Member-facing Registration route reuses the 030 FreeForm twin
- [ ] Member dashboard "Your Iron Cage access" panel renders gateway, IC token, default model, Request list
- [ ] IC token copy-to-clipboard works; storage remains hash-only on server
- [ ] `POST /access-requests` (member-initiated)
- [ ] `POST /access-requests/:id/resolve` (Admin)
- [ ] Admin Pending Requests area on Users screen with Approve/Deny
- [ ] Expired link rejected with clear error
- [ ] All-seats-used link rejected with clear error
- [ ] Policy snapshot persists even if workspace policy changes later
- [ ] Telemetry events fire at all four lifecycle points
- [ ] End-to-end deck flow completes under 60 seconds on a real browser run

## Validation Procedure

1. Run task 032's Example 1 paste in the Providers FreeForm; Apply.
2. From the Users screen, click "+ Invite Team"; verify the Magic Invite Link Generated modal renders with the four deck elements.
3. Copy the link, open it in an incognito window; verify the member-facing Registration loads with policy snapshot visible.
4. Use the FreeForm twin to paste a member intro (e.g., "Alice Johnson, alice@acmecorp.com — born Feb 3 1994"); Confirm.
5. Verify the member lands on a dashboard showing "Your Iron Cage access" with gateway URL, IC token, default model (active), and Request buttons for the two gated models.
6. Click Request on `claude-4-6-sonnet`; verify the button becomes a Requested badge.
7. Switch to the admin side; verify Pending Requests lists Alice's claude request; click Approve.
8. Switch back to Alice; refresh; verify the Request UI for claude is replaced with active status.
9. Wait for invite-link expiry (or fast-forward); verify a fresh click on the expired link rejects with a clear error.
10. Time the full path from admin paste to member's first model use; verify it completes in under 60 seconds.

## Acceptance Criteria

- Admin can generate a magic invite link with seat count, expiry, and policy chips matching the deck.
- The link is bound to a policy snapshot taken at generation time; later workspace-policy changes do not affect prior joiners.
- Member clicks the link, completes Registration via the FreeForm twin, and lands on the dashboard with the "Your Iron Cage access" panel.
- The panel shows gateway URL, IC token (copy-to-clipboard, hash-only on server), the active default model, and a Request list for gated models.
- Members can request access to gated models; admins can approve or deny.
- The end-to-end deck flow (admin paste → invite link → member registration → member model use) completes in under 60 seconds.
