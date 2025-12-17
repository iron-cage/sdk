# Sitemap and Navigation Structure

## Purpose

Visual representation of iron_dashboard's navigation hierarchy, route structure, and page flow. This document serves as a quick reference for understanding the application's information architecture and navigation patterns.

---

## Scope

**Included:**
- Route hierarchy and URL structure
- Navigation menu organization
- Page relationships and flow
- Authentication-based access control

**Excluded:**
- Route implementation details → See `architecture.md` (Routing Strategy section)
- API endpoints for data → See `api_integration.md`
- Component implementation → See `components.md`
- Requirements and acceptance criteria → See `spec.md`

---

## Route Hierarchy

```
/
├── /login (public)
│   └── Redirects to /dashboard if authenticated
│
├── /dashboard (authenticated) ← Default redirect from /
│   ├── Shows: Agent status, budget metrics, active agents
│   └── Real-time: WebSocket updates for agent status
│
├── /agents (authenticated)
│   ├── Shows: Agent list with status, owner, providers
│   ├── Actions: Create, edit, delete agents (admin only for edit/delete)
│   └── Features: Owner assignment (admin only), IC token generation
│
├── /usage (authenticated)
│   ├── Shows: Cost breakdown by provider/model/time period
│   ├── Data: Total cost, requests, tokens consumed
│   └── Features: Recent logs with "Load More" pagination
│
├── /limits (authenticated)
│   ├── Shows: Budget limits per agent
│   └── Actions: Update budget (admin only)
│
├── /providers (authenticated)
│   ├── Shows: Provider keys list
│   └── Actions: Create, update, delete provider keys (admin only)
│
├── /users (authenticated)
│   ├── Shows: User list with roles
│   └── Actions: Create, update, delete users (admin only)
│
└── /budget-requests (authenticated)
    ├── Shows: Budget increase requests
    └── Actions: Approve/reject requests (admin only)
```

---

## Navigation Schema

### Primary Navigation (Sidebar)

**Location:** Left sidebar (collapsible on mobile)
**Component:** `src/components/MainLayout.vue`

**Menu Structure:**
1. **Dashboard** (`/dashboard`)
   - Icon: Home
   - Always visible when authenticated
   - Default landing page after login

2. **Agents** (`/agents`)
   - Icon: Users
   - Agent management (create, edit, delete)
   - Owner assignment (admin only)

3. **Usage Analytics** (`/usage`)
   - Icon: Bar Chart
   - Cost and usage visualization
   - Recent logs with pagination

4. **Limits** (`/limits`)
   - Icon: Lock
   - Budget status per agent
   - Budget modification (admin only)

5. **Providers** (`/providers`)
   - Icon: Key
   - Provider key management (admin only)

6. **Users** (`/users`)
   - Icon: Users
   - User management (admin only)

7. **Budget Requests** (`/budget-requests`)
   - Icon: Document
   - Budget increase request approval (admin only)

### Secondary Navigation (Header)

**Location:** Top-right header
**Component:** `src/components/MainLayout.vue:164-172`

**Elements:**
- Username display (from auth store)
- Logout button → Redirects to `/login`

---

## Page Flow Diagrams

### Authentication Flow

```
┌─────────────┐
│  /login     │ ← Entry point (unauthenticated)
└─────┬───────┘
      │
      │ Login success
      ▼
┌─────────────┐
│ /dashboard  │ ← Default authenticated page
└─────────────┘
      │
      │ Logout
      ▼
┌─────────────┐
│  /login     │ ← Returns here
└─────────────┘
```

### Typical User Journey (Conference Demo)

```
1. Login (/login)
   └──> Username: "demo", Password: [configured]

2. Dashboard (/dashboard)
   └──> View agent status, budget metrics

3. Tokens (/tokens)
   └──> Create new token for agent

4. Usage Analytics (/usage)
   └──> View cost breakdown by provider

5. Traces (/traces)
   └──> Inspect detailed request trace

6. Logout
   └──> Return to /login
```

---

## Access Control Matrix

| Route             | Auth Required | Admin Only | Redirect If Unauth |
|-------------------|---------------|------------|---------------------|
| `/login`          | No            | No         | → `/dashboard` if auth |
| `/dashboard`      | Yes           | No         | → `/login`          |
| `/agents`         | Yes           | Edit/Delete only | → `/login`     |
| `/usage`          | Yes           | No         | → `/login`          |
| `/limits`         | Yes           | Budget edit only | → `/login`     |
| `/providers`      | Yes           | Yes        | → `/login`          |
| `/users`          | Yes           | Yes        | → `/login`          |
| `/budget-requests`| Yes           | Approve only | → `/login`        |
| `/` (root)        | -             | -          | → `/dashboard`      |

**Role-Based Data Filtering:**
- **Admin:** Sees all agents, budgets, and analytics
- **User:** Sees only owned agents and their data

**Implementation:** `src/router/index.ts` (navigation guard using `meta.requiresAuth`)

---

## Navigation Patterns

### Sidebar Toggle Behavior

**Desktop (≥1024px):**
- Sidebar always visible
- Content area offset by 256px (`ml-64`)
- Smooth transition when toggled

**Mobile (<1024px):**
- Sidebar hidden by default
- Overlay when opened (z-index: 50)
- Closes on route navigation

**State Management:**
- Local component state (`ref(true)` in MainLayout)
- Not persisted across sessions
- Resets to open on page reload

### Active Route Indication

**Pattern:** Vue Router's `router-link` auto-applies active classes
**Styling:** Active link shows different background/text color
**Implementation:** Tailwind classes in MainLayout sidebar navigation

---

## Route Definitions (Source Reference)

**File:** `src/router/index.ts`

All routes use lazy loading except `/login`:
```typescript
{ path: '/login', component: LoginView }  // Eager load
{ path: '/dashboard', component: () => import('../views/DashboardView.vue') }  // Lazy load
```

**Rationale for Lazy Loading:**
- Reduces initial bundle size
- Faster time-to-interactive for login page
- Dashboard/authenticated routes loaded after login success

---

## Future Expansion Points

**When adding new routes:**

1. **Update router definition** (`src/router/index.ts`)
   ```typescript
   {
     path: '/new-page',
     name: 'new-page',
     component: () => import('../views/NewPageView.vue'),
     meta: { requiresAuth: true },
   }
   ```

2. **Add navigation link** (`src/components/MainLayout.vue`)
   - Insert `<router-link>` in sidebar navigation
   - Choose appropriate icon from Heroicons
   - Follow existing spacing/styling patterns

3. **Update this sitemap**
   - Add route to hierarchy diagram
   - Update access control matrix
   - Document page purpose and relationships

4. **Update spec.md** if new functional requirement

---

## Decision Rationale

**Why Single-Level Navigation?**
- Conference demo scope: 6 pages, single user
- Nested navigation adds complexity without value
- All pages equally important (no hierarchy needed)
- Future: Can add nested routes for settings, admin, etc.

**Why Sidebar Navigation?**
- Persistent visibility improves discoverability
- Consistent with dashboard/admin UI conventions
- Mobile: Collapsible sidebar preserves screen space
- Alternative breadcrumbs unnecessary for flat hierarchy

**Why Redirect / to /dashboard?**
- Dashboard is primary user destination
- Avoids blank root page
- Consistent with SPA conventions (single entry point)

---

## References

- **Route Implementation:** `src/router/index.ts`
- **Navigation Component:** `src/components/MainLayout.vue`
- **Authentication Guard Logic:** `src/router/index.ts:52-63`
- **View Components:** `src/views/*.vue`
- **Routing Architecture:** `docs/architecture.md` (Routing Strategy section)

---

**End of Sitemap Documentation**
