# iron_dashboard

Web dashboard for real-time Iron Cage agent monitoring and token management.

## Scope

**Responsibilities:**
Provides Vue 3-based web interface for monitoring agent execution, managing API tokens, tracking usage costs, setting budget limits, and viewing request traces. Features real-time WebSocket updates, REST API integration with iron_api backend, and accessible UI components. Supports authentication, session management, and responsive layouts for desktop/mobile. Requires Node.js 20+, modern browsers (Chrome 120+, Firefox 120+, Safari 17+).

**In Scope:**
- Authentication and session management (JWT tokens, localStorage)
- Dashboard view (agent status, budget metrics, recent requests)
- Token management (create, rotate, revoke API tokens)
- Usage analytics (cost breakdown by provider/model)
- Budget limit management (create, update, delete limits)
- Request trace visualization (detailed per-request data)
- Real-time updates (WebSocket integration)
- Responsive layout (desktop 1920x1080, mobile 390x844)
- Accessibility (WCAG 2.1 Level AA, keyboard navigation, screen readers)
- REST API integration (iron_api endpoints)

**Out of Scope:**
- Multi-user management → Pilot is single-user (see spec.md §2.2)
- Role-based access control → Pilot has no roles (see spec.md §2.2)
- Advanced analytics → Pilot uses tables only (see spec.md §FR-4)
- Export functionality → Pilot is view-only (see spec.md §2.2)
- Backend implementation → Use iron_api module
- API authentication → Backend responsibility (see iron_api/spec.md)
- Data persistence → Backend responsibility (see iron_api/spec.md)

---

## Installation

### Prerequisites

- **Node.js:** 20.0.0 or higher
- **npm:** 10.0.0 or higher
- **Backend:** iron_api running on http://localhost:3000

### Setup

```bash
# Navigate to frontend directory
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_dashboard

# Install dependencies
npm install

# Copy environment template
cp .env.example .env.local

# Edit environment variables (if needed)
# VITE_API_URL defaults to http://localhost:3000
```

---

## Features

### Authentication
- Secure login/logout flow
- JWT token storage (localStorage)
- Session persistence across page refreshes
- Automatic redirect to /login when unauthenticated

### Dashboard
- Real-time agent execution status
- Budget metrics (spent, remaining, percentage)
- Recent request stream (model, tokens, cost)
- WebSocket updates (< 100ms latency)

### Token Management
- Create API tokens (with optional project_id, description)
- Rotate tokens (invalidate old, generate new)
- Revoke tokens (mark as inactive)
- Token masking (show only once after creation)

### Usage Analytics
- Total requests, input/output tokens, cost
- Cost breakdown by provider (OpenAI, Anthropic)
- Cost breakdown by model (gpt-4o, claude-3.5-sonnet)
- Real-time data updates

### Budget Limits
- Create limits (daily_cost, monthly_cost, per_request_cost, total_requests)
- Update limit values and periods
- Delete limits with confirmation
- Limit enforcement (backend)

### Request Traces
- Detailed per-request data (request_id, provider, model, tokens, cost)
- Trace detail view (expandable JSON metadata)
- Sortable, filterable table
- Real-time trace stream

### Accessibility (WCAG 2.1 Level AA Compliant)
- **shadcn-vue components:** Built on Radix Vue primitives for accessibility
  - Dialog: Focus trapping, keyboard navigation, ARIA attributes
  - Select: Arrow key navigation, Escape closes, screen reader announcements
  - Button: Focus indicators, disabled states, keyboard activation
- Semantic HTML (nav, main, section, article)
- ARIA labels (buttons, inputs, links)
- Keyboard navigation (Tab, Enter, Escape, Arrow keys)
- Focus indicators (visible ring-2 outlines)
- Color contrast ≥4.5:1 (measured via axe DevTools)
- Screen reader support (NVDA, JAWS tested)

---

## Development

### Start Development Server

```bash
npm run dev
```

Starts Vite dev server on **http://localhost:5173** with hot module replacement (HMR).

**Backend Required:** Ensure iron_api is running on http://localhost:3000 before starting frontend.

```bash
# Terminal 1: Start backend
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_api
cargo run

# Terminal 2: Start frontend
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_dashboard
npm run dev
```

### Build for Production

```bash
npm run build
```

Compiles TypeScript and bundles assets to `dist/` directory (~200 KB gzipped).

**Output:**
- `dist/index.html` - Entry point
- `dist/assets/` - Bundled JavaScript and CSS

### Preview Production Build

```bash
npm run preview
```

Serves production build on **http://localhost:4173** for local testing.

### Type Checking

```bash
npx vue-tsc --noEmit
```

Runs TypeScript compiler without emitting files (type validation only).

### Linting

```bash
npx eslint src/
```

Runs ESLint on source files (Vue + TypeScript rules).

---

## Project Structure

```
module/iron_dashboard/
├── src/                    # Source code
│   ├── main.ts            # App entry point
│   ├── App.vue            # Root component
│   ├── router/            # Vue Router configuration
│   ├── views/             # Page components (Login, Dashboard, Tokens, etc.)
│   ├── components/        # Reusable UI components
│   ├── composables/       # Composition API functions (useApi, useAuth)
│   ├── stores/            # Pinia state stores (auth, theme)
│   └── types/             # TypeScript type definitions
├── public/                # Static assets (favicon)
├── tests/                 # Test organization
│   ├── readme.md          # Test strategy documentation
│   └── manual/            # Manual testing procedures
├── docs/                  # Technical documentation
│   ├── readme.md          # Documentation index
│   ├── architecture.md    # Technical architecture guide
│   └── api_integration.md # Backend integration guide
├── spec.md                # Complete specification (FR, NFR, API)
├── readme.md              # This file
├── package.json           # npm dependencies and scripts
├── vite.config.ts         # Vite build configuration
├── tsconfig.json          # TypeScript configuration (strict mode)
├── tailwind.config.js     # Tailwind CSS configuration
└── .env.example           # Environment variable template
```

---

## Responsibility Table

**CRITICAL:** This table lists EVERY file and subdirectory in module/iron_dashboard/ (Complete Entity Coverage per files_structure.rulebook.md).

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|----------------|--------------|-------|--------------|
| `spec.md` | Define frontend requirements and architecture | Requirements → Specification | All FR/NFR, API contracts, tech stack, deployment | Implementation details (see src/), test procedures (see tests/) |
| `readme.md` | User-facing documentation and setup guide | - → Setup/usage guide | Installation, features, development workflow, responsibility table | Internal architecture (see docs/architecture.md), requirements (see spec.md) |
| `src/` | Frontend source code and application logic | User interactions → UI updates | Vue components, composables, stores, routing, types | Build artifacts (dist/), tests (tests/), documentation (docs/) |
| `tests/` | Test organization and procedures | - → Test suite | Manual testing plan, future automated tests | Source code testing (see src/), integration tests (backend) |
| `docs/` | Technical documentation for developers | - → Documentation | Architecture, API integration, shadcn-vue component inventory, guides | User-facing docs (readme.md), requirements (spec.md), test docs (tests/) |
| `public/` | Static assets served as-is | - → Public assets | Favicon, robots.txt, static images | Source code (src/), build output (dist/) |
| `dist/` | Production build output (generated) | src/ → Bundled app | Minified JS/CSS, optimized HTML, assets | Source code (src/), temporary files (.vite/) |
| `node_modules/` | npm package dependencies (generated) | package.json → Installed packages | All npm dependencies | Source code (src/), documentation (docs/), custom code |
| `.vite/` | Vite dev server cache (generated) | - → Build cache | Dev server metadata, HMR cache | Source code (src/), production builds (dist/) |
| `package.json` | npm project manifest and scripts | - → Dependency/script config | Production + dev dependencies, npm scripts, metadata | Dependency lock (package-lock.json), actual code (src/) |
| `package-lock.json` | npm dependency lock file (generated) | package.json → Locked versions | Exact dependency versions, integrity hashes | Manual editing (npm manages), source code (src/) |
| `vite.config.ts` | Vite build tool configuration | - → Build settings | Dev server, build options, plugins, aliases | Application code (src/), dependencies (node_modules/) |
| `tsconfig.json` | TypeScript compiler configuration | - → Type checking rules | Strict mode, paths, lib, module resolution | Application code (src/), build config (vite.config.ts) |
| `tsconfig.app.json` | App-specific TypeScript config | tsconfig.json → App settings | Extends base, app-specific includes/excludes | Node scripts (tsconfig.node.json), tests |
| `tsconfig.node.json` | Node-specific TypeScript config | tsconfig.json → Node settings | Extends base, config file type checking | Application code (tsconfig.app.json) |
| `tailwind.config.js` | Tailwind CSS configuration | - → Utility classes | Theme, colors, spacing, plugins, content paths | Application styling (src/), build output (dist/) |
| `postcss.config.js` | PostCSS configuration for Tailwind | - → CSS processing | Tailwind plugin, autoprefixer | Application styling (src/), build config (vite.config.ts) |
| `.env.example` | Environment variable template | - → Config template | Required env vars (VITE_API_URL), defaults | Actual values (.env.local, gitignored) |
| `.env.local` | Local environment overrides (gitignored) | .env.example → Local config | Dev-specific API URLs, feature flags | Committed config (.env.example), source code (src/) |
| `.gitignore` | Git exclusion patterns | - → Ignore rules | node_modules/, dist/, .env.local, IDE files | Source code tracking (src/), documentation (docs/) |
| `index.html` | HTML entry point for Vite | - → App shell | Root div, script tag, meta tags | Application logic (src/main.ts), styling (src/App.vue) |
| `README.md` (original) | Legacy README from Vue template | - → Generic template | Vue + Vite setup instructions (obsolete) | Project-specific docs (readme.md with lowercase) |

**Complete Entity Coverage Verified:** 22 entities listed (all files and directories in module/iron_dashboard/).

**Note:** `README.md` (uppercase, original Vue template) is superseded by `readme.md` (lowercase, iron_cage standard). The uppercase version should be removed after migration completes.

---

## Documentation

### Primary Documentation
- [**spec.md**](spec.md) - Complete specification (1890 lines, FR/NFR, API contracts)
- [**readme.md**](readme.md) - This file (setup, features, development)

### Test Documentation
- [tests/readme.md](tests/readme.md) - Test organization and strategy
- [tests/manual/readme.md](tests/manual/readme.md) - Manual testing procedures (8 test categories)

### Technical Documentation
- [docs/readme.md](docs/readme.md) - Documentation index
- [docs/architecture.md](docs/architecture.md) - Vue 3 architecture, state management, routing
- [docs/api_integration.md](docs/api_integration.md) - Backend integration, REST endpoints, WebSocket

### External Documentation
- [Vue 3 Documentation](https://vuejs.org/guide/) - Framework reference
- [Vite Documentation](https://vitejs.dev/guide/) - Build tool reference
- [Radix Vue Documentation](https://www.radix-vue.com/) - UI primitives reference
- [TanStack Query Documentation](https://tanstack.com/query/latest) - Async state management
- [Tailwind CSS Documentation](https://tailwindcss.com/docs) - Utility classes reference

---

## Technology Stack

- **Framework:** Vue 3.5.24 (Composition API, `<script setup>`)
- **Language:** TypeScript 5.9.3 (strict mode)
- **Build Tool:** Vite 7.2.4 (dev server, bundler)
- **UI Components:** shadcn-vue 2.4.2 (production-ready components)
  - 12 components installed (Button, Dialog, Card, Input, Label, Badge, Select, Separator, Skeleton, Alert, Toast, DropdownMenu)
  - Copy-paste architecture (components owned by project)
  - Built on Radix Vue primitives (accessibility)
  - Variants via class-variance-authority (type-safe)
- **UI Primitives:** Radix Vue 1.9.17 (accessible headless components)
- **Styling:** Tailwind CSS 3.4.17 (utility-first, CSS variables for theming)
- **Icons:** Lucide Vue Next 0.555.0 (SVG icon components, not yet used)
- **State Management:** Pinia 3.0.4 (global state)
- **Server State:** TanStack Vue Query 5.92.0 (async state, caching)
- **Routing:** Vue Router 4.6.3 (client-side routing)

---

## Browser Compatibility

**Supported:**
- Chrome 120+ (primary)
- Firefox 120+
- Safari 17+
- Edge 120+

**Not Supported:**
- Internet Explorer (any version)
- Chrome < 100
- Safari < 16

---

## Migration Status

**Current Status:** Phase 1 - Documentation Creation

This module was migrated from `dev/frontend/` to `dev/module/iron_dashboard/` to maintain consistency with other Iron Cage modules (iron_api, iron_cli, iron_runtime). The migration follows TDD principles with verification scripts ensuring correctness.

**Migration Plan:** See `./-frontend_migration_plan_refined_tdd.md` for complete plan.

**Verification:** Run `./scripts/verify_frontend_migration.sh` to check migration status.

---

## License

MIT
