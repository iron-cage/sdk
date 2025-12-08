# spec

**Version:** 0.2
**Date:** 2025-12-07
**Component:** iron_dashboard
**Layer:** Application (Layer 6)
**Status:** Implementation Complete (95%), Documentation New (100%)
**Priority:** REQUIRED for Pilot (Dashboard for Conference Demo)

---

## 1. Overview

### 1.1 Purpose

**iron_dashboard** provides a **web-based dashboard** for real-time monitoring and management of Iron Cage agents, token usage, and budget controls. It serves as the primary user interface for the conference demo, displaying live agent execution, cost tracking, and safety metrics.

**Primary Responsibilities:**
- Real-time agent execution monitoring via WebSocket
- Token management (create, rotate, revoke API tokens)
- Usage analytics and cost tracking (per-provider, per-model)
- Budget limit management and alerts
- Request trace visualization
- Authentication and session management

**Pilot Scope:** Dashboard displaying agent execution metrics, token management, and basic analytics for 5-minute conference demo.

**Full Platform (Out of Scope):** Multi-user management, RBAC, audit logs, custom dashboards, mobile app, advanced analytics (charts, trends).

### 1.2 Design Principles

1. **Real-Time by Default** - WebSocket updates for instant feedback
2. **Responsive Design** - Desktop-first, mobile-compatible layouts
3. **Accessible by Default** - ARIA labels, keyboard navigation, screen reader support
4. **Type-Safe Integration** - TypeScript types match backend Rust schemas
5. **Composition API** - Vue 3 `<script setup>` for clarity and performance
6. **Fail-Safe Error Handling** - All API errors displayed to user with actionable messages

---

## 2. Scope

### 2.1 In Scope (Pilot)

**For Pilot Project (Conference Demo):**
- Authentication view (username/password login)
- Dashboard view (agent list, real-time metrics, budget status)
- Token management view (create, rotate, revoke tokens)
- Usage analytics view (cost breakdown by provider/model)
- Budget limits view (create, update, delete limits)
- Request traces view (detailed per-request data)
- Responsive layout (desktop: 1920x1080, mobile: 390x844)
- Dark/light theme toggle
- Session persistence (localStorage)
- REST API integration (iron_api endpoints)
- WebSocket integration (real-time updates)

**Rationale:** Dashboard demonstrates Iron Cage capabilities to conference attendees. Real-time updates showcase monitoring and safety features.

### 2.2 Out of Scope (Full Platform)

**Deferred to Post-Pilot:**
- Multi-user management (user roles, permissions) → Pilot is single-user
- Role-based access control (RBAC) → Pilot has no roles
- Audit log viewer (historical actions) → Pilot has no audit trail
- Custom dashboard builder (drag-drop widgets) → Pilot uses fixed layout
- Mobile native app (iOS, Android) → Pilot is web-only
- Advanced analytics (charts, trends, forecasts) → Pilot uses tables
- Export functionality (CSV, PDF reports) → Pilot is view-only
- Internationalization (i18n) → Pilot is English-only
- Notification system (push, email, SMS) → Pilot uses in-app alerts only
- Collaborative features (comments, sharing) → Pilot is single-user

**Reasoning:** Conference demo runs on presenter's laptop with single user. Multi-user and enterprise features add complexity without demo value.

### 2.3 Deployment Context

Iron Cage supports two deployment modes. This module's behavior differs between modes.

**See:** [docs/deployment_packages.md](../../docs/deployment_packages.md) § Deployment Modes for complete architecture.

**This Module (iron_dashboard):**

**Pilot Mode:**
- Connects to WebSocket at `ws://localhost:8080/ws` for real-time updates
- All components (iron_api, iron_runtime, dashboard) on same localhost machine
- Single-user session (no authentication required for demo)

**Production Mode:**
- Connects to Control Panel WebSocket at `wss://control.example.com/ws` (HTTPS/TLS)
- Control Panel runs on cloud infrastructure (separate from local Agent Runtime)
- Multi-user authentication via iron_control_store (PostgreSQL)
- Dashboard displays aggregated metrics from multiple distributed agents

---

## 3. Functional Requirements

### FR-1: Authentication & Authorization

**Requirement:**
Provide secure login/logout flow with session persistence.

**Login Flow:**
1. User navigates to `http://localhost:5173`
2. Frontend checks localStorage for valid token
3. If no token → redirect to `/login`
4. User enters credentials (demo: `test` / `test`)
5. Frontend sends `POST /api/auth/login` to backend
6. Backend returns JWT token
7. Frontend stores token in localStorage
8. Frontend redirects to `/dashboard`

**Logout Flow:**
1. User clicks "Logout" button
2. Frontend removes token from localStorage
3. Frontend redirects to `/login`

**Session Persistence:**
- Token stored in localStorage (key: `auth_token`)
- Token validated on each protected route navigation
- Token expires after 24 hours (backend enforces)

**UI Components:**
- Login form (username input, password input, submit button)
- Logout button (header navigation)
- Error messages (invalid credentials, network errors)

**Out of Scope:** Multi-factor authentication (MFA), password reset, OAuth providers.

---

### FR-2: Dashboard View

**Requirement:**
Display agent execution status, budget metrics, and system health.

**Dashboard Layout:**
```
┌──────────────────────────────────────────────────────────┐
│ Header: Iron Cage | Dashboard | Logout                    │
├──────────────────────────────────────────────────────────┤
│                                                            │
│  Budget Status                    Agent Execution         │
│  ┌────────────────────┐          ┌──────────────────┐    │
│  │ $23.14 / $50.00    │          │ Agent: lead_gen  │    │
│  │ 46.3% used         │          │ Status: RUNNING  │    │
│  │ $26.86 remaining   │          │ Cost: $12.45     │    │
│  └────────────────────┘          │ Duration: 2m 34s │    │
│                                   └──────────────────┘    │
│                                                            │
│  Recent Requests                                          │
│  ┌──────────────────────────────────────────────────────┐│
│  │ Model        Input  Output  Cost   Time             ││
│  ├──────────────────────────────────────────────────────┤│
│  │ gpt-4o       1234   567     $0.12  14:23:45         ││
│  │ claude-3.5   890    234     $0.08  14:23:42         ││
│  │ gpt-4o-mini  456    123     $0.03  14:23:38         ││
│  └──────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────┘
```

**Data Sources:**
- Budget: `GET /api/budget`
- Agents: `GET /api/agents`
- Recent requests: WebSocket stream `/ws/traces`

**Update Frequency:**
- Budget: Poll every 5 seconds
- Agents: Poll every 2 seconds
- Requests: Real-time WebSocket updates

**Out of Scope:** Historical charts (cost over time), agent comparison tables.

---

### FR-3: Token Management

**Requirement:**
Allow users to create, rotate, and revoke API tokens.

**Token List View:**
- Display all tokens (id, name, created_at, last_used_at, status)
- Columns: ID, Name, Created, Last Used, Status (Active/Revoked)
- Actions: Create Token, Rotate Token, Revoke Token

**Create Token Flow:**
1. User clicks "Create Token" button
2. Frontend shows modal dialog
3. User enters token name (optional), project_id (optional)
4. User clicks "Create"
5. Frontend sends `POST /api/tokens` with data
6. Backend returns token string (ONLY shown once)
7. Frontend displays token in modal with "Copy" button
8. User copies token (clipboard API)
9. Frontend adds new token to list

**Rotate Token Flow:**
1. User clicks "Rotate" button for token
2. Frontend shows confirmation dialog
3. User confirms
4. Frontend sends `POST /api/tokens/:id/rotate`
5. Backend invalidates old token, returns new token
6. Frontend displays new token (ONLY shown once)

**Revoke Token Flow:**
1. User clicks "Revoke" button for token
2. Frontend shows confirmation dialog ("Are you sure?")
3. User confirms
4. Frontend sends `POST /api/tokens/:id/revoke`
5. Backend marks token as inactive
6. Frontend updates token status in list

**Security:**
- Tokens displayed only once after creation/rotation
- Masked in list view (e.g., `iron_***************abcd`)
- Revoked tokens cannot be reactivated (create new instead)

**Out of Scope:** Token scopes (permissions), token expiration (TTL), token usage limits.

---

### FR-4: Usage Analytics

**Requirement:**
Display token usage statistics with cost breakdown by provider and model.

**Usage Overview:**
- Total requests (count)
- Total input tokens (count)
- Total output tokens (count)
- Total cost (USD)

**By Provider:**
- Provider name (OpenAI, Anthropic, etc.)
- Request count
- Total cost

**By Model:**
- Model name (gpt-4o, claude-3.5-sonnet, etc.)
- Request count
- Total cost

**Data Source:**
- `GET /api/usage/stats` → Returns UsageStats object

**UI Components:**
- Summary cards (total requests, tokens, cost)
- Provider breakdown table (provider, requests, cost)
- Model breakdown table (model, requests, cost)
- Date range filter (optional for pilot)

**Out of Scope:** Historical trends, cost forecasting, budget alerts.

---

### FR-5: Budget Limits

**Requirement:**
Allow users to create, update, and delete budget limits.

**Limit Types:**
- `daily_cost` - Maximum daily cost (USD)
- `monthly_cost` - Maximum monthly cost (USD)
- `per_request_cost` - Maximum cost per request (USD)
- `total_requests` - Maximum requests per period

**Limit Periods:**
- `daily` - Resets at midnight UTC
- `monthly` - Resets on 1st of month UTC
- `total` - Never resets (lifetime limit)

**Limit List View:**
- Display all limits (id, type, value, period, status)
- Columns: ID, Type, Limit, Period, Status (Active/Disabled)
- Actions: Create Limit, Edit Limit, Delete Limit

**Create Limit Flow:**
1. User clicks "Create Limit" button
2. Frontend shows modal dialog
3. User selects limit type (dropdown)
4. User enters limit value (number input)
5. User selects period (dropdown)
6. User clicks "Create"
7. Frontend sends `POST /api/limits` with data
8. Backend creates limit, returns LimitRecord
9. Frontend adds new limit to list

**Update Limit Flow:**
1. User clicks "Edit" button for limit
2. Frontend shows modal dialog with current values
3. User modifies limit value or period
4. User clicks "Update"
5. Frontend sends `PUT /api/limits/:id` with data
6. Backend updates limit, returns updated LimitRecord
7. Frontend updates limit in list

**Delete Limit Flow:**
1. User clicks "Delete" button for limit
2. Frontend shows confirmation dialog
3. User confirms
4. Frontend sends `DELETE /api/limits/:id`
5. Backend deletes limit
6. Frontend removes limit from list

**Out of Scope:** Limit alerts (email, SMS), limit history, limit templates.

---

### FR-6: Request Traces

**Requirement:**
Display detailed per-request trace data for debugging and analysis.

**Trace List View:**
- Display all traces (id, request_id, provider, model, tokens, cost, timestamp)
- Columns: ID, Request ID, Provider, Model, Input Tokens, Output Tokens, Cost, Time
- Filtering: By provider, model, date range (optional for pilot)
- Sorting: By timestamp (descending by default)

**Trace Detail View:**
1. User clicks trace row
2. Frontend shows modal dialog with full trace details
3. Display: request_id, provider, model, input/output tokens, cost, timestamp, metadata (JSON)

**Data Source:**
- `GET /api/traces` → Returns TraceRecord[]
- `GET /api/traces/:id` → Returns TraceRecord (detail view)

**UI Components:**
- Trace table (sortable, filterable)
- Trace detail modal (expandable JSON metadata)
- Pagination (if >100 traces, optional for pilot)

**Out of Scope:** Trace search (by request_id), trace export (CSV), trace analytics.

---

## 4. Non-Functional Requirements

### NFR-1: Performance

**Target Metrics:**
- Initial page load: <2 seconds (localhost)
- Route navigation: <200ms
- WebSocket latency: <100ms (localhost)
- API response time: <500ms (localhost)

**Optimization Strategies:**
- Code splitting (lazy-loaded routes)
- Virtual scrolling (for long lists)
- Debounced API calls (search/filter inputs)
- Cached query results (TanStack Query, 5-minute stale time)

**Out of Scope:** CDN deployment, image optimization, service worker caching.

---

### NFR-2: Security

**Threat Model:**
- **In Scope:** XSS (Cross-Site Scripting), CSRF (Cross-Site Request Forgery), token leakage
- **Out of Scope:** DDoS, SQL injection (backend responsibility), man-in-the-middle (HTTPS)

**Mitigations:**
- XSS: Vue 3 template escaping (automatic), no `v-html` usage
- CSRF: SameSite cookies (backend), JWT tokens (stateless)
- Token leakage: Tokens shown only once, never logged, localStorage (not sessionStorage)
- CORS: Backend allows only `http://localhost:5173` (dev) and production domain

**Out of Scope:** Content Security Policy (CSP), Subresource Integrity (SRI), HSTS headers.

---

### NFR-3: Accessibility

**WCAG 2.1 Level AA Compliance:**
- Semantic HTML (nav, main, section, article)
- ARIA labels (buttons, inputs, links)
- Keyboard navigation (Tab, Enter, Escape)
- Focus indicators (visible outlines)
- Color contrast ratio ≥4.5:1 (text/background)
- Screen reader support (NVDA, JAWS tested)

**UI Framework:**
- **shadcn-vue** - Component library built on Radix Vue primitives
  - Copy-paste architecture (components in src/components/ui/)
  - 12 core components installed (Button, Dialog, Card, Input, Label, Badge, Select, Separator, Skeleton, Alert, Toast, DropdownMenu)
  - Full TypeScript support with prop types
  - Customizable via Tailwind CSS (tailwind.config.js)
  - Accessible by default (WCAG 2.1 AA compliance)
- **Radix Vue 1.9.17** - Underlying accessible primitives (Dialog, Select, etc.)
  - Provides focus management, keyboard navigation, ARIA attributes
  - Used by shadcn-vue for accessibility foundation
- **Tailwind CSS 3.4.17** - Utility-first CSS framework
  - CSS variables for theming (light/dark mode support)
  - Component styling via utility classes
  - Custom theme in tailwind.config.js (colors, radius, animations)
- **class-variance-authority 0.7.1** - Variant-based component styling
  - Used for Button variants (default, secondary, outline, ghost, link, destructive)
  - Used for Badge variants (default, secondary, destructive, outline)

**Rationale**: shadcn-vue provides production-ready components with consistent styling, full accessibility, and minimal bundle overhead. Copy-paste architecture allows customization without ejecting from framework.

**Out of Scope:** WCAG 2.2 Level AAA, internationalization (i18n), custom theme builder UI.

---

### NFR-4: Browser Compatibility

**Supported Browsers:**
- Chrome 120+ (primary)
- Firefox 120+
- Safari 17+
- Edge 120+

**Not Supported:**
- Internet Explorer (any version)
- Chrome <100
- Safari <16

**Polyfills:**
- None required (Vite transpiles to ES2020)

**Out of Scope:** Mobile browsers (iOS Safari, Chrome Android).

---

### NFR-5: Maintainability

**Code Quality:**
- TypeScript strict mode enabled
- ESLint configured (Vue + TypeScript rules)
- Prettier configured (2-space indent, single quotes)
- No `any` types (use `unknown` instead)
- No `@ts-ignore` comments (fix types instead)

**Component Structure:**
- Composition API (`<script setup>`)
- Single-file components (SFC)
- shadcn-vue components (src/components/ui/)
  - Copy-paste architecture (components owned by project)
  - Customizable via props and className
  - Variants managed via class-variance-authority (CVA)
  - Accessible by default (Radix Vue primitives)
- Props with TypeScript interfaces
- Emits with TypeScript types
- Composables for shared logic (useApi, useAuth, useWebSocket)

**shadcn-vue Component Usage Pattern:**
```vue
<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
</script>

<template>
  <Button variant="default">Click Me</Button>
  <Dialog v-model:open="showDialog">
    <DialogContent>
      <DialogTitle>Dialog Title</DialogTitle>
      <!-- content -->
    </DialogContent>
  </Dialog>
</template>
```

**State Management:**
- Pinia stores for global state (auth, theme)
- TanStack Vue Query for server state (tokens, usage, limits)
- Local component state for UI-only state

**Out of Scope:** Unit tests (Vitest), E2E tests (Playwright), component tests (Cypress).

---

## 5. Technical Specifications

### 5.1 Technology Stack

**Frontend Framework:**
- Vue 3.5.24 (Composition API, `<script setup>`)
- TypeScript 5.9.3 (strict mode)
- Vite 7.2.4 (build tool, dev server)

**UI Component Library:**
- **shadcn-vue 2.4.2** (component library built on Radix Vue)
  - 12 core components installed (see §5.2 for full list)
  - Copy-paste architecture (components copied to src/components/ui/)
  - Full accessibility (WCAG 2.1 AA compliant)
  - Customizable via Tailwind CSS variables
- **Radix Vue 1.9.17** (accessible primitives)
  - Dialog, Select, DropdownMenu, Toast primitives
  - Focus management, keyboard navigation, ARIA attributes
  - Foundation for shadcn-vue components
- **class-variance-authority 0.7.1** (variant-based styling)
  - Button variants: default, secondary, outline, ghost, link, destructive
  - Badge variants: default, secondary, destructive, outline
  - Type-safe variant props
- **clsx 2.1.1** (conditional class names)
  - Utility for constructing className strings
  - Used in cn() helper (src/lib/utils.ts)
- **tailwind-merge 3.4.0** (Tailwind class merging)
  - Merges conflicting Tailwind classes
  - Prevents duplicate utility classes
  - Used in cn() helper (src/lib/utils.ts)

**Styling Framework:**
- Tailwind CSS 3.4.17 (utility-first CSS)
  - Custom theme (slate base color, CSS variables)
  - Container, border radius, animations configured
  - Dark mode support via .dark class
  - Note: Downgraded from v4 for shadcn-vue compatibility
- Lucide Vue Next 0.555.0 (icon library)
  - SVG icon components (not yet used, inline SVGs currently)
  - Future replacement for inline SVG

**State Management:**
- Pinia 3.0.4 (global state)
- TanStack Vue Query 5.92.0 (server state, caching)

**Routing:**
- Vue Router 4.6.3 (client-side routing)

**HTTP Client:**
- Fetch API (native browser, no axios)

**WebSocket Client:**
- WebSocket API (native browser, no socket.io)

---

### 5.2 Project Structure

```
module/iron_dashboard/
├── src/
│   ├── main.ts                 # App entry point
│   ├── App.vue                 # Root component
│   ├── router/
│   │   └── index.ts            # Route definitions
│   ├── views/
│   │   ├── LoginView.vue       # FR-1: Authentication
│   │   ├── DashboardView.vue   # FR-2: Dashboard
│   │   ├── TokensView.vue      # FR-3: Token management
│   │   ├── UsageView.vue       # FR-4: Usage analytics
│   │   ├── LimitsView.vue      # FR-5: Budget limits
│   │   └── TracesView.vue      # FR-6: Request traces
│   ├── components/
│   │   ├── ui/                 # shadcn-vue components (copy-paste architecture)
│   │   │   ├── button/         # Button component (6 variants)
│   │   │   ├── dialog/         # Dialog component (9 subcomponents)
│   │   │   ├── card/           # Card component (6 subcomponents)
│   │   │   ├── input/          # Input component
│   │   │   ├── label/          # Label component
│   │   │   ├── badge/          # Badge component (4 variants)
│   │   │   ├── select/         # Select component (11 subcomponents)
│   │   │   ├── separator/      # Separator component
│   │   │   ├── skeleton/       # Skeleton loader component
│   │   │   ├── alert/          # Alert component (3 subcomponents)
│   │   │   ├── toast/          # Toast notification (10 subcomponents)
│   │   │   └── dropdown-menu/  # DropdownMenu (14 subcomponents)
│   │   ├── MainLayout.vue      # Authenticated page layout
│   │   └── HelloWorld.vue      # Legacy template component (unused)
│   ├── composables/
│   │   ├── useApi.ts           # REST API client
│   │   ├── useWebSocket.ts     # WebSocket client
│   │   └── useAuth.ts          # Auth helpers
│   ├── stores/
│   │   ├── auth.ts             # Auth state (Pinia)
│   │   └── theme.ts            # Theme state (Pinia)
│   └── types/
│       └── api.ts              # TypeScript types (match backend)
├── public/
│   └── favicon.ico             # App icon
├── index.html                  # HTML entry point
├── package.json                # npm dependencies
├── package-lock.json           # Dependency lock
├── vite.config.ts              # Vite configuration
├── tsconfig.json               # TypeScript config (strict)
├── tsconfig.app.json           # App-specific TypeScript config
├── tsconfig.node.json          # Node-specific TypeScript config
├── tailwind.config.js          # Tailwind CSS config
├── postcss.config.js           # PostCSS config (Tailwind)
├── .env.example                # Environment variable template
├── spec.md                     # This file
├── readme.md                   # User-facing documentation
├── tests/
│   ├── readme.md               # Test organization guide
│   └── manual/
│       └── readme.md           # Manual testing plan
└── docs/
    ├── readme.md               # Documentation index
    ├── architecture.md         # Technical architecture guide
    └── api_integration.md      # Backend integration guide
```

---

### 5.3 Build Configuration

**Development Server:**
```bash
npm run dev
# Starts Vite dev server on http://localhost:5173
# Hot module replacement (HMR) enabled
# Source maps enabled
```

**Production Build:**
```bash
npm run build
# Compiles TypeScript (vue-tsc)
# Bundles with Vite (rollup)
# Minifies JavaScript (terser)
# Generates source maps (hidden)
# Output: dist/
```

**Preview Production Build:**
```bash
npm run preview
# Serves dist/ on http://localhost:4173
# Simulates production environment
```

**Build Output:**
```
dist/
├── index.html              # Entry point
├── assets/
│   ├── index-[hash].js     # Main bundle (~136 KB uncompressed, ~50 KB gzipped)
│   ├── index-[hash].css    # Main styles (~32 KB uncompressed, ~6.4 KB gzipped)
│   └── [chunk]-[hash].js   # Lazy-loaded route chunks
└── favicon.ico
```

---

### 5.4 Environment Variables

**Required:**
- `VITE_API_URL` - Backend API base URL (default: `http://localhost:3000`)

**Optional:**
- `VITE_WS_URL` - WebSocket server URL (default: `ws://localhost:8080`)
- `VITE_AUTH_TOKEN_KEY` - localStorage key for JWT (default: `auth_token`)

**Configuration Files:**
- `.env.local` - Local development overrides (gitignored)
- `.env.example` - Template for required variables

**Example `.env.local`:**
```
VITE_API_URL=http://localhost:3000
VITE_WS_URL=ws://localhost:8080
```

---

## 6. API Integration

### 6.1 REST API Endpoints

**Backend:** iron_api (Axum Rust server)
**Base URL:** `http://localhost:3000`

**Authentication:**
| Method | Path | Description | Request | Response |
|--------|------|-------------|---------|----------|
| POST | /api/auth/login | User login | LoginRequest | LoginResponse |
| POST | /api/auth/logout | User logout | - | SuccessResponse |

**Tokens:**
| Method | Path | Description | Request | Response |
|--------|------|-------------|---------|----------|
| GET | /api/tokens | List all tokens | - | TokenMetadata[] |
| GET | /api/tokens/:id | Get token details | - | TokenMetadata |
| POST | /api/tokens | Create new token | CreateTokenRequest | CreateTokenResponse |
| POST | /api/tokens/:id/rotate | Rotate token | - | CreateTokenResponse |
| POST | /api/tokens/:id/revoke | Revoke token | - | SuccessResponse |

**Usage:**
| Method | Path | Description | Response |
|--------|------|-------------|----------|
| GET | /api/usage | List all usage records | UsageRecord[] |
| GET | /api/usage/stats | Get usage statistics | UsageStats |
| GET | /api/usage/token/:id | Get usage by token | UsageRecord[] |

**Limits:**
| Method | Path | Description | Request | Response |
|--------|------|-------------|---------|----------|
| GET | /api/limits | List all limits | - | LimitRecord[] |
| GET | /api/limits/:id | Get limit details | - | LimitRecord |
| POST | /api/limits | Create new limit | CreateLimitRequest | LimitRecord |
| PUT | /api/limits/:id | Update limit | UpdateLimitRequest | LimitRecord |
| DELETE | /api/limits/:id | Delete limit | - | SuccessResponse |

**Traces:**
| Method | Path | Description | Response |
|--------|------|-------------|----------|
| GET | /api/traces | List all traces | TraceRecord[] |
| GET | /api/traces/:id | Get trace details | TraceRecord |

---

### 6.2 TypeScript Type Definitions

**Type Mapping (Frontend ↔ Backend):**

Frontend types must match backend Rust schemas exactly to ensure type safety.

```typescript
// src/composables/useApi.ts (existing implementation)

interface TokenMetadata {
  id: number
  user_id: string
  project_id?: string
  name?: string
  created_at: number
  last_used_at?: number
  is_active: boolean
}

interface CreateTokenRequest {
  user_id: string
  project_id?: string
  description?: string
}

interface CreateTokenResponse {
  id: number
  token: string
  user_id: string
  project_id?: string
  description?: string
  created_at: number
}

interface UsageRecord {
  id: number
  token_id: number
  provider: string
  model: string
  input_tokens: number
  output_tokens: number
  cost: number
  timestamp: number
}

interface UsageStats {
  total_requests: number
  total_input_tokens: number
  total_output_tokens: number
  total_cost: number
  by_provider: {
    provider: string
    requests: number
    cost: number
  }[]
  by_model: {
    model: string
    requests: number
    cost: number
  }[]
}

interface LimitRecord {
  id: number
  user_id: string
  project_id?: string
  limit_type: string
  limit_value: number
  period: string
  created_at: number
}

interface TraceRecord {
  id: number
  token_id: number
  request_id: string
  provider: string
  model: string
  input_tokens: number
  output_tokens: number
  cost: number
  timestamp: number
  metadata?: Record<string, unknown>
}
```

---

### 6.3 WebSocket Integration

**WebSocket Endpoint:**
- URL: `ws://localhost:8080/ws`
- Protocol: JSON messages
- Reconnect: Automatic (exponential backoff)

**Message Format:**
```typescript
interface WebSocketMessage {
  type: 'trace' | 'agent_status' | 'budget_update'
  data: unknown
}

// Example trace message
{
  type: 'trace',
  data: {
    id: 123,
    token_id: 1,
    request_id: 'req-abc',
    provider: 'openai',
    model: 'gpt-4o',
    input_tokens: 1234,
    output_tokens: 567,
    cost: 0.12,
    timestamp: 1733404800
  }
}
```

**Connection Lifecycle:**
1. Frontend connects on mount (DashboardView.vue)
2. Backend sends initial state (all agents, budget)
3. Backend streams updates (new traces, agent status changes)
4. Frontend updates UI in real-time
5. Frontend disconnects on unmount

**Error Handling:**
- Connection error → Retry after 1s, 2s, 4s, 8s (exponential backoff, max 30s)
- Parse error → Log to console, ignore message
- Backend unavailable → Show "Offline" indicator in UI

**Out of Scope:** WebSocket authentication (JWT in initial handshake), binary messages (Protocol Buffers).

---

## 7. Testing Strategy

### 7.1 Manual Testing (Pilot)

**Primary Testing Method:**
Manual testing via browser developer tools and user interaction.

**Test Plan Location:**
`tests/manual/readme.md`

**Test Categories:**
1. Authentication flow (login, logout, session persistence)
2. Token management (create, rotate, revoke)
3. Usage analytics (data display, filtering)
4. Budget limits (create, update, delete)
5. Request traces (list, detail view)
6. Responsive layout (desktop 1920x1080, mobile 390x844)
7. Keyboard navigation (Tab, Enter, Escape)
8. Screen reader compatibility (NVDA, JAWS)

**Test Execution:**
1. Start backend: `cd /path/to/iron_api && cargo run`
2. Start frontend: `cd /path/to/iron_dashboard && npm run dev`
3. Navigate to `http://localhost:5173`
4. Execute manual test cases (see `tests/manual/readme.md`)

---

### 7.2 Automated Testing (Future)

**Deferred to Post-Pilot:**

**Unit Tests (Vitest):**
- Composable functions (useApi, useAuth, useWebSocket)
- Utility functions (date formatting, cost calculation)
- Store logic (Pinia actions, getters)

**Component Tests (Cypress):**
- Component rendering (props, slots, emits)
- User interactions (button clicks, form submissions)
- Conditional rendering (loading states, error states)

**E2E Tests (Playwright):**
- Full user flows (login → create token → view usage)
- Cross-browser testing (Chrome, Firefox, Safari)
- Visual regression testing (Percy, Chromatic)

**Rationale:** Conference demo prioritizes manual testing to minimize development time. Automated tests add value for long-term maintenance but are not critical for 5-minute demo.

---

## 8. Deployment

### 8.1 Development Deployment

**Local Development:**
```bash
# Terminal 1: Start backend
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_api
cargo run

# Terminal 2: Start frontend
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_dashboard
npm run dev
```

**Access:**
- Frontend: `http://localhost:5173`
- Backend API: `http://localhost:3000`
- WebSocket: `ws://localhost:8080`

---

### 8.2 Production Deployment (Future)

**Build:**
```bash
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_dashboard
npm run build
# Output: dist/ (static files)
```

**Deployment Options:**
1. **Static hosting** (Vercel, Netlify, Cloudflare Pages)
   - Upload `dist/` directory
   - Configure environment variables (VITE_API_URL)
   - Enable SPA fallback (all routes → index.html)

2. **Self-hosted** (Nginx, Apache, Caddy)
   - Serve `dist/` as static files
   - Configure reverse proxy to backend API
   - Enable HTTPS (Let's Encrypt)

3. **Embedded in backend** (Rust binary)
   - Bundle `dist/` into iron_api binary (tower-http ServeDir)
   - Serve frontend and API from single process
   - Simplifies deployment (single binary)

**Environment Variables (Production):**
```
VITE_API_URL=https://api.ironcage.com
VITE_WS_URL=wss://api.ironcage.com
```

**Out of Scope:** Docker deployment, Kubernetes, CDN configuration.

---

## 9. Migration and Integration

### 9.1 Migration from `frontend/` to `module/iron_dashboard/`

**Status:** In Progress (Phase 1 of 5)

**Migration Plan:**
See `./-frontend_migration_plan_refined_tdd.md` for complete plan.

**Phases:**
- Phase 0: Create verification suite (RED) ✅
- Phase 1: Create documentation (this file + 6 others) ← IN PROGRESS
- Phase 2: Move directory (`frontend/` → `module/iron_dashboard/`)
- Phase 3: Update references (workspace readme.md, security audit)
- Phase 4: Refinement (REFACTOR)

**Rationale:**
Frontend was originally at `dev/frontend/` (root level). Moving to `module/iron_dashboard/` ensures consistency with other Iron Cage modules (iron_api, iron_cli, iron_runtime) and applies same documentation standards (spec.md, readme.md, tests/, docs/).

---

### 9.2 Workspace Integration

**Workspace Structure:**
```
/home/user1/pro/lib/wip_iron/iron_cage/dev/
├── module/
│   ├── iron_api/               # Backend (Axum REST + WebSocket)
│   ├── iron_cli/               # CLI (agent execution)
│   ├── iron_runtime/           # Runtime (agent orchestration)
│   ├── iron_dashboard/          # Frontend (Vue 3 dashboard) ← THIS MODULE
│   └── ...
├── Cargo.toml                  # Rust workspace manifest
├── readme.md                   # Workspace documentation
└── scripts/
    ├── build_all.sh            # Build Rust + Frontend
    ├── test_all.sh             # Test Rust + Frontend
    └── verify_frontend_migration.sh  # Migration verification
```

**Cargo.toml:**
Frontend is NOT listed in `members = [...]` because it's a Node.js project, not a Rust crate. It's documented in workspace readme.md instead.

**Build Integration:**
- `scripts/build_all.sh` builds Rust workspace + frontend
- `scripts/test_all.sh` runs Rust tests + frontend tests (when available)

---

## 10. Known Issues and Limitations

### 10.1 Current Issues

1. **No WebSocket reconnection UI** - When connection drops, no visible indicator to user
   - Workaround: Refresh page manually
   - Fix: Add "Offline" badge in header

2. **~~No loading states for API calls~~** - ✅ RESOLVED
   - Fixed by shadcn-vue Skeleton component
   - Loading states available but not yet implemented in all views
   - Remaining work: Add Skeleton to DashboardView, UsageView, TracesView

3. **No pagination for traces** - Performance degrades with >1000 traces
   - Workaround: Limit demo to <100 traces
   - Fix: Add virtual scrolling (TanStack Virtual)

---

### 10.2 Pilot Limitations

1. **Single-user only** - No user management, roles, or permissions
2. **No persistent storage** - Tokens and limits stored in backend memory (lost on restart)
3. **No error recovery** - API errors displayed but not retried
4. **No mobile optimization** - Layout works but not optimized for touch
5. **No offline mode** - Requires backend to be running
6. **No export functionality** - Cannot export usage data to CSV
7. **No dark mode** - Only light theme available (dark mode toggle exists but not styled)

**Rationale:** These limitations are acceptable for conference demo. Full platform development addresses them post-pilot.

---

## 11. Future Enhancements

### 11.1 Short-Term (Post-Pilot)

1. Add automated tests (Vitest + Playwright)
2. ~~Implement dark mode styling~~ → ✅ CSS variables ready, toggle component needed
3. Add pagination for traces (virtual scrolling)
4. ~~Add loading states (skeleton loaders)~~ → ✅ Skeleton component available (needs implementation)
5. Add WebSocket reconnection UI
6. Add error retry logic (exponential backoff)
7. **NEW:** Migrate remaining views to shadcn-vue (LimitsView, DashboardView, UsageView, TracesView, LoginView)
8. **NEW:** Replace inline SVG icons with Lucide Vue components
9. **NEW:** Add component usage examples to docs/components.md

---

### 11.2 Long-Term (Full Platform)

1. Multi-user management (user accounts, roles, permissions)
2. Custom dashboard builder (drag-drop widgets)
3. Advanced analytics (charts, trends, forecasts)
4. Export functionality (CSV, PDF reports)
5. Mobile native app (iOS, Android)
6. Notification system (push, email, SMS)
7. Audit log viewer (historical actions)
8. Internationalization (i18n, multiple languages)
9. Collaborative features (comments, sharing)
10. API documentation (OpenAPI/Swagger UI)

---

## 12. Appendix

### 12.1 Glossary

- **SPA** - Single-Page Application (client-side routing)
- **SFC** - Single-File Component (Vue .vue files)
- **SSR** - Server-Side Rendering (not used in pilot)
- **HMR** - Hot Module Replacement (Vite dev feature)
- **ARIA** - Accessible Rich Internet Applications (accessibility)
- **WCAG** - Web Content Accessibility Guidelines
- **JWT** - JSON Web Token (authentication)
- **CORS** - Cross-Origin Resource Sharing (browser security)
- **XSS** - Cross-Site Scripting (security vulnerability)
- **CSRF** - Cross-Site Request Forgery (security vulnerability)

---

### 12.2 References

- Vue 3 Documentation: https://vuejs.org/guide/
- Vite Documentation: https://vitejs.dev/guide/
- **shadcn-vue Documentation: https://www.shadcn-vue.com/**
- Radix Vue Documentation: https://www.radix-vue.com/
- **class-variance-authority Documentation: https://cva.style/docs**
- TanStack Query Documentation: https://tanstack.com/query/latest
- Pinia Documentation: https://pinia.vuejs.org/
- Tailwind CSS Documentation: https://tailwindcss.com/docs
- WCAG 2.1 Guidelines: https://www.w3.org/WAI/WCAG21/quickref/

---

## Revision History

- **2025-12-07 (v0.2):** Added § 2.3 Deployment Context - distinguish pilot vs production WebSocket connections and authentication
- **2025-12-05 (v0.1):** Initial specification

---

**End of Specification**
