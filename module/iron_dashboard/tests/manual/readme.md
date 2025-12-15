# manual/

Manual testing procedures for iron_dashboard pilot.

## Overview

**Purpose:** Comprehensive manual test plan for conference demo verification.

**Scope:** All frontend functionality (authentication, dashboard, token management, usage analytics, limits, traces, accessibility).

**Prerequisites:**
- Backend (iron_control_api) running on http://localhost:3001
- Frontend dev server running on http://localhost:5173
- Modern browser (Chrome 120+, Firefox 120+, Safari 17+)
- Test credentials: username `test`, password `test`

**Execution Time:** ~30 minutes (complete test suite)

---

## Manual Testing Plan

### Test 1: Authentication Flow

**Purpose:** Verify login, logout, and session persistence functionality.

**Procedure:**

1. **Initial State (Unauthenticated)**
   - Navigate to `http://localhost:5173` in browser
   - ✅ Expect: Automatic redirect to `/login`
   - ✅ Expect: Login form visible (username input, password input, submit button)

2. **Invalid Login**
   - Enter username: `wrong`
   - Enter password: `wrong`
   - Click "Login" button
   - ✅ Expect: Error message displayed ("Invalid credentials" or similar)
   - ✅ Expect: Remain on `/login` page

3. **Valid Login**
   - Enter username: `test`
   - Enter password: `test`
   - Click "Login" button
   - ✅ Expect: Redirect to `/dashboard`
   - ✅ Expect: No error messages
   - ✅ Expect: Header shows "Logout" button

4. **Session Persistence**
   - Refresh page (F5 or Ctrl+R)
   - ✅ Expect: Remain on `/dashboard` (no redirect to login)
   - ✅ Expect: Dashboard data loads (budget, agents, requests)

5. **Logout**
   - Click "Logout" button in header
   - ✅ Expect: Redirect to `/login`
   - ✅ Expect: localStorage token removed (check DevTools → Application → Local Storage)

6. **Protected Route Access (Unauthenticated)**
   - Manually navigate to `http://localhost:5173/dashboard`
   - ✅ Expect: Automatic redirect to `/login`

**Acceptance Criteria:**
- [ ] Invalid login shows error message
- [ ] Valid login redirects to dashboard
- [ ] Session persists across page refresh
- [ ] Logout clears session and redirects to login
- [ ] Unauthenticated users cannot access protected routes

---

### Test 2: Token Management

**Purpose:** Verify token creation, rotation, and revocation functionality.

**Procedure:**

1. **Navigate to Tokens View**
   - Login (username: `test`, password: `test`)
   - Click "Tokens" in navigation menu
   - ✅ Expect: Navigate to `/tokens`
   - ✅ Expect: Token list table visible

2. **Create Token**
   - Click "Create Token" button
   - ✅ Expect: Modal dialog appears
   - Enter token name: `test-token-1`
   - Enter project_id: `project-001` (optional)
   - Click "Create" button
   - ✅ Expect: Token string displayed (starts with `iron_`)
   - ✅ Expect: "Copy" button visible
   - Click "Copy" button
   - ✅ Expect: Token copied to clipboard (paste in notepad to verify)
   - Close modal
   - ✅ Expect: New token appears in token list table
   - ✅ Expect: Token string masked (e.g., `iron_***************abcd`)

3. **Rotate Token**
   - Locate `test-token-1` in token list
   - Click "Rotate" button for that token
   - ✅ Expect: Confirmation dialog appears ("Are you sure?")
   - Click "Confirm"
   - ✅ Expect: New token string displayed (DIFFERENT from original)
   - ✅ Expect: "Copy" button visible
   - Copy new token (for verification)
   - ✅ Expect: Original token no longer valid (verify with backend if possible)

4. **Revoke Token**
   - Locate `test-token-1` in token list
   - Click "Revoke" button for that token
   - ✅ Expect: Confirmation dialog appears ("Are you sure?")
   - Click "Confirm"
   - ✅ Expect: Token status changes to "Revoked" or "Inactive"
   - ✅ Expect: Token no longer usable (verify with backend if possible)

5. **Token List Display**
   - Verify table columns: ID, Name, Created, Last Used, Status
   - ✅ Expect: All tokens visible in table
   - ✅ Expect: Timestamps formatted correctly (human-readable date/time)
   - ✅ Expect: Active tokens show "Active" status
   - ✅ Expect: Revoked tokens show "Revoked" status

**Acceptance Criteria:**
- [ ] Create token flow completes successfully
- [ ] Token string shown only once after creation
- [ ] Copy button successfully copies token to clipboard
- [ ] Rotate token generates new token and invalidates old
- [ ] Revoke token marks as inactive
- [ ] Token list displays all tokens with correct status

---

### Test 3: Usage Analytics

**Purpose:** Verify usage statistics display and cost breakdown functionality.

**Procedure:**

1. **Navigate to Usage View**
   - Login (username: `test`, password: `test`)
   - Click "Usage" in navigation menu
   - ✅ Expect: Navigate to `/usage`
   - ✅ Expect: Usage overview cards visible

2. **Usage Overview Cards**
   - ✅ Expect: "Total Requests" card shows count
   - ✅ Expect: "Total Input Tokens" card shows count
   - ✅ Expect: "Total Output Tokens" card shows count
   - ✅ Expect: "Total Cost" card shows USD amount (e.g., `$12.34`)

3. **By Provider Breakdown**
   - ✅ Expect: Table with columns: Provider, Requests, Cost
   - ✅ Expect: Rows for each provider (OpenAI, Anthropic, etc.)
   - ✅ Expect: Request counts displayed as integers
   - ✅ Expect: Costs displayed as USD (e.g., `$5.67`)

4. **By Model Breakdown**
   - ✅ Expect: Table with columns: Model, Requests, Cost
   - ✅ Expect: Rows for each model (gpt-4o, claude-3.5-sonnet, etc.)
   - ✅ Expect: Request counts displayed as integers
   - ✅ Expect: Costs displayed as USD (e.g., `$3.21`)

5. **Data Accuracy**
   - Compare totals in overview cards with sum of breakdown tables
   - ✅ Expect: Total cost in overview equals sum of provider costs
   - ✅ Expect: Total requests in overview equals sum of provider requests

**Acceptance Criteria:**
- [ ] Usage overview cards display correct totals
- [ ] Provider breakdown shows all providers with costs
- [ ] Model breakdown shows all models with costs
- [ ] Totals match sum of breakdown tables

---

### Test 4: Budget Limits

**Purpose:** Verify budget limit creation, update, and deletion functionality.

**Procedure:**

1. **Navigate to Limits View**
   - Login (username: `test`, password: `test`)
   - Click "Limits" in navigation menu
   - ✅ Expect: Navigate to `/limits`
   - ✅ Expect: Limits list table visible

2. **Create Daily Cost Limit**
   - Click "Create Limit" button
   - ✅ Expect: Modal dialog appears
   - Select limit type: `daily_cost`
   - Enter limit value: `100`
   - Select period: `daily`
   - Click "Create" button
   - ✅ Expect: New limit appears in limits list table
   - ✅ Expect: Limit displays: Type = "daily_cost", Value = 100, Period = "daily"

3. **Create Monthly Request Limit**
   - Click "Create Limit" button
   - Select limit type: `total_requests`
   - Enter limit value: `10000`
   - Select period: `monthly`
   - Click "Create" button
   - ✅ Expect: New limit appears in limits list table

4. **Update Limit**
   - Locate first limit in list
   - Click "Edit" button for that limit
   - ✅ Expect: Modal dialog appears with current values
   - Change limit value to `150`
   - Click "Update" button
   - ✅ Expect: Limit value updates in table to 150

5. **Delete Limit**
   - Locate first limit in list
   - Click "Delete" button for that limit
   - ✅ Expect: Confirmation dialog appears ("Are you sure?")
   - Click "Confirm"
   - ✅ Expect: Limit removed from table

**Acceptance Criteria:**
- [ ] Create limit flow completes successfully for all limit types
- [ ] Update limit modifies value correctly
- [ ] Delete limit removes from list after confirmation
- [ ] Limits list displays all limits with correct values

---

### Test 5: Request Traces

**Purpose:** Verify request trace list and detail view functionality.

**Procedure:**

1. **Navigate to Traces View**
   - Login (username: `test`, password: `test`)
   - Click "Traces" in navigation menu
   - ✅ Expect: Navigate to `/traces`
   - ✅ Expect: Traces list table visible

2. **Traces List Display**
   - ✅ Expect: Table columns: ID, Request ID, Provider, Model, Input Tokens, Output Tokens, Cost, Time
   - ✅ Expect: Rows for each trace (if backend has data)
   - ✅ Expect: Timestamps formatted correctly (human-readable)
   - ✅ Expect: Costs formatted as USD (e.g., `$0.12`)

3. **Trace Sorting**
   - ✅ Expect: Traces sorted by timestamp (descending, newest first)
   - Click column header (e.g., "Cost")
   - ✅ Expect: Table re-sorts by that column

4. **Trace Detail View**
   - Click any trace row
   - ✅ Expect: Modal dialog appears with trace details
   - ✅ Expect: Full request_id displayed
   - ✅ Expect: Provider, model, tokens, cost displayed
   - ✅ Expect: Metadata displayed (if available, as JSON)
   - Close modal
   - ✅ Expect: Return to traces list

5. **Real-Time Updates (if WebSocket connected)**
   - Trigger new API request from backend (if possible)
   - ✅ Expect: New trace appears at top of list (real-time)
   - ✅ Expect: No page refresh required

**Acceptance Criteria:**
- [ ] Traces list displays all traces with correct data
- [ ] Sorting by columns works correctly
- [ ] Trace detail view shows full trace data
- [ ] Real-time updates appear without refresh (if WebSocket active)

---

### Test 6: Responsive Layout

**Purpose:** Verify layout adapts correctly to different screen sizes.

**Procedure:**

1. **Desktop Layout (1920x1080)**
   - Open browser DevTools (F12)
   - Set viewport to 1920x1080
   - Navigate to each view (Dashboard, Tokens, Usage, Limits, Traces)
   - ✅ Expect: All content visible without horizontal scrolling
   - ✅ Expect: Tables display full width with all columns visible
   - ✅ Expect: Cards arranged in grid layout (2-3 columns)

2. **Tablet Layout (768x1024)**
   - Set viewport to 768x1024 (DevTools responsive mode)
   - Navigate to each view
   - ✅ Expect: Layout adapts (cards stack to 1-2 columns)
   - ✅ Expect: Tables remain readable (may scroll horizontally)
   - ✅ Expect: Header navigation remains functional

3. **Mobile Layout (390x844)**
   - Set viewport to 390x844 (iPhone 12 Pro)
   - Navigate to each view
   - ✅ Expect: Cards stack to single column
   - ✅ Expect: Tables scroll horizontally (if needed)
   - ✅ Expect: Navigation menu collapses to hamburger icon (if implemented)
   - ✅ Expect: All buttons remain tappable (min 44x44px touch target)

4. **Touch Interactions (Mobile)**
   - Use DevTools touch mode
   - ✅ Expect: All buttons respond to tap events
   - ✅ Expect: Modals open/close correctly
   - ✅ Expect: Form inputs focus correctly (no keyboard overlap)

**Acceptance Criteria:**
- [ ] Desktop layout uses full screen width efficiently
- [ ] Tablet layout adapts to smaller screen (stacked cards)
- [ ] Mobile layout works on 390px width (single column)
- [ ] Touch targets meet minimum 44x44px size

---

### Test 7: Keyboard Navigation

**Purpose:** Verify all functionality accessible via keyboard only (no mouse).

**Procedure:**

1. **Login Flow (Keyboard Only)**
   - Navigate to `http://localhost:5173`
   - Press `Tab` to focus username input
   - ✅ Expect: Username input focused (visible outline)
   - Type `test`
   - Press `Tab` to focus password input
   - ✅ Expect: Password input focused
   - Type `test`
   - Press `Tab` to focus submit button
   - Press `Enter` to submit
   - ✅ Expect: Login succeeds, redirect to dashboard

2. **Navigation Menu (Keyboard)**
   - Press `Tab` repeatedly to focus navigation links
   - ✅ Expect: Each link shows focus indicator (outline)
   - Press `Enter` on "Tokens" link
   - ✅ Expect: Navigate to `/tokens`

3. **Modal Dialogs (Keyboard)**
   - Press `Tab` to focus "Create Token" button
   - Press `Enter` to open modal
   - ✅ Expect: Modal opens
   - ✅ Expect: Focus moves to first input in modal
   - Press `Tab` to move between form fields
   - ✅ Expect: Tab order logical (top-to-bottom, left-to-right)
   - Press `Escape` key
   - ✅ Expect: Modal closes
   - ✅ Expect: Focus returns to "Create Token" button

4. **Table Navigation (Keyboard)**
   - Navigate to Tokens view
   - Press `Tab` to focus first token row
   - ✅ Expect: Row highlighted or focused
   - Press `Down Arrow` (if implemented)
   - ✅ Expect: Next row focused
   - Press `Enter` on action button (Rotate/Revoke)
   - ✅ Expect: Action executes

5. **Form Submission (Keyboard)**
   - Open "Create Token" modal
   - Fill form using Tab navigation
   - Press `Enter` in last input field
   - ✅ Expect: Form submits (same as clicking "Create" button)

**Acceptance Criteria:**
- [ ] All interactive elements reachable via Tab key
- [ ] Focus indicators visible on all focused elements
- [ ] Tab order logical (matches visual layout)
- [ ] Enter key submits forms and activates buttons
- [ ] Escape key closes modals
- [ ] Arrow keys navigate tables (if implemented)

---

### Test 8: Screen Reader Compatibility

**Purpose:** Verify frontend usable with screen readers (NVDA on Windows, VoiceOver on macOS).

**Procedure:**

1. **Screen Reader Setup**
   - Windows: Install NVDA (https://www.nvaccess.org/)
   - macOS: Enable VoiceOver (Cmd+F5)
   - ✅ Expect: Screen reader announces page content

2. **Login Form (Screen Reader)**
   - Navigate to login page
   - ✅ Expect: Screen reader announces "Username" label for input
   - ✅ Expect: Screen reader announces "Password" label for input
   - ✅ Expect: Screen reader announces "Login" button

3. **Navigation Menu (Screen Reader)**
   - Navigate to dashboard
   - Tab through navigation menu
   - ✅ Expect: Each link announced with descriptive text ("Dashboard", "Tokens", etc.)
   - ✅ Expect: Current page indicated (e.g., "Dashboard, current page")

4. **Data Tables (Screen Reader)**
   - Navigate to Tokens view
   - Tab into token list table
   - ✅ Expect: Table announced as "Table with N rows"
   - ✅ Expect: Column headers announced (ID, Name, Created, Status)
   - ✅ Expect: Row data announced in logical order

5. **Modal Dialogs (Screen Reader)**
   - Open "Create Token" modal
   - ✅ Expect: Modal announced (e.g., "Dialog: Create Token")
   - ✅ Expect: Form fields announced with labels
   - ✅ Expect: Focus trapped inside modal (Tab doesn't leave modal)

6. **Action Buttons (Screen Reader)**
   - Tab to "Create Token" button
   - ✅ Expect: Announced as "Button: Create Token"
   - Tab to "Rotate" button (in token list)
   - ✅ Expect: Announced as "Button: Rotate token [token-name]"

7. **Error Messages (Screen Reader)**
   - Submit login form with invalid credentials
   - ✅ Expect: Error message announced immediately
   - ✅ Expect: Error associated with form (not just visual)

**Acceptance Criteria:**
- [ ] All form labels announced correctly
- [ ] Navigation menu accessible and current page indicated
- [ ] Tables announced with row/column structure
- [ ] Modals announced and focus trapped
- [ ] Action buttons have descriptive labels
- [ ] Error messages announced immediately

---

## Corner Case Verification Checklist

This section documents exhaustive corner cases across all frontend functionality. Each corner case must be manually verified before production release.

### Authentication Flow Corner Cases

#### Credential Input Edge Cases
- [ ] **Empty username**: Submit login with empty username field → Expect form validation error
- [ ] **Empty password**: Submit login with empty password field → Expect form validation error
- [ ] **Both empty**: Submit completely empty form → Expect validation errors
- [ ] **Whitespace-only username**: Username of just spaces → Should trim and reject
- [ ] **Whitespace-only password**: Password of just spaces → Should reject or trim
- [ ] **Very long username**: 1000+ character username → Should reject or truncate
- [ ] **Very long password**: 1000+ character password → Should reject or truncate
- [ ] **Special characters**: Username/password with `<>"'/\` → Should handle safely (no XSS)
- [ ] **XSS attempt**: `<script>alert('xss')</script>` in fields → Should escape, not execute
- [ ] **SQL injection attempt**: `' OR '1'='1` in fields → Backend should reject safely
- [ ] **Unicode credentials**: Chinese/Arabic/emoji characters → Should accept or reject consistently
- [ ] **Copy-paste credentials**: Paste from clipboard → Should work correctly
- [ ] **Autofill credentials**: Browser autofill → Should work correctly

#### Session Management Edge Cases
- [ ] **Token expiry during session**: JWT expires while viewing dashboard → Should detect and redirect to login
- [ ] **Concurrent logins**: Same user in two browser tabs → Both should work independently
- [ ] **Logout in one tab**: Logout in tab A, continue using tab B → Tab B should detect session loss
- [ ] **Page refresh during login**: Refresh while login request in progress → Should handle gracefully
- [ ] **Browser back after logout**: Click back button after logout → Should redirect to login
- [ ] **Deep link while logged out**: Navigate to /dashboard directly → Should redirect to login
- [ ] **localStorage cleared**: Manually clear localStorage while logged in → Next request should detect and redirect
- [ ] **Invalid token in localStorage**: Corrupt token value → Should detect on next request and redirect
- [ ] **Network error during login**: Network failure during POST /auth/login → Should show error, not crash
- [ ] **API timeout during login**: Backend takes 30s+ to respond → Should timeout with clear message

#### Session Persistence Edge Cases
- [ ] **Page refresh**: Refresh any page → Should restore session and page state
- [ ] **Browser close and reopen**: Close browser, reopen, navigate to app → Session should persist (if not expired)
- [ ] **Multiple windows**: Open app in two windows with same session → Both should work
- [ ] **Private browsing mode**: Login in private/incognito mode → Should work, logout clears session completely

### Token Management UI Corner Cases

#### Token Creation Edge Cases
- [ ] **Empty token name**: Create token with blank name field → Should reject with validation error
- [ ] **Whitespace-only name**: Name of just spaces → Should trim and reject
- [ ] **Very long name**: 1000+ character token name → Should reject or truncate
- [ ] **Special characters in name**: `<>"'/\` → Should escape safely
- [ ] **XSS in name**: `<script>` in name → Should escape, not execute in table display
- [ ] **Emoji in name**: `Token 🚀 Test` → Should display correctly or reject
- [ ] **Unicode in name**: Chinese/Arabic characters → Should display correctly or reject
- [ ] **Duplicate name**: Create two tokens with same name → Should both exist (name not unique constraint)
- [ ] **Empty project_id**: Optional field left blank → Should accept
- [ ] **Special characters in project_id**: Same as name tests → Should handle safely
- [ ] **Network error during create**: API failure → Should show error, not create partial token
- [ ] **API timeout during create**: Backend takes 30s+ → Should timeout with clear message
- [ ] **Concurrent creates**: Click create button rapidly 5 times → Should handle gracefully (disable button or queue)

#### Token Display Edge Cases
- [ ] **Zero tokens**: Empty token list → Should show empty state message
- [ ] **Single token**: One token in list → Should display correctly
- [ ] **100+ tokens**: Very large token list → Should render without performance issues (or paginate)
- [ ] **Token string masking**: Full token never displayed after creation → Verify mask format consistent
- [ ] **Token copy functionality**: Click copy button → Should copy to clipboard successfully
- [ ] **Copy on different browsers**: Test clipboard API on Chrome, Firefox, Safari → Should work on all
- [ ] **Copy without clipboard permission**: Browser blocks clipboard access → Should show fallback (manual copy or error)
- [ ] **Timestamp formatting**: "Created" dates display correctly → Verify format (relative or absolute)
- [ ] **Status display**: Active vs Revoked status → Should show clearly differentiated

#### Token Rotation Edge Cases
- [ ] **Rotate active token**: Normal rotation → Should succeed, show new token once
- [ ] **Rotate twice rapidly**: Click rotate button twice quickly → Should queue or show error
- [ ] **Rotate while API request using that token**: Token in use → Should invalidate immediately
- [ ] **Rotate during network error**: API failure → Should not change token state locally
- [ ] **Copy rotated token**: Copy button on rotation modal → Should work correctly
- [ ] **Close modal without copying**: Show new token, close modal → Token lost (intentional security)

#### Token Revocation Edge Cases
- [ ] **Revoke active token**: Normal revoke → Should succeed, status changes to Revoked
- [ ] **Revoke already-revoked**: Revoke same token twice → Should handle gracefully (idempotent or error)
- [ ] **Revoke while API request using that token**: Token in use → Should invalidate immediately
- [ ] **Cancel revoke confirmation**: Click Revoke → Cancel in dialog → Should not revoke
- [ ] **Network error during revoke**: API failure → Should not change status locally

### Usage Analytics UI Corner Cases

#### Zero Data Display
- [ ] **Zero usage records**: No data from API → Should show "No usage data" empty state
- [ ] **Zero total requests**: All usage metrics zero → Should display 0, not crash or show NaN
- [ ] **Zero total cost**: $0.00 cost → Should display correctly formatted
- [ ] **Zero input/output tokens**: Zero tokens but requests exist → Should display 0

#### Large Number Display
- [ ] **Very large request count**: 1,000,000+ requests → Should format with commas (1,000,000)
- [ ] **Very large token count**: 1,000,000,000+ tokens → Should format correctly
- [ ] **Very large cost**: $10,000+ cost → Should format correctly ($10,000.00)
- [ ] **Very small cost**: $0.001 cost → Should round or display with precision
- [ ] **Integer overflow**: Counts exceeding JavaScript Number.MAX_SAFE_INTEGER → Should handle gracefully

#### Invalid Data Handling
- [ ] **Negative cost from API**: Backend returns negative cost → Should handle (show error or clamp to 0)
- [ ] **NULL values**: API returns null for cost/requests → Should treat as 0 or show error
- [ ] **NaN values**: API returns non-numeric values → Should show error, not display NaN
- [ ] **Missing provider breakdown**: API missing provider data → Should show partial data or error

#### Chart/Graph Corner Cases (If Implemented)
- [ ] **Chart with zero data**: No usage history → Should show empty chart with message
- [ ] **Chart with single data point**: Only one day of usage → Should render without breaking
- [ ] **Chart with 1000+ data points**: Very dense chart → Should handle performance or aggregate
- [ ] **Chart resize**: Resize browser window → Chart should resize responsively
- [ ] **Chart with negative values**: API returns invalid negative values → Should handle or reject

#### Real-Time Updates (WebSocket)
- [ ] **New usage record arrives**: WebSocket pushes new trace → UI should update without refresh
- [ ] **WebSocket disconnects**: Connection drops → Should show disconnected status
- [ ] **WebSocket reconnects**: Connection restored → Should resume real-time updates
- [ ] **Stale data during disconnect**: Updates missed while offline → Should refresh on reconnect
- [ ] **Very frequent updates**: 10+ traces per second → Should throttle/batch updates for performance

#### Network Error Cases
- [ ] **API timeout**: Backend takes 30s+ for /api/usage/aggregate → Should show timeout error
- [ ] **API returns 500**: Server error → Should show user-friendly error, not crash
- [ ] **API returns 404**: Endpoint not found → Should show error
- [ ] **Network offline**: No internet connection → Should show network error, retry option

### Budget Limits UI Corner Cases

#### Form Validation Edge Cases
- [ ] **Empty limit value**: Submit form with blank value → Should show validation error
- [ ] **Zero limit value**: Enter 0 → Should reject (0 limit meaningless)
- [ ] **Negative limit value**: Enter -100 → Should reject
- [ ] **Non-numeric input**: Enter "abc" → Should reject with validation error
- [ ] **Decimal input**: Enter 99.99 → Should handle (round, accept, or reject based on spec)
- [ ] **Very large number**: Enter 999999999999 → Should accept if within i64 range
- [ ] **Number overflow**: Enter number > Number.MAX_SAFE_INTEGER → Should reject
- [ ] **Scientific notation**: Enter 1e6 → Should parse correctly or reject
- [ ] **Special characters**: Enter `<script>` → Should reject as non-numeric

#### CRUD Operation Edge Cases
- [ ] **Create duplicate limit**: Create two limits with same value → Should both exist (no uniqueness)
- [ ] **Update to same value**: Edit limit to same value → Should succeed (no-op)
- [ ] **Update to invalid value**: Edit limit to 0 or negative → Should reject
- [ ] **Delete during network error**: API failure → Should not remove from UI until confirmed
- [ ] **Concurrent updates**: Edit same limit in two tabs → Last write wins or error
- [ ] **Cancel create modal**: Open create modal → close without saving → Should not create
- [ ] **Cancel edit modal**: Open edit modal → close without saving → Should not update

#### Display Edge Cases
- [ ] **Zero limits**: Empty limits list → Should show empty state
- [ ] **100+ limits**: Very large list → Should render without performance issues (or paginate)
- [ ] **Limit type display**: Different types (daily/monthly/total) → Should display clearly
- [ ] **Value formatting**: Large numbers (1,000,000) → Should format with commas

### Request Traces UI Corner Cases

#### Trace List Display Edge Cases
- [ ] **Zero traces**: Empty trace list → Should show empty state message
- [ ] **Single trace**: One trace in list → Should display correctly
- [ ] **1000+ traces**: Very large trace list (no pagination) → Should handle performance or show warning
- [ ] **Very long request_id**: 1000+ character ID → Should truncate or wrap in table cell
- [ ] **Very large cost**: $100+ cost per trace → Should format correctly
- [ ] **Zero cost trace**: Cached request with $0.00 cost → Should display correctly
- [ ] **Very large token counts**: 100,000+ tokens → Should format with commas
- [ ] **Timestamp display**: Various timestamps (recent, old, future) → Should format consistently

#### Trace Detail Modal Edge Cases
- [ ] **Modal with minimal data**: Only required fields → Should display, empty optionals not shown
- [ ] **Modal with all data**: All fields populated → Should display fully
- [ ] **Very large metadata**: 1MB+ JSON metadata → Should handle (pretty-print, scroll, or truncate)
- [ ] **Malformed JSON metadata**: Invalid JSON → Should show parse error
- [ ] **XSS in metadata**: `<script>` in JSON → Should escape, not execute
- [ ] **Unicode in metadata**: Chinese/Arabic/emoji in JSON → Should display correctly

#### Sorting and Filtering Edge Cases
- [ ] **Sort by each column**: Click all column headers → Should sort correctly (asc/desc)
- [ ] **Sort with equal values**: Multiple traces with same cost → Should maintain stable sort
- [ ] **Sort empty list**: Sort when no traces → Should not crash
- [ ] **Filter with zero results**: Filter criteria matches nothing → Should show "no results" message
- [ ] **Filter with one result**: Only one match → Should display correctly

#### Real-Time Trace Updates (WebSocket)
- [ ] **New trace arrives**: WebSocket pushes trace → Should prepend to list (most recent first)
- [ ] **Very frequent traces**: 10+ per second → Should throttle/batch for performance
- [ ] **Trace updates while scrolled down**: New trace at top while viewing bottom → Should notify or auto-scroll
- [ ] **Trace arrives during sort**: New data while sorted by column → Should maintain sort order
- [ ] **WebSocket disconnected**: No real-time updates → Should show status indicator

### Responsive Layout Corner Cases

#### Viewport Size Edge Cases
- [ ] **Minimum width (320px)**: Smallest mobile viewport → All content should fit (single column)
- [ ] **Tablet portrait (768px)**: Mid-size viewport → Should adapt layout (2 columns or stack)
- [ ] **Laptop (1366px)**: Standard laptop → Should use full width efficiently
- [ ] **4K display (3840px)**: Very large viewport → Should not stretch excessively
- [ ] **Viewport resize during use**: Drag browser window smaller/larger → Should resize responsively
- [ ] **Orientation change**: Rotate mobile device → Should adapt layout
- [ ] **Browser zoom 50%**: Zoom out → Content should remain readable
- [ ] **Browser zoom 200%**: Zoom in → Layout should not break, scroll if needed

#### Modal Behavior at Different Sizes
- [ ] **Modal on small viewport**: Open modal on 320px viewport → Should fit without overflow
- [ ] **Modal on large viewport**: Open modal on 4K display → Should not stretch excessively
- [ ] **Modal scrolling**: Modal content taller than viewport → Should scroll internally
- [ ] **Background scroll lock**: Modal open → Background should not scroll

#### Table Responsiveness
- [ ] **Table on small viewport**: Token/trace tables on mobile → Should scroll horizontally or stack
- [ ] **Table with many columns**: All columns visible or responsive design → Should handle overflow
- [ ] **Table with long content**: Very long text in cells → Should wrap or truncate

### Keyboard Navigation Corner Cases

#### Tab Order Edge Cases
- [ ] **Tab through empty page**: No interactive elements → Tab does nothing
- [ ] **Tab through full page**: All elements → Tab order should match visual layout (left-to-right, top-to-bottom)
- [ ] **Tab skips hidden elements**: Display:none elements → Should not receive focus
- [ ] **Tab through disabled elements**: Disabled buttons → Should skip
- [ ] **Shift+Tab reverse order**: Navigate backwards → Should reverse through same elements

#### Focus Management Edge Cases
- [ ] **Focus visible**: Tab to each element → Should show focus indicator (outline/ring)
- [ ] **Focus color contrast**: Focus indicator → Should meet WCAG contrast requirements
- [ ] **Focus trap in modal**: Tab within modal → Should cycle within modal, not escape
- [ ] **Focus restoration**: Close modal → Focus returns to trigger button
- [ ] **Focus lost**: Element receiving focus is removed → Should move to next focusable element

#### Keyboard Shortcuts
- [ ] **Enter on buttons**: Press Enter on focused button → Should activate
- [ ] **Space on buttons**: Press Space on focused button → Should activate
- [ ] **Enter on links**: Press Enter on focused link → Should navigate
- [ ] **Escape closes modal**: Press Esc in modal → Should close
- [ ] **Escape in nested modals**: Multiple modals open → Should close top-most

### Screen Reader Corner Cases

#### Table Announcements
- [ ] **Table structure**: Screen reader on table → Should announce "Table with N rows"
- [ ] **Column headers**: Navigate table cells → Should announce column header with data
- [ ] **Empty table**: Table with no rows → Should announce empty state
- [ ] **Sorted table**: Sorted column → Should announce sort direction

#### Form Announcements
- [ ] **Label association**: Focus input → Should announce label text
- [ ] **Required fields**: Focus required input → Should announce required state
- [ ] **Error messages**: Submit invalid form → Should announce error immediately
- [ ] **Placeholder text**: Input with placeholder → Should not rely only on placeholder

#### Dynamic Content
- [ ] **ARIA live regions**: New trace arrives → Should announce "New trace added" (if implemented)
- [ ] **Loading states**: Data fetching → Should announce loading
- [ ] **Error states**: API error → Should announce error message

### Browser Compatibility Corner Cases

#### Browser Storage Edge Cases
- [ ] **localStorage unavailable**: Private browsing mode → Should show error or use fallback
- [ ] **localStorage quota exceeded**: Store very large data → Should handle quota error
- [ ] **Cookies disabled**: If using cookies → Should detect and show error
- [ ] **sessionStorage vs localStorage**: Session persistence → Should use correct storage type

#### API Support Edge Cases
- [ ] **WebSocket not supported**: Old browser → Should fallback to polling or show warning
- [ ] **Fetch API not supported**: Old browser → Should use XHR fallback or polyfill
- [ ] **Clipboard API not available**: No clipboard access → Should show manual copy fallback
- [ ] **CSS Grid not supported**: Old browser → Should use flexbox fallback

#### JavaScript Edge Cases
- [ ] **JavaScript disabled**: User disabled JS → Should show "Please enable JavaScript" message
- [ ] **JavaScript error**: Uncaught exception → Should show error boundary, not white screen
- [ ] **Service worker not supported**: Offline functionality → Should still work online

### Network and Performance Corner Cases

#### Network Conditions
- [ ] **Slow network (3G)**: Simulate slow connection → Should show loading states, not timeout
- [ ] **Network offline**: Disconnect internet → Should show offline message, retry option
- [ ] **Intermittent network**: Connection drops and returns → Should handle reconnection gracefully
- [ ] **API timeout**: Backend takes 60s+ → Should timeout with clear message (default fetch timeout)

#### Concurrent Requests
- [ ] **Multiple concurrent API calls**: Load page with many requests → Should handle all
- [ ] **Request deduplication**: Same request made twice → Should deduplicate or allow both
- [ ] **Race condition**: Two requests for same data → Should handle last response wins

#### Cache and State Management
- [ ] **Stale cache**: Cached data older than fresh API response → Should use fresh data
- [ ] **Cache invalidation**: Create/update/delete operation → Should invalidate related cache
- [ ] **Optimistic updates**: UI updates before API confirmation → Should rollback on error

#### Performance Degradation
- [ ] **Large DOM**: 1000+ elements rendered → Should not cause jank or freezing
- [ ] **Memory leak**: Use app for extended period → Should not consume excessive memory
- [ ] **Event listener leak**: Add/remove components → Should clean up event listeners

### Input Sanitization Corner Cases

#### XSS Prevention
- [ ] **Script tags in input**: `<script>alert('xss')</script>` → Should escape, not execute
- [ ] **Event handlers in input**: `<img src=x onerror=alert('xss')>` → Should escape
- [ ] **JavaScript URLs**: `javascript:alert('xss')` → Should sanitize or reject
- [ ] **Data URLs**: `data:text/html,<script>...` → Should sanitize or reject

#### HTML Injection Prevention
- [ ] **HTML tags in text**: `<b>bold</b>` → Should escape (display as text, not render)
- [ ] **Unclosed tags**: `<div>` without closing → Should escape
- [ ] **Style injection**: `<style>body{display:none}</style>` → Should escape

---

## Test Execution Checklist

**Before Testing:**
- [ ] Backend (iron_control_api) running on http://localhost:3001
- [ ] Frontend dev server running on http://localhost:5173
- [ ] Browser DevTools open (Console tab for error monitoring)
- [ ] Test credentials confirmed: `test` / `test`

**During Testing:**
- [ ] No console errors in browser DevTools
- [ ] All API responses return within 500ms (localhost)
- [ ] All UI updates appear within 200ms

**After Testing:**
- [ ] All 8 test categories passed
- [ ] All acceptance criteria met
- [ ] Any failures documented with screenshots/details

---

## Known Issues

**Current Issues (as of migration):**
1. **No WebSocket reconnection UI** - No visible indicator when connection drops
2. **No loading states** - Tables appear empty during loading (fast on localhost)
3. **No pagination** - Performance degrades with >1000 traces

**Workarounds:**
1. Refresh page manually if WebSocket disconnects
2. Wait 1-2 seconds for initial data load
3. Limit demo to <100 traces

---

## Responsibility Table

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|----------------|--------------|-------|--------------|
| `readme.md` | Manual testing procedures and acceptance criteria | - → Test plan | All 8 test categories, step-by-step procedures, acceptance criteria | Automated tests (../unit/, ../component/), test results (external) |

**Complete Entity Coverage Verified:** 1 entity listed (all files in manual/).

---

## Testing Report Template

After completing manual testing, document results:

```markdown
## Manual Testing Report - [Date]

**Tester:** [Name]
**Duration:** [Minutes]
**Environment:**
- Browser: [Chrome 120 / Firefox 120 / Safari 17]
- Backend: [iron_control_api version / commit hash]
- Frontend: [git commit hash]

### Test Results

| Test Category | Status | Notes |
|--------------|--------|-------|
| 1. Authentication Flow | ✅ PASS | All flows working |
| 2. Token Management | ✅ PASS | Create/rotate/revoke successful |
| 3. Usage Analytics | ✅ PASS | Data accurate |
| 4. Budget Limits | ✅ PASS | CRUD operations working |
| 5. Request Traces | ✅ PASS | Real-time updates working |
| 6. Responsive Layout | ⚠️ PARTIAL | Mobile layout needs adjustment |
| 7. Keyboard Navigation | ✅ PASS | All elements accessible |
| 8. Screen Reader | ✅ PASS | NVDA compatibility confirmed |

### Issues Found

1. **Issue:** Mobile navigation menu overlaps content
   - **Severity:** Minor
   - **Reproduction:** Resize to 390px, open navigation
   - **Workaround:** Use desktop layout for demo

### Recommendations

- Fix mobile navigation overlap before production
- Add loading states for better UX
- Consider pagination for traces view
```

---

**End of Manual Testing Documentation**
