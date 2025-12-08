# tests/

Test organization and strategy for iron_dashboard.

## Organization

**Primary Testing Method (Pilot):**
Manual testing via browser interaction and developer tools.

**Future Testing Methods (Post-Pilot):**
Automated tests (unit tests with Vitest, component tests with Cypress, E2E tests with Playwright).

**Current Structure:**
```
tests/
├── readme.md           # This file (test strategy)
└── manual/             # Manual testing procedures
    └── readme.md       # Manual test plan (8 test categories)
```

---

## Test Principles

Following iron_cage test organization standards (test_organization.rulebook.md):

1. **Manual Testing Required for Pilot** - Frontend UI requires human verification
2. **No Mocking** - Use real iron_api backend for integration testing
3. **Loud Failures** - Tests must fail obviously and loudly
4. **Complete Flows** - Test entire user journeys (login → action → verification)
5. **Accessibility Testing** - Keyboard navigation and screen reader compatibility mandatory

---

## Manual Testing (Pilot)

**Location:** `tests/manual/readme.md`

**Test Categories:**
1. Authentication Flow (login, logout, session persistence)
2. Token Management (create, rotate, revoke)
3. Usage Analytics (data display, cost breakdown)
4. Budget Limits (create, update, delete)
5. Request Traces (list view, detail view)
6. Responsive Layout (desktop 1920x1080, mobile 390x844)
7. Keyboard Navigation (Tab, Enter, Escape)
8. Screen Reader Compatibility (NVDA, JAWS)

**Execution:**
```bash
# Terminal 1: Start backend
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_api
cargo run

# Terminal 2: Start frontend
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_dashboard
npm run dev

# Browser: Navigate to http://localhost:5173
# Follow test procedures in tests/manual/readme.md
```

**Acceptance Criteria:**
- All 8 test categories pass without errors
- All UI elements accessible via keyboard
- All actions produce visible feedback (loading states, success/error messages)
- No console errors in browser developer tools

---

## Automated Testing (Future)

**Deferred to Post-Pilot** - Not required for conference demo.

### Unit Tests (Vitest)

**Test Framework:** Vitest (Vite-native test runner)

**Scope:**
- Composable functions (`useApi`, `useAuth`, `useWebSocket`)
- Utility functions (date formatting, cost calculation)
- Store logic (Pinia actions, getters)

**Example:**
```typescript
// tests/unit/useApi.test.ts
import { describe, it, expect, vi } from 'vitest'
import { useApi } from '@/composables/useApi'

describe('useApi', () => {
  it('fetches tokens from API', async () => {
    const api = useApi()
    const tokens = await api.getTokens()
    expect(tokens).toBeInstanceOf(Array)
  })
})
```

**Run:**
```bash
npm run test:unit
```

---

### Component Tests (Cypress)

**Test Framework:** Cypress Component Testing

**Scope:**
- Component rendering (props, slots, emits)
- User interactions (button clicks, form submissions)
- Conditional rendering (loading states, error states)

**Example:**
```typescript
// tests/component/TokenTable.cy.ts
import TokenTable from '@/components/TokenTable.vue'

describe('TokenTable', () => {
  it('renders token list', () => {
    const tokens = [
      { id: 1, name: 'test-token', created_at: 1733404800, is_active: true }
    ]
    cy.mount(TokenTable, { props: { tokens } })
    cy.contains('test-token').should('be.visible')
  })
})
```

**Run:**
```bash
npm run test:component
```

---

### E2E Tests (Playwright)

**Test Framework:** Playwright (cross-browser testing)

**Scope:**
- Full user flows (login → create token → view usage → logout)
- Cross-browser testing (Chrome, Firefox, Safari)
- Visual regression testing (Percy, Chromatic)

**Example:**
```typescript
// tests/e2e/token-management.spec.ts
import { test, expect } from '@playwright/test'

test('create token flow', async ({ page }) => {
  await page.goto('http://localhost:5173')
  await page.fill('[name=username]', 'test')
  await page.fill('[name=password]', 'test')
  await page.click('[type=submit]')
  await page.click('text=Create Token')
  await page.fill('[name=token-name]', 'my-token')
  await page.click('text=Create')
  await expect(page.locator('text=Token created successfully')).toBeVisible()
})
```

**Run:**
```bash
npm run test:e2e
```

---

## Test Configuration (Future)

**Vitest Configuration:**
```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'jsdom',
    globals: true,
    coverage: {
      reporter: ['text', 'html', 'lcov'],
      exclude: ['node_modules/', 'tests/', 'dist/']
    }
  }
})
```

**Playwright Configuration:**
```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './tests/e2e',
  use: {
    baseURL: 'http://localhost:5173',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ]
})
```

---

## Test Coverage Goals (Future)

**Target Coverage:**
- Unit tests: 80% line coverage (composables, utilities, stores)
- Component tests: 70% component coverage (critical UI components)
- E2E tests: 100% happy path coverage (all user flows)

**Critical Components Requiring Tests:**
1. `useApi` composable (all REST endpoints)
2. `useAuth` composable (login, logout, session persistence)
3. `useWebSocket` composable (connection, reconnection, message handling)
4. `auth` store (Pinia)
5. `TokenTable` component (create, rotate, revoke)
6. `CreateTokenModal` component (form validation, submission)

---

## Responsibility Table

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|----------------|--------------|-------|--------------|
| `readme.md` | Test organization and strategy documentation | - → Test guide | Manual testing plan, automated testing future plan, test principles | Actual test procedures (manual/readme.md), test implementation (unit/component/e2e/) |
| `manual/` | Manual testing procedures and checklists | - → Test procedures | All manual test cases, acceptance criteria, execution steps | Automated tests (unit/component/e2e/), test results (external) |

**Complete Entity Coverage Verified:** 2 entities listed (all files and directories in tests/).

---

## Migration Knowledge

**Migration Context:**
Frontend testing strategy created during migration from `dev/frontend/` to `dev/module/iron_dashboard/` (Phase 1 of TDD migration plan).

**Rationale for Manual Testing (Pilot):**
Conference demo prioritizes speed over test coverage. Manual testing allows rapid iteration and immediate feedback. Automated tests provide long-term value but require additional setup time (1-2 days) not available for 5-minute demo.

**Post-Pilot Testing Plan:**
After conference demo, implement automated tests to ensure regression safety for platform development. Manual testing remains valuable for UX verification and accessibility validation.

**Test Knowledge Preservation:**
All test insights must be captured in test documentation (this file and manual/readme.md), not scattered across temporary notes or external documentation.

---

## Running Tests

**Current (Pilot):**
```bash
# Manual testing only
# See tests/manual/readme.md for procedures
```

**Future (Post-Pilot):**
```bash
# All tests
npm run test

# Unit tests only
npm run test:unit

# Component tests only
npm run test:component

# E2E tests only
npm run test:e2e

# Coverage report
npm run test:coverage
```

---

## Known Testing Gaps (Pilot)

1. **No automated tests** - Pilot relies on manual verification only
2. **No regression detection** - Changes require full manual retest
3. **No coverage metrics** - Unknown which code paths are tested
4. **No CI integration** - Tests not run automatically on commits
5. **No visual regression** - UI changes not automatically detected

**Acceptable for Pilot:** These gaps do not impact conference demo execution. Post-pilot development addresses them.

---

## Test Documentation Standards

Following codebase_hygiene.rulebook.md and test_organization.rulebook.md:

1. **Test File Naming:** `[feature].test.ts` (unit), `[Component].cy.ts` (component), `[feature].spec.ts` (E2E)
2. **Test Organization:** Group by feature domain, not test methodology
3. **Test Documentation:** Every test file has file-level doc comment explaining purpose
4. **Knowledge Capture:** Test insights documented here (tests/readme.md), not in code comments
5. **No Disabled Tests:** Fix or remove, never skip or ignore

---

**End of Test Organization Documentation**
